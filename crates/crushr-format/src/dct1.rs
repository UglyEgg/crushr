// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! DCT1 dictionary table framing.
//!
//! DCT1 is embedded per tail frame so each tail frame is self-contained.
//!
//! Layout (little-endian):
//!
//! - magic: [u8;4] = "DCT1"
//! - count: u32
//! - entries[count]:
//!   - dict_id: u32 (non-zero; unique within table)
//!   - dict_len: u32
//!   - dict_hash: [u8;32] (BLAKE3 of dict_bytes)
//!   - dict_bytes: [u8;dict_len]

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::io::{Read, Write};

pub const DCT1_MAGIC: [u8; 4] = *b"DCT1";

/// Maximum number of dictionaries allowed in a single DCT1 table.
///
/// This is a corruption guard, not a policy knob.
pub const DCT1_MAX_COUNT: u32 = 65_535;

/// Maximum size for a single dictionary entry.
///
/// Zstd dicts are typically 8KiB..256KiB, but we allow larger for advanced use.
pub const DCT1_MAX_DICT_LEN: u32 = 16 * 1024 * 1024; // 16 MiB

/// Maximum total size of all dictionary bytes in a DCT1 table.
///
/// This prevents pathological allocations on corrupted input.
pub const DCT1_MAX_TOTAL_DICT_BYTES: u64 = 256 * 1024 * 1024; // 256 MiB

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dct1Entry {
    pub dict_id: u32,
    pub dict_bytes: Vec<u8>,
    pub dict_hash: [u8; 32],
}

impl Dct1Entry {
    pub fn compute_hash(dict_bytes: &[u8]) -> [u8; 32] {
        *blake3::hash(dict_bytes).as_bytes()
    }

