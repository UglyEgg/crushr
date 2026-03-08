//! BLK3 block framing.
//!
//! BLK3 is the canonical block header for crushr v1.x.
//!
//! Layout (little-endian):
//!
//! - magic: [u8;4] = "BLK3"
//! - header_len: u16 (bytes, including magic)
//! - flags: u16
//! - codec: u32
//! - level: i32
//! - dict_id: u32 (0 = none)
//! - raw_len: u64
//! - comp_len: u64
//! - payload_hash: [u8;32] (optional; BLAKE3 of compressed payload bytes)
//! - raw_hash: [u8;32] (optional; BLAKE3 of raw/decompressed bytes)
//! - reserved/pad: zero bytes up to header_len

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};

pub const BLK3_MAGIC: [u8; 4] = *b"BLK3";

/// Maximum accepted BLK3 header length.
///
/// This protects against corrupted header_len values.
pub const BLK3_MAX_HEADER_LEN: usize = 4096;

/// Fixed-size portion of BLK3 header (including magic).
const BLK3_FIXED_LEN: usize = 4  // magic
    + 2  // header_len
    + 2  // flags
    + 4  // codec
    + 4  // level
    + 4  // dict_id
    + 8  // raw_len
    + 8; // comp_len

/// BLK3 flags.
///
/// Only a small subset is defined for v1.0.
/// Unknown bits must be zero in v1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blk3Flags(pub u16);

impl Blk3Flags {
    pub const HAS_PAYLOAD_HASH: u16 = 1 << 0;
    pub const HAS_RAW_HASH: u16 = 1 << 1;
    pub const USES_DICT: u16 = 1 << 2;
    pub const IS_META_FRAME: u16 = 1 << 3;

    pub fn has_payload_hash(self) -> bool {
        (self.0 & Self::HAS_PAYLOAD_HASH) != 0
    }
    pub fn has_raw_hash(self) -> bool {
        (self.0 & Self::HAS_RAW_HASH) != 0
    }
    pub fn uses_dict(self) -> bool {
        (self.0 & Self::USES_DICT) != 0
    }
    pub fn is_meta_frame(self) -> bool {
        (self.0 & Self::IS_META_FRAME) != 0
    }

    pub fn unknown_bits(self) -> u16 {
        let known =
            Self::HAS_PAYLOAD_HASH | Self::HAS_RAW_HASH | Self::USES_DICT | Self::IS_META_FRAME;
        self.0 & !known
    }
}

/// Parsed BLK3 header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blk3Header {
    pub header_len: u16,
    pub flags: Blk3Flags,
    pub codec: u32,
    pub level: i32,
    pub dict_id: u32,
    pub raw_len: u64,
    pub comp_len: u64,
    pub payload_hash: Option<[u8; 32]>,
    pub raw_hash: Option<[u8; 32]>,
}

impl Blk3Header {
    /// Validate v1.0 invariants.
    pub fn validate_v1(&self) -> Result<()> {
        if self.flags.unknown_bits() != 0 {
            bail!(
                "BLK3: unknown flag bits set: 0x{:04x}",
                self.flags.unknown_bits()
            );
        }

        let expected_min = BLK3_FIXED_LEN
            + if self.flags.has_payload_hash() { 32 } else { 0 }
            + if self.flags.has_raw_hash() { 32 } else { 0 };

        let hl = self.header_len as usize;
        if hl < expected_min {
            bail!("BLK3: header_len too small: {} < {}", hl, expected_min);
        }
        if hl > BLK3_MAX_HEADER_LEN {
            bail!(
                "BLK3: header_len too large: {} > {}",
                hl,
                BLK3_MAX_HEADER_LEN
            );
        }

        // Dict binding must be consistent.
        if self.flags.uses_dict() {
            if self.dict_id == 0 {
                bail!("BLK3: USES_DICT set but dict_id==0");
            }
        } else if self.dict_id != 0 {
            bail!("BLK3: dict_id!=0 but USES_DICT not set");
        }

        // Hash presence must match flags.
        if self.flags.has_payload_hash() != self.payload_hash.is_some() {
            bail!("BLK3: payload_hash presence does not match flag");
        }
        if self.flags.has_raw_hash() != self.raw_hash.is_some() {
            bail!("BLK3: raw_hash presence does not match flag");
        }

        Ok(())
    }

    /// Compute the canonical header length for v1.0 (no reserved bytes).
    pub fn canonical_len_v1(&self) -> usize {
        BLK3_FIXED_LEN
            + if self.flags.has_payload_hash() { 32 } else { 0 }
            + if self.flags.has_raw_hash() { 32 } else { 0 }
    }
}

/// Write a BLK3 header (v1.0 canonical; pads with zeros if header_len exceeds canonical length).
pub fn write_blk3_header<W: Write>(mut w: W, h: &Blk3Header) -> Result<()> {
    h.validate_v1()?;

    w.write_all(&BLK3_MAGIC)?;
    w.write_all(&h.header_len.to_le_bytes())?;
    w.write_all(&h.flags.0.to_le_bytes())?;
    w.write_all(&h.codec.to_le_bytes())?;
    w.write_all(&h.level.to_le_bytes())?;
    w.write_all(&h.dict_id.to_le_bytes())?;
    w.write_all(&h.raw_len.to_le_bytes())?;
    w.write_all(&h.comp_len.to_le_bytes())?;

    if let Some(ph) = h.payload_hash {
        w.write_all(&ph)?;
    }
    if let Some(rh) = h.raw_hash {
        w.write_all(&rh)?;
    }

    let written = h.canonical_len_v1();
    let target = h.header_len as usize;
    if target < written {
        bail!(
            "BLK3: header_len {} smaller than bytes written {}",
            target,
            written
        );
    }
    let pad = target - written;
    if pad > 0 {
        // v1.0 requires reserved bytes to be zero.
        let zeros = vec![0u8; pad];
        w.write_all(&zeros)?;
    }

    Ok(())
}

