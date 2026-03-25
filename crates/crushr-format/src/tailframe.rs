// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! Tail frame assembly and parsing helpers.
//!
//! Tail frame v1 layout is deterministic:
//! `DCT1? + IDX3 + LDG1? + FTR4`.

use crate::dct1::{DCT1_MAGIC, Dct1Table, read_dct1, write_dct1};
use crate::ftr4::{FTR4_LEN, Ftr4};
use crate::ledger::{LDG1_MAGIC, LedgerBlob, read_ldg1, write_ldg1};
use anyhow::{Context, Result, ensure};
use std::io::Cursor;

pub const IDX3_MAGIC: [u8; 4] = *b"IDX3";
pub const IDX4_MAGIC: [u8; 4] = *b"IDX4";
pub const IDX5_MAGIC: [u8; 4] = *b"IDX5";

fn is_supported_index_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(&IDX3_MAGIC)
        || bytes.starts_with(&IDX4_MAGIC)
        || bytes.starts_with(&IDX5_MAGIC)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailFrameParts {
    pub dct1: Option<Dct1Table>,
    pub idx3_bytes: Vec<u8>,
    pub ldg1: Option<LedgerBlob>,
    pub footer: Ftr4,
}

fn component_slice(frame_bytes: &[u8], start: u64, len: u64) -> Result<&[u8]> {
    let end = start.checked_add(len).context("component range overflow")?;
    ensure!(
        end <= frame_bytes.len() as u64,
        "component range extends past tail frame bytes"
    );

    let start = usize::try_from(start).context("component start offset too large")?;
    let end = usize::try_from(end).context("component end offset too large")?;
    Ok(&frame_bytes[start..end])
}

pub fn assemble_tail_frame(
    blocks_end_offset: u64,
    dct1: Option<&Dct1Table>,
    idx3_bytes: &[u8],
    ldg1: Option<&LedgerBlob>,
) -> Result<Vec<u8>> {
    ensure!(is_supported_index_magic(idx3_bytes), "IDX: bad magic");

    let mut frame = Vec::new();

    let (dct_offset, dct_len) = if let Some(table) = dct1 {
        let start = blocks_end_offset
            .checked_add(frame.len() as u64)
            .context("tail frame offset overflow while placing DCT1")?;
        write_dct1(&mut frame, table)?;
        let len = (frame.len() as u64)
            .checked_sub(start - blocks_end_offset)
            .context("tail frame length underflow for DCT1")?;
        (start, len)
    } else {
        (0, 0)
    };

    let index_offset = blocks_end_offset
        .checked_add(frame.len() as u64)
        .context("tail frame offset overflow while placing IDX3")?;
    frame.extend_from_slice(idx3_bytes);
    let index_len = idx3_bytes.len() as u64;
    let index_hash = *blake3::hash(idx3_bytes).as_bytes();

    let (ledger_offset, ledger_len, ledger_hash) = if let Some(blob) = ldg1 {
        let start = blocks_end_offset
            .checked_add(frame.len() as u64)
            .context("tail frame offset overflow while placing LDG1")?;
        write_ldg1(&mut frame, blob)?;
        let len = (frame.len() as u64)
            .checked_sub(start - blocks_end_offset)
            .context("tail frame length underflow for LDG1")?;
        (start, len, blob.hash)
    } else {
        (0, 0, [0u8; 32])
    };

    let footer = Ftr4 {
        version: 1,
        flags: 0,
        blocks_end_offset,
        dct_offset,
        dct_len,
        index_offset,
        index_len,
        ledger_offset,
        ledger_len,
        index_hash,
        ledger_hash,
        ..Default::default()
    }
    .finalize()?;

    footer.write_to(&mut frame)?;
    Ok(frame)
}