    pub fn new(dict_id: u32, dict_bytes: Vec<u8>) -> Result<Self> {
        if dict_id == 0 {
            bail!("DCT1: dict_id must be non-zero");
        }
        if dict_bytes.len() > (DCT1_MAX_DICT_LEN as usize) {
            bail!(
                "DCT1: dict_len too large: {} > {}",
                dict_bytes.len(),
                DCT1_MAX_DICT_LEN
            );
        }
        let dict_hash = Self::compute_hash(&dict_bytes);
        Ok(Self {
            dict_id,
            dict_bytes,
            dict_hash,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.dict_id == 0 {
            bail!("DCT1: dict_id must be non-zero");
        }
        if self.dict_bytes.len() > (DCT1_MAX_DICT_LEN as usize) {
            bail!(
                "DCT1: dict_len too large: {} > {}",
                self.dict_bytes.len(),
                DCT1_MAX_DICT_LEN
            );
        }
        let expected = Self::compute_hash(&self.dict_bytes);
        if expected != self.dict_hash {
            bail!("DCT1: dict_hash mismatch for dict_id={}", self.dict_id);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dct1Table {
    pub entries: Vec<Dct1Entry>,
}

impl Dct1Table {
    pub fn new(entries: Vec<Dct1Entry>) -> Result<Self> {
        let t = Self { entries };
        t.validate()?;
        Ok(t)
    }

    pub fn validate(&self) -> Result<()> {
        if self.entries.len() > (DCT1_MAX_COUNT as usize) {
            bail!(
                "DCT1: too many dictionaries: {} > {}",
                self.entries.len(),
                DCT1_MAX_COUNT
            );
        }

        let mut seen: BTreeSet<u32> = BTreeSet::new();
        let mut total: u64 = 0;
        for e in &self.entries {
            if !seen.insert(e.dict_id) {
                bail!("DCT1: duplicate dict_id: {}", e.dict_id);
            }
            e.validate()?;
            total = total
                .checked_add(e.dict_bytes.len() as u64)
                .ok_or_else(|| anyhow::anyhow!("DCT1: total dict bytes overflow"))?;
            if total > DCT1_MAX_TOTAL_DICT_BYTES {
                bail!(
                    "DCT1: total dict bytes too large: {} > {}",
                    total,
                    DCT1_MAX_TOTAL_DICT_BYTES
                );
            }
        }
        Ok(())
    }

    pub fn get(&self, dict_id: u32) -> Option<&Dct1Entry> {
        self.entries.iter().find(|e| e.dict_id == dict_id)
    }
}

pub fn write_dct1<W: Write>(mut w: W, t: &Dct1Table) -> Result<()> {
    t.validate()?;

    w.write_all(&DCT1_MAGIC)?;
    let count: u32 = t.entries.len().try_into().context("DCT1 count overflow")?;
    w.write_all(&count.to_le_bytes())?;

    for e in &t.entries {
        w.write_all(&e.dict_id.to_le_bytes())?;
        let len: u32 = e
            .dict_bytes
            .len()
            .try_into()
            .context("DCT1 dict_len overflow")?;
        w.write_all(&len.to_le_bytes())?;
        w.write_all(&e.dict_hash)?;
        w.write_all(&e.dict_bytes)?;
    }

    Ok(())
}

pub fn read_dct1<R: Read>(mut r: R) -> Result<Dct1Table> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic).context("reading DCT1 magic")?;
    if magic != DCT1_MAGIC {
        bail!("DCT1: bad magic");
    }

    let count = read_u32_le(&mut r).context("reading DCT1 count")?;
    if count > DCT1_MAX_COUNT {
        bail!("DCT1: count too large: {} > {}", count, DCT1_MAX_COUNT);
    }

    let mut entries: Vec<Dct1Entry> = Vec::with_capacity(count as usize);
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    let mut total: u64 = 0;

    for _ in 0..count {
        let dict_id = read_u32_le(&mut r).context("reading DCT1 dict_id")?;
        if dict_id == 0 {
            bail!("DCT1: dict_id must be non-zero");
        }
        if !seen.insert(dict_id) {
            bail!("DCT1: duplicate dict_id: {}", dict_id);
        }

        let dict_len = read_u32_le(&mut r).context("reading DCT1 dict_len")?;
        if dict_len > DCT1_MAX_DICT_LEN {
            bail!(
                "DCT1: dict_len too large for dict_id={}: {} > {}",
                dict_id,
                dict_len,
                DCT1_MAX_DICT_LEN
            );
        }

        let mut dict_hash = [0u8; 32];
        r.read_exact(&mut dict_hash)
            .context("reading DCT1 dict_hash")?;

        let mut dict_bytes = vec![0u8; dict_len as usize];
        r.read_exact(&mut dict_bytes)
            .context("reading DCT1 dict_bytes")?;

        total = total
            .checked_add(dict_len as u64)
            .ok_or_else(|| anyhow::anyhow!("DCT1: total dict bytes overflow"))?;
        if total > DCT1_MAX_TOTAL_DICT_BYTES {
            bail!(
                "DCT1: total dict bytes too large: {} > {}",
                total,
                DCT1_MAX_TOTAL_DICT_BYTES
            );
        }

        let expected = *blake3::hash(&dict_bytes).as_bytes();
        if expected != dict_hash {
            bail!("DCT1: dict_hash mismatch for dict_id={}", dict_id);
        }

        entries.push(Dct1Entry {
            dict_id,
            dict_bytes,
            dict_hash,
        });
    }

    let t = Dct1Table { entries };
    t.validate()?;
    Ok(t)
}

fn read_u32_le<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dct1_round_trip_single() {
        let e = Dct1Entry::new(1, b"hello dict".to_vec()).unwrap();
        let t = Dct1Table::new(vec![e]).unwrap();

        let mut buf = Vec::new();
        write_dct1(&mut buf, &t).unwrap();

        let parsed = read_dct1(buf.as_slice()).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn dct1_round_trip_multi_and_get() {
        let e1 = Dct1Entry::new(1, vec![1u8; 32]).unwrap();
        let e2 = Dct1Entry::new(2, vec![2u8; 64]).unwrap();
        let t = Dct1Table::new(vec![e1.clone(), e2.clone()]).unwrap();

        let mut buf = Vec::new();
        write_dct1(&mut buf, &t).unwrap();

        let parsed = read_dct1(buf.as_slice()).unwrap();
        assert_eq!(parsed.get(1).unwrap(), &e1);
        assert_eq!(parsed.get(2).unwrap(), &e2);
        assert!(parsed.get(3).is_none());
    }

    #[test]
    fn dct1_reject_duplicate_ids() {
        let e1 = Dct1Entry::new(1, vec![1u8; 8]).unwrap();
        let e2 = Dct1Entry::new(1, vec![2u8; 8]).unwrap();
        assert!(Dct1Table::new(vec![e1, e2]).is_err());
    }

    #[test]
    fn dct1_reject_hash_mismatch() {
        let mut buf = Vec::new();
        // Build an entry with an intentionally wrong hash.
        buf.extend_from_slice(&DCT1_MAGIC);
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 32]); // wrong hash
        buf.extend_from_slice(&[9u8, 9, 9, 9]);

        assert!(read_dct1(buf.as_slice()).is_err());
    }
}