/// Read and validate a BLK3 header.
///
/// Returns the parsed header and the total header length in bytes.
pub fn read_blk3_header<R: Read>(mut r: R) -> Result<Blk3Header> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic).context("reading BLK3 magic")?;
    if magic != BLK3_MAGIC {
        bail!("BLK3: bad magic");
    }

    let header_len = read_u16_le(&mut r).context("reading BLK3 header_len")?;
    let hl = header_len as usize;
    if hl < BLK3_FIXED_LEN {
        bail!("BLK3: header_len too small: {}", hl);
    }
    if hl > BLK3_MAX_HEADER_LEN {
        bail!("BLK3: header_len too large: {}", hl);
    }

    let flags = Blk3Flags(read_u16_le(&mut r).context("reading BLK3 flags")?);
    let codec = read_u32_le(&mut r).context("reading BLK3 codec")?;
    let level = read_i32_le(&mut r).context("reading BLK3 level")?;
    let dict_id = read_u32_le(&mut r).context("reading BLK3 dict_id")?;
    let raw_len = read_u64_le(&mut r).context("reading BLK3 raw_len")?;
    let comp_len = read_u64_le(&mut r).context("reading BLK3 comp_len")?;

    let mut payload_hash: Option<[u8; 32]> = None;
    let mut raw_hash: Option<[u8; 32]> = None;

    if flags.has_payload_hash() {
        let mut h = [0u8; 32];
        r.read_exact(&mut h).context("reading BLK3 payload_hash")?;
        payload_hash = Some(h);
    }
    if flags.has_raw_hash() {
        let mut h = [0u8; 32];
        r.read_exact(&mut h).context("reading BLK3 raw_hash")?;
        raw_hash = Some(h);
    }

    // Skip reserved bytes and enforce they are zero (v1.0).
    let consumed = BLK3_FIXED_LEN
        + if flags.has_payload_hash() { 32 } else { 0 }
        + if flags.has_raw_hash() { 32 } else { 0 };

    if hl < consumed {
        bail!(
            "BLK3: header_len {} smaller than parsed bytes {}",
            hl,
            consumed
        );
    }

    let mut reserved = vec![0u8; hl - consumed];
    if !reserved.is_empty() {
        r.read_exact(&mut reserved)
            .context("reading BLK3 reserved")?;
        if reserved.iter().any(|b| *b != 0) {
            bail!("BLK3: reserved/pad bytes must be zero (v1.0)");
        }
    }

    let h = Blk3Header {
        header_len,
        flags,
        codec,
        level,
        dict_id,
        raw_len,
        comp_len,
        payload_hash,
        raw_hash,
    };

    h.validate_v1()?;
    Ok(h)
}

fn read_u16_le<R: Read>(r: &mut R) -> Result<u16> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn read_u32_le<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_i32_le<R: Read>(r: &mut R) -> Result<i32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(i32::from_le_bytes(b))
}

fn read_u64_le<R: Read>(r: &mut R) -> Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blk3_roundtrip_minimal_no_hashes_no_dict() {
        let h = Blk3Header {
            header_len: BLK3_FIXED_LEN as u16,
            flags: Blk3Flags(0),
            codec: 1,
            level: 3,
            dict_id: 0,
            raw_len: 123,
            comp_len: 45,
            payload_hash: None,
            raw_hash: None,
        };

        let mut buf = Vec::new();
        write_blk3_header(&mut buf, &h).unwrap();
        assert_eq!(buf.len(), BLK3_FIXED_LEN);

        let parsed = read_blk3_header(buf.as_slice()).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn blk3_roundtrip_with_hashes_and_dict() {
        let mut ph = [0u8; 32];
        ph[0] = 1;
        let mut rh = [0u8; 32];
        rh[0] = 2;

        let flags =
            Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH | Blk3Flags::USES_DICT);
        let canonical_len = BLK3_FIXED_LEN + 32 + 32;

        let h = Blk3Header {
            header_len: canonical_len as u16,
            flags,
            codec: 1,
            level: 19,
            dict_id: 7,
            raw_len: 1_000_000,
            comp_len: 123_456,
            payload_hash: Some(ph),
            raw_hash: Some(rh),
        };

        let mut buf = Vec::new();
        write_blk3_header(&mut buf, &h).unwrap();
        assert_eq!(buf.len(), canonical_len);

        let parsed = read_blk3_header(buf.as_slice()).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn blk3_rejects_nonzero_reserved_bytes() {
        let h = Blk3Header {
            header_len: (BLK3_FIXED_LEN + 4) as u16,
            flags: Blk3Flags(0),
            codec: 1,
            level: 3,
            dict_id: 0,
            raw_len: 123,
            comp_len: 45,
            payload_hash: None,
            raw_hash: None,
        };

        let mut buf = Vec::new();
        write_blk3_header(&mut buf, &h).unwrap();
        // Corrupt a reserved byte.
        *buf.last_mut().unwrap() = 1;

        let err = read_blk3_header(buf.as_slice()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("reserved") || msg.contains("zero"));
    }

    #[test]
    fn blk3_rejects_dict_flag_mismatch() {
        let h = Blk3Header {
            header_len: BLK3_FIXED_LEN as u16,
            flags: Blk3Flags(0),
            codec: 1,
            level: 3,
            dict_id: 9,
            raw_len: 1,
            comp_len: 1,
            payload_hash: None,
            raw_hash: None,
        };

        let mut buf = Vec::new();
        // Writer should fail validation first.
        assert!(write_blk3_header(&mut buf, &h).is_err());
    }
}
