// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! FTR4 tail footer.
//!
//! FTR4 is the authoritative tail footer for crushr archive format v1.
//! It provides offsets/lengths for the tail-frame components and
//! cryptographic integrity for the IDX and optional LEDGER components.
//!
//! This module implements strict, versioned parsing and encoding.
//! All reserved bytes MUST be zero.

use anyhow::{Context, Result, bail, ensure};
use blake3::Hash;
use std::io::{Read, Write};

/// Magic for an FTR4 footer (v1).
pub const FTR4_MAGIC: &[u8; 4] = b"FTR4";

/// FTR4 version supported by this crate.
pub const FTR4_VERSION: u32 = 1;

/// Size of the reserved region in bytes (must be zero).
pub const FTR4_RESERVED_LEN: usize = 28;

/// Total encoded length of an FTR4 footer in bytes.
pub const FTR4_LEN: usize = 4  // magic
    + 4                        // version
    + 4                        // flags
    + 7 * 8                    // offsets/lengths (blocks_end + dct_off + dct_len + idx_off + idx_len + ldg_off + ldg_len)
    + 32                       // index_hash
    + 32                       // ledger_hash
    + FTR4_RESERVED_LEN        // reserved
    + 32; // footer_hash

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ftr4 {
    pub version: u32,
    pub flags: u32,

    /// Offset in the file where the blocks region ends (tail frame begins at or after this).
    pub blocks_end_offset: u64,

    /// Offset/len for DCT1 (0/0 if absent).
    pub dct_offset: u64,
    pub dct_len: u64,

    /// Offset/len for IDX3 (required).
    pub index_offset: u64,
    pub index_len: u64,

    /// Offset/len for LDG1 (0/0 if absent).
    pub ledger_offset: u64,
    pub ledger_len: u64,

    /// BLAKE3 hash of the raw IDX bytes.
    pub index_hash: [u8; 32],

    /// BLAKE3 hash of the raw ledger JSON bytes (or all-zero if absent).
    pub ledger_hash: [u8; 32],

    /// Reserved bytes (must be zero).
    pub reserved: [u8; FTR4_RESERVED_LEN],

    /// BLAKE3 hash of this footer excluding the footer_hash field.
    pub footer_hash: [u8; 32],
}

impl Default for Ftr4 {
    fn default() -> Self {
        Self {
            version: FTR4_VERSION,
            flags: 0,
            blocks_end_offset: 0,
            dct_offset: 0,
            dct_len: 0,
            index_offset: 0,
            index_len: 0,
            ledger_offset: 0,
            ledger_len: 0,
            index_hash: [0u8; 32],
            ledger_hash: [0u8; 32],
            reserved: [0u8; FTR4_RESERVED_LEN],
            footer_hash: [0u8; 32],
        }
    }
}

fn read_u32_le(mut r: impl Read) -> Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).context("read u32")?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64_le(mut r: impl Read) -> Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b).context("read u64")?;
    Ok(u64::from_le_bytes(b))
}

fn write_u32_le(mut w: impl Write, v: u32) -> Result<()> {
    w.write_all(&v.to_le_bytes()).context("write u32")?;
    Ok(())
}

fn write_u64_le(mut w: impl Write, v: u64) -> Result<()> {
    w.write_all(&v.to_le_bytes()).context("write u64")?;
    Ok(())
}

fn is_all_zero(a: &[u8]) -> bool {
    a.iter().all(|&b| b == 0)
}

