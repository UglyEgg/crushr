// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::io::{Len, ReadAt};
use anyhow::{ensure, Context, Result};
use crushr_format::blk3::{read_blk3_header, BLK3_MAGIC};
use std::collections::BTreeSet;
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpanV1 {
    pub block_id: u32,
    pub header_offset: u64,
    pub payload_offset: u64,
    pub comp_len: u64,
    pub payload_hash: Option<[u8; 32]>,
}

pub fn scan_blocks_v1<R: ReadAt + Len>(
    reader: &R,
    blocks_end_offset: u64,
) -> Result<Vec<BlockSpanV1>> {
    let archive_len = reader.len().context("read archive length")?;
    ensure!(
        blocks_end_offset <= archive_len,
        "blocks_end_offset beyond archive length"
    );

    let mut blocks = Vec::new();
    let mut offset = 0u64;
    let mut block_id = 0u32;

    while offset < blocks_end_offset {
        let remaining = blocks_end_offset - offset;
        ensure!(remaining >= 4, "short block region: missing BLK3 magic");

        let mut magic = [0u8; 4];
        read_exact_at(reader, offset, &mut magic).context("read BLK3 magic")?;
        ensure!(magic == BLK3_MAGIC, "invalid BLK3 magic at offset {offset}");

        let mut header_prefix = vec![0u8; 6];
        read_exact_at(reader, offset, &mut header_prefix).context("read BLK3 header prefix")?;
        let header_len = u16::from_le_bytes([header_prefix[4], header_prefix[5]]) as u64;

        ensure!(
            header_len <= remaining,
            "BLK3 header exceeds blocks region at offset {offset}"
        );

        let mut header_bytes = vec![0u8; header_len as usize];
        read_exact_at(reader, offset, &mut header_bytes).context("read BLK3 header")?;
        let header = read_blk3_header(Cursor::new(&header_bytes)).context("parse BLK3 header")?;

        let payload_offset = offset
            .checked_add(header.header_len as u64)
            .context("payload offset overflow")?;
        let block_end = payload_offset
            .checked_add(header.comp_len)
            .context("block end overflow")?;

        ensure!(
            block_end <= blocks_end_offset,
            "BLK3 payload exceeds blocks region at offset {offset}"
        );

        blocks.push(BlockSpanV1 {
            block_id,
            header_offset: offset,
            payload_offset,
            comp_len: header.comp_len,
            payload_hash: header.payload_hash,
        });

        offset = block_end;
        block_id = block_id.checked_add(1).context("too many blocks")?;
    }

    Ok(blocks)
}

pub fn verify_block_payloads_v1<R: ReadAt + Len>(
    reader: &R,
    blocks_end_offset: u64,
) -> Result<BTreeSet<u32>> {
    let blocks = scan_blocks_v1(reader, blocks_end_offset)?;
    let mut corrupted = BTreeSet::new();

    for block in blocks {
        let Some(expected_hash) = block.payload_hash else {
            continue;
        };

        let mut hasher = blake3::Hasher::new();
        let mut scratch = vec![0u8; 64 * 1024];
        let mut read_offset = block.payload_offset;
        let mut remaining = block.comp_len;

        while remaining > 0 {
            let chunk = (remaining as usize).min(scratch.len());
            let n = read_exact_at(reader, read_offset, &mut scratch[..chunk])
                .context("read block payload")?;
            hasher.update(&scratch[..n]);
            read_offset = read_offset
                .checked_add(n as u64)
                .context("payload read offset overflow")?;
            remaining -= n as u64;
        }

        if *hasher.finalize().as_bytes() != expected_hash {
            corrupted.insert(block.block_id);
        }
    }

    Ok(corrupted)
}

fn read_exact_at<R: ReadAt>(reader: &R, offset: u64, dst: &mut [u8]) -> Result<usize> {
    let mut remaining = dst;
    let mut off = offset;
    let mut total = 0usize;

    while !remaining.is_empty() {
        let n = reader.read_at(off, remaining)?;
        ensure!(n != 0, "unexpected EOF while reading archive");

        let (_, rest) = remaining.split_at_mut(n);
        remaining = rest;
        off = off
            .checked_add(n as u64)
            .context("offset overflow while reading archive")?;
        total += n;
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{Len, ReadAt};
    use anyhow::Result;
    use crushr_format::{
        blk3::{write_blk3_header, Blk3Flags, Blk3Header},
        tailframe::assemble_tail_frame,
    };

    #[derive(Clone)]
    struct MemReader {
        bytes: Vec<u8>,
    }

    impl ReadAt for MemReader {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
            let offset = offset as usize;
            if offset >= self.bytes.len() {
                return Ok(0);
            }
            let n = (self.bytes.len() - offset).min(buf.len());
            buf[..n].copy_from_slice(&self.bytes[offset..offset + n]);
            Ok(n)
        }
    }

    impl Len for MemReader {
        fn len(&self) -> Result<u64> {
            Ok(self.bytes.len() as u64)
        }
    }

    fn build_block(payload: &[u8], with_hash: bool) -> Vec<u8> {
        let mut flags = Blk3Flags(0);
        if with_hash {
            flags.0 |= Blk3Flags::HAS_PAYLOAD_HASH;
        }
        let header = Blk3Header {
            header_len: if with_hash { 68 } else { 36 },
            flags,
            codec: 1,
            level: 3,
            dict_id: 0,
            raw_len: payload.len() as u64,
            comp_len: payload.len() as u64,
            payload_hash: with_hash.then(|| *blake3::hash(payload).as_bytes()),
            raw_hash: None,
        };

        let mut out = Vec::new();
        write_blk3_header(&mut out, &header).unwrap();
        out.extend_from_slice(payload);
        out
    }

    fn build_archive_with_blocks(blocks: &[Vec<u8>]) -> (MemReader, u64) {
        let mut bytes = Vec::new();
        for b in blocks {
            bytes.extend_from_slice(b);
        }
        let blocks_end = bytes.len() as u64;
        let tail = assemble_tail_frame(blocks_end, None, b"IDX3\x01", None).unwrap();
        bytes.extend_from_slice(&tail);
        (MemReader { bytes }, blocks_end)
    }

    #[test]
    fn clean_payload_hashes_report_zero_corruptions() {
        let b0 = build_block(b"payload-0", true);
        let b1 = build_block(b"payload-1", true);
        let (reader, blocks_end) = build_archive_with_blocks(&[b0, b1]);

        let corrupted = verify_block_payloads_v1(&reader, blocks_end).unwrap();
        assert!(corrupted.is_empty());
    }

    #[test]
    fn corrupted_payload_byte_reports_block_id() {
        let b0 = build_block(b"payload-0", true);
        let b1 = build_block(b"payload-1", true);
        let (mut reader, blocks_end) = build_archive_with_blocks(&[b0.clone(), b1]);

        let payload_offset = b0.len() + 68;
        reader.bytes[payload_offset] ^= 0x01;

        let corrupted = verify_block_payloads_v1(&reader, blocks_end).unwrap();
        assert_eq!(corrupted.into_iter().collect::<Vec<_>>(), vec![1]);
    }
}
