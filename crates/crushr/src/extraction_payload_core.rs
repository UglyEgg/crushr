// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::format::Entry;
use anyhow::{Context, Result, bail};
use crushr_core::io::ReadAt;
use crushr_core::verify::BlockSpanV1;
use crushr_format::blk3::read_blk3_header;
use std::fs;
use std::io::Cursor;
use std::path::Path;

pub(crate) fn read_entry_bytes<R: ReadAt>(
    reader: &R,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<Vec<u8>> {
    let mut out = vec![0u8; entry.size as usize];
    let mut write_cursor = 0u64;

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;

        let raw = block_raw_payload(reader, block)?;

        let begin = extent.offset as usize;
        let end = begin
            .checked_add(extent.len as usize)
            .context("extent length overflow")?;
        if end > raw.len() {
            bail!(
                "extent out of range for block {} while reading {}",
                extent.block_id,
                entry.path
            );
        }

        let target_off = if entry.sparse {
            extent.logical_offset as usize
        } else {
            write_cursor as usize
        };
        let target_end = target_off
            .checked_add(extent.len as usize)
            .context("target extent overflow")?;
        if target_end > out.len() {
            bail!("entry size mismatch while reading {}", entry.path);
        }
        out[target_off..target_end].copy_from_slice(&raw[begin..end]);
        write_cursor = write_cursor
            .checked_add(extent.len)
            .context("entry size overflow while reading")?;
    }

    if !entry.sparse && write_cursor != entry.size {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(out)
}

pub(crate) fn validate_entry_bytes<R: ReadAt>(
    reader: &R,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<()> {
    let mut total = 0u64;
    let mut max_end = 0u64;

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;

        let raw = block_raw_payload(reader, block)?;
        let begin = extent.offset as usize;
        let end = begin
            .checked_add(extent.len as usize)
            .context("extent length overflow")?;
        if end > raw.len() {
            bail!(
                "extent out of range for block {} while reading {}",
                extent.block_id,
                entry.path
            );
        }
        total = total
            .checked_add(extent.len)
            .context("entry size overflow while validating extents")?;
        let logical_end = extent
            .logical_offset
            .checked_add(extent.len)
            .context("logical extent overflow while validating extents")?;
        max_end = max_end.max(logical_end);
    }

    if (!entry.sparse && total != entry.size) || (entry.sparse && max_end > entry.size) {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(())
}

pub(crate) fn recover_partial_entry_bytes<R: ReadAt>(
    reader: &R,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<Vec<u8>> {
    let mut out = if entry.sparse {
        vec![0u8; entry.size as usize]
    } else {
        Vec::new()
    };

    for extent in &entry.extents {
        let Some(block) = blocks.get(extent.block_id as usize) else {
            continue;
        };
        let Ok(raw) = block_raw_payload(reader, block) else {
            continue;
        };

        let begin = extent.offset as usize;
        let Some(end) = begin.checked_add(extent.len as usize) else {
            continue;
        };
        if end > raw.len() {
            continue;
        }

        if entry.sparse {
            let target_off = extent.logical_offset as usize;
            let Some(target_end) = target_off.checked_add(extent.len as usize) else {
                continue;
            };
            if target_end <= out.len() {
                out[target_off..target_end].copy_from_slice(&raw[begin..end]);
            }
        } else {
            out.extend_from_slice(&raw[begin..end]);
        }
    }

    Ok(out)
}

pub(crate) fn block_raw_payload<R: ReadAt>(reader: &R, block: &BlockSpanV1) -> Result<Vec<u8>> {
    let header_len = (block.payload_offset - block.header_offset) as usize;
    let mut header_bytes = vec![0u8; header_len];
    read_exact_at(reader, block.header_offset, &mut header_bytes)?;
    let header = read_blk3_header(Cursor::new(&header_bytes)).context("parse BLK3 header")?;

    if header.codec != 1 {
        bail!(
            "unsupported BLK3 codec {} for block {}",
            header.codec,
            block.block_id
        );
    }

    let mut payload = vec![0u8; block.comp_len as usize];
    read_exact_at(reader, block.payload_offset, &mut payload)?;

    let raw = zstd::decode_all(Cursor::new(payload)).context("decompress BLK3 payload")?;
    if raw.len() as u64 != header.raw_len {
        bail!("raw length mismatch for block {}", block.block_id);
    }

    Ok(raw)
}

pub(crate) fn write_entry_bytes(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() && !overwrite {
        bail!("destination exists (use --overwrite): {}", path.display());
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_sparse_entry<R: ReadAt>(
    reader: &R,
    entry: &Entry,
    path: &Path,
    blocks: &[BlockSpanV1],
    overwrite: bool,
) -> Result<()> {
    use std::os::unix::fs::FileExt;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if path.exists() && !overwrite {
        bail!("destination exists (use --overwrite): {}", path.display());
    }

    let out = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    out.set_len(entry.size)
        .with_context(|| format!("set_len {}", path.display()))?;

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;
        let raw = block_raw_payload(reader, block)?;
        let begin = extent.offset as usize;
        let end = begin
            .checked_add(extent.len as usize)
            .context("extent length overflow")?;
        if end > raw.len() {
            bail!("extent out of range for sparse write {}", entry.path);
        }
        out.write_at(&raw[begin..end], extent.logical_offset)
            .with_context(|| format!("write sparse extent {}", path.display()))?;
    }
    Ok(())
}

fn read_exact_at<R: ReadAt>(reader: &R, mut offset: u64, mut dst: &mut [u8]) -> Result<()> {
    while !dst.is_empty() {
        let n = reader.read_at(offset, dst)?;
        if n == 0 {
            bail!("unexpected EOF while reading archive");
        }
        let (_, rest) = dst.split_at_mut(n);
        dst = rest;
        offset += n as u64;
    }
    Ok(())
}