pub fn parse_tail_frame(frame_bytes: &[u8]) -> Result<TailFrameParts> {
    ensure!(
        frame_bytes.len() >= FTR4_LEN,
        "tail frame too short for FTR4"
    );

    let footer_start = frame_bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&frame_bytes[footer_start..])?;

    let base = footer.blocks_end_offset;
    let mut dct1 = None;

    if footer.dct_len > 0 {
        let rel = footer
            .dct_offset
            .checked_sub(base)
            .context("DCT1 offset before blocks_end_offset")?;
        let end = rel
            .checked_add(footer.dct_len)
            .context("DCT1 range overflow")?;
        ensure!(
            end <= footer.index_offset - base,
            "DCT1 overlaps IDX3 or exceeds layout"
        );

        let slice = component_slice(frame_bytes, rel, footer.dct_len)?;
        ensure!(slice.starts_with(&DCT1_MAGIC), "DCT1: bad magic");
        let table = read_dct1(Cursor::new(slice))?;
        dct1 = Some(table);
    }

    let idx_rel = footer
        .index_offset
        .checked_sub(base)
        .context("IDX3 offset before blocks_end_offset")?;
    let idx_end = idx_rel
        .checked_add(footer.index_len)
        .context("IDX3 range overflow")?;
    ensure!(
        idx_end <= footer_start as u64,
        "IDX3 range extends past footer"
    );

    let idx3_bytes = frame_bytes[idx_rel as usize..idx_end as usize].to_vec();
    ensure!(is_supported_index_magic(&idx3_bytes), "IDX: bad magic");
    ensure!(
        blake3::hash(&idx3_bytes).as_bytes() == &footer.index_hash,
        "IDX3 hash mismatch"
    );

    let ldg1 = if footer.ledger_len == 0 {
        None
    } else {
        let ldg_rel = footer
            .ledger_offset
            .checked_sub(base)
            .context("LDG1 offset before blocks_end_offset")?;
        let ldg_end = ldg_rel
            .checked_add(footer.ledger_len)
            .context("LDG1 range overflow")?;
        ensure!(
            ldg_end <= footer_start as u64,
            "LDG1 range extends past footer"
        );
        ensure!(idx_end <= ldg_rel, "IDX3 overlaps LDG1");

        let slice = component_slice(frame_bytes, ldg_rel, footer.ledger_len)?;
        ensure!(slice.starts_with(&LDG1_MAGIC), "LDG1: bad magic");
        let mut rdr = Cursor::new(slice);
        let mut blob = read_ldg1(&mut rdr)?;
        ensure!(
            blob.hash == footer.ledger_hash,
            "LDG1 hash mismatch vs footer"
        );

        // Ensure reader consumed exactly the component and there are no extra bytes in the slice.
        ensure!(
            rdr.position() as usize == slice.len(),
            "LDG1 trailing bytes"
        );

        // Preserve parsed blob.
        blob.hash = footer.ledger_hash;
        Some(blob)
    };

    if footer.ledger_len == 0 {
        ensure!(
            idx_end == footer_start as u64,
            "unexpected bytes between IDX3 and FTR4"
        );
    } else {
        let ldg_end = footer
            .ledger_offset
            .checked_sub(base)
            .and_then(|off| off.checked_add(footer.ledger_len))
            .context("LDG1 range overflow")?;
        ensure!(
            ldg_end == footer_start as u64,
            "unexpected bytes before FTR4"
        );
    }

    Ok(TailFrameParts {
        dct1,
        idx3_bytes,
        ldg1,
        footer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dct1::{Dct1Entry, Dct1Table};

    fn sample_idx3() -> Vec<u8> {
        b"IDX3\x01\x02\x03\x04".to_vec()
    }

    #[test]
    fn roundtrip_full_tail_frame() {
        let dct = Dct1Table::new(vec![Dct1Entry::new(7, b"dict".to_vec()).unwrap()]).unwrap();
        let ledger = LedgerBlob::from_value(&serde_json::json!({"z":1,"a":2})).unwrap();

        let frame = assemble_tail_frame(1024, Some(&dct), &sample_idx3(), Some(&ledger)).unwrap();
        let parsed = parse_tail_frame(&frame).unwrap();

        assert_eq!(parsed.dct1, Some(dct));
        assert_eq!(parsed.idx3_bytes, sample_idx3());
        assert_eq!(parsed.ldg1, Some(ledger));
        assert_eq!(parsed.footer.dct_offset, 1024);
        assert_eq!(parsed.footer.index_offset, 1024 + parsed.footer.dct_len);
        assert_eq!(
            parsed.footer.ledger_offset,
            parsed.footer.index_offset + parsed.footer.index_len
        );
    }

    #[test]
    fn roundtrip_without_optional_components() {
        let frame = assemble_tail_frame(4096, None, &sample_idx3(), None).unwrap();
        let parsed = parse_tail_frame(&frame).unwrap();

        assert!(parsed.dct1.is_none());
        assert!(parsed.ldg1.is_none());
        assert_eq!(parsed.footer.dct_offset, 0);
        assert_eq!(parsed.footer.dct_len, 0);
        assert_eq!(parsed.footer.ledger_offset, 0);
        assert_eq!(parsed.footer.ledger_len, 0);
    }

    #[test]
    fn rejects_footer_corruption() {
        let mut frame = assemble_tail_frame(1, None, &sample_idx3(), None).unwrap();
        *frame.last_mut().unwrap() ^= 0xFF;
        assert!(parse_tail_frame(&frame).is_err());
    }

    #[test]
    fn rejects_ledger_hash_mismatch() {
        let ledger = LedgerBlob::from_value(&serde_json::json!({"k":"v"})).unwrap();
        let mut frame = assemble_tail_frame(1, None, &sample_idx3(), Some(&ledger)).unwrap();

        // Corrupt one byte in the ledger payload bytes.
        let parsed = parse_tail_frame(&frame).unwrap();
        let l_off = (parsed.footer.ledger_offset - parsed.footer.blocks_end_offset) as usize;
        let payload_off = l_off + 4 + 8 + 32;
        frame[payload_off] ^= 0x01;

        assert!(parse_tail_frame(&frame).is_err());
    }
}