impl Ftr4 {
    /// Compute the footer hash (BLAKE3) over the footer bytes excluding the footer_hash field.
    pub fn compute_footer_hash(&self) -> Hash {
        let mut buf: Vec<u8> = Vec::with_capacity(FTR4_LEN - 32);
        buf.extend_from_slice(FTR4_MAGIC);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.flags.to_le_bytes());
        buf.extend_from_slice(&self.blocks_end_offset.to_le_bytes());
        buf.extend_from_slice(&self.dct_offset.to_le_bytes());
        buf.extend_from_slice(&self.dct_len.to_le_bytes());
        buf.extend_from_slice(&self.index_offset.to_le_bytes());
        buf.extend_from_slice(&self.index_len.to_le_bytes());
        buf.extend_from_slice(&self.ledger_offset.to_le_bytes());
        buf.extend_from_slice(&self.ledger_len.to_le_bytes());
        buf.extend_from_slice(&self.index_hash);
        buf.extend_from_slice(&self.ledger_hash);
        buf.extend_from_slice(&self.reserved);
        blake3::hash(&buf)
    }

    /// Validate invariants that do not depend on the overall file length.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == FTR4_VERSION,
            "unsupported FTR4 version: {}",
            self.version
        );

        // IDX is required.
        ensure!(self.index_offset != 0, "index_offset must be nonzero");
        ensure!(self.index_len != 0, "index_len must be nonzero");
        ensure!(!is_all_zero(&self.index_hash), "index_hash must be nonzero");

        // Optional DCT.
        if self.dct_len == 0 {
            ensure!(
                self.dct_offset == 0,
                "dct_offset must be 0 when dct_len is 0"
            );
        } else {
            ensure!(
                self.dct_offset != 0,
                "dct_offset must be nonzero when dct_len > 0"
            );
        }

        // Optional ledger.
        if self.ledger_len == 0 {
            ensure!(
                self.ledger_offset == 0,
                "ledger_offset must be 0 when ledger_len is 0"
            );
            ensure!(
                is_all_zero(&self.ledger_hash),
                "ledger_hash must be zero when ledger absent"
            );
        } else {
            ensure!(
                self.ledger_offset != 0,
                "ledger_offset must be nonzero when ledger_len > 0"
            );
            ensure!(
                !is_all_zero(&self.ledger_hash),
                "ledger_hash must be nonzero when ledger present"
            );
        }

        // Reserved bytes must be zero.
        ensure!(is_all_zero(&self.reserved), "reserved bytes must be zero");

        // Basic overflow checks for offset + len (no wrap).
        for (name, off, len) in [
            ("dct", self.dct_offset, self.dct_len),
            ("index", self.index_offset, self.index_len),
            ("ledger", self.ledger_offset, self.ledger_len),
        ] {
            if len != 0 {
                off.checked_add(len)
                    .with_context(|| format!("{name} offset+len overflow"))?;
            }
        }

        // Compute footer hash and compare.
        let expected = self.compute_footer_hash();
        ensure!(
            self.footer_hash == *expected.as_bytes(),
            "footer_hash mismatch"
        );

        // Loose monotonicity constraints (component offsets should be at/after blocks_end_offset).
        for (name, off) in [
            ("dct", self.dct_offset),
            ("index", self.index_offset),
            ("ledger", self.ledger_offset),
        ] {
            if off != 0 {
                ensure!(
                    off >= self.blocks_end_offset,
                    "{name}_offset precedes blocks_end_offset"
                );
            }
        }

        Ok(())
    }

    pub fn read_from(mut r: impl Read) -> Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic).context("read magic")?;
        if &magic != FTR4_MAGIC {
            bail!("bad footer magic: {:?}", magic);
        }

        let version = read_u32_le(&mut r)?;
        let flags = read_u32_le(&mut r)?;
        let blocks_end_offset = read_u64_le(&mut r)?;
        let dct_offset = read_u64_le(&mut r)?;
        let dct_len = read_u64_le(&mut r)?;
        let index_offset = read_u64_le(&mut r)?;
        let index_len = read_u64_le(&mut r)?;
        let ledger_offset = read_u64_le(&mut r)?;
        let ledger_len = read_u64_le(&mut r)?;

        let mut index_hash = [0u8; 32];
        r.read_exact(&mut index_hash).context("read index_hash")?;
        let mut ledger_hash = [0u8; 32];
        r.read_exact(&mut ledger_hash).context("read ledger_hash")?;

        let mut reserved = [0u8; FTR4_RESERVED_LEN];
        r.read_exact(&mut reserved).context("read reserved")?;

        let mut footer_hash = [0u8; 32];
        r.read_exact(&mut footer_hash).context("read footer_hash")?;

        let f = Self {
            version,
            flags,
            blocks_end_offset,
            dct_offset,
            dct_len,
            index_offset,
            index_len,
            ledger_offset,
            ledger_len,
            index_hash,
            ledger_hash,
            reserved,
            footer_hash,
        };
        f.validate()?;
        Ok(f)
    }

    pub fn write_to(&self, mut w: impl Write) -> Result<()> {
        // Enforce invariants before writing.
        self.validate()?;

        w.write_all(FTR4_MAGIC).context("write magic")?;
        write_u32_le(&mut w, self.version)?;
        write_u32_le(&mut w, self.flags)?;
        write_u64_le(&mut w, self.blocks_end_offset)?;
        write_u64_le(&mut w, self.dct_offset)?;
        write_u64_le(&mut w, self.dct_len)?;
        write_u64_le(&mut w, self.index_offset)?;
        write_u64_le(&mut w, self.index_len)?;
        write_u64_le(&mut w, self.ledger_offset)?;
        write_u64_le(&mut w, self.ledger_len)?;
        w.write_all(&self.index_hash).context("write index_hash")?;
        w.write_all(&self.ledger_hash)
            .context("write ledger_hash")?;
        w.write_all(&self.reserved).context("write reserved")?;
        w.write_all(&self.footer_hash)
            .context("write footer_hash")?;
        Ok(())
    }

    /// Create a footer with correct footer_hash computed.
    ///
    /// Callers must set all other fields, including index_hash and optional ledger_hash.
    pub fn finalize(mut self) -> Result<Self> {
        // zero reserved by default
        self.reserved = [0u8; FTR4_RESERVED_LEN];
        // temporarily set footer_hash to zero for validate? We'll set computed and then validate.
        let h = self.compute_footer_hash();
        self.footer_hash = *h.as_bytes();
        self.validate()?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash32(tag: &[u8]) -> [u8; 32] {
        *blake3::hash(tag).as_bytes()
    }

    #[test]
    fn ftr4_round_trip_minimal() {
        let f = Ftr4 {
            version: FTR4_VERSION,
            flags: 0,
            blocks_end_offset: 123,
            dct_offset: 0,
            dct_len: 0,
            index_offset: 456,
            index_len: 789,
            ledger_offset: 0,
            ledger_len: 0,
            index_hash: hash32(b"idx"),
            ledger_hash: [0u8; 32],
            reserved: [0u8; FTR4_RESERVED_LEN],
            footer_hash: [0u8; 32],
        }
        .finalize()
        .unwrap();

        let mut buf = Vec::new();
        f.write_to(&mut buf).unwrap();
        assert_eq!(buf.len(), FTR4_LEN);

        let parsed = Ftr4::read_from(&buf[..]).unwrap();
        assert_eq!(parsed, f);
    }

    #[test]
    fn rejects_nonzero_reserved() {
        let f = Ftr4 {
            blocks_end_offset: 0,
            index_offset: 8,
            index_len: 16,
            index_hash: hash32(b"idx"),
            ..Default::default()
        }
        .finalize()
        .unwrap();

        // Corrupt reserved byte after encoding.
        let mut buf = Vec::new();
        f.write_to(&mut buf).unwrap();
        // reserved begins after: magic(4)+ver(4)+flags(4)+56 + 32 + 32 = 164- (32 footer hash?) Let's locate:
        // We'll just flip a byte near the end but before footer_hash.
        let reserved_start = 4 + 4 + 4 + 56 + 32 + 32;
        assert_eq!(reserved_start, 132);
        buf[reserved_start] = 1;

        assert!(Ftr4::read_from(&buf[..]).is_err());
    }

    #[test]
    fn rejects_footer_hash_mismatch() {
        let f = Ftr4 {
            blocks_end_offset: 0,
            index_offset: 8,
            index_len: 16,
            index_hash: hash32(b"idx"),
            ..Default::default()
        }
        .finalize()
        .unwrap();

        let mut buf = Vec::new();
        f.write_to(&mut buf).unwrap();
        // flip last byte (footer_hash field)
        *buf.last_mut().unwrap() ^= 0xFF;
        assert!(Ftr4::read_from(&buf[..]).is_err());
    }

    #[test]
    fn ledger_presence_rules() {
        let base = Ftr4 {
            blocks_end_offset: 0,
            index_offset: 8,
            index_len: 16,
            index_hash: hash32(b"idx"),
            ..Default::default()
        };

        // ledger_len=0 but ledger_hash nonzero should fail
        let mut bad = base.clone();
        bad.ledger_hash = hash32(b"ledger");
        assert!(bad.finalize().is_err());

        // ledger present but hash zero should fail
        let mut bad2 = base.clone();
        bad2.ledger_offset = 100;
        bad2.ledger_len = 10;
        bad2.ledger_hash = [0u8; 32];
        assert!(bad2.finalize().is_err());
    }
}
