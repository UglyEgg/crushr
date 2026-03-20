// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::format::{
    Entry, EntryKind, Extent, Index, Xattr, IDX_MAGIC_V1, IDX_MAGIC_V2, IDX_MAGIC_V3,
};
use anyhow::{bail, Result};

fn put_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn get_u8(input: &[u8], off: &mut usize) -> Result<u8> {
    if *off + 1 > input.len() {
        bail!("truncated index");
    }
    let v = input[*off];
    *off += 1;
    Ok(v)
}
fn get_u32(input: &[u8], off: &mut usize) -> Result<u32> {
    if *off + 4 > input.len() {
        bail!("truncated index");
    }
    let v = u32::from_le_bytes(input[*off..*off + 4].try_into().unwrap());
    *off += 4;
    Ok(v)
}
fn get_u64(input: &[u8], off: &mut usize) -> Result<u64> {
    if *off + 8 > input.len() {
        bail!("truncated index");
    }
    let v = u64::from_le_bytes(input[*off..*off + 8].try_into().unwrap());
    *off += 8;
    Ok(v)
}
fn get_i64(input: &[u8], off: &mut usize) -> Result<i64> {
    if *off + 8 > input.len() {
        bail!("truncated index");
    }
    let v = i64::from_le_bytes(input[*off..*off + 8].try_into().unwrap());
    *off += 8;
    Ok(v)
}

fn put_len_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_u32(out, b.len() as u32);
    out.extend_from_slice(b);
}
fn get_len_bytes<'a>(input: &'a [u8], off: &mut usize) -> Result<&'a [u8]> {
    let n = get_u32(input, off)? as usize;
    if *off + n > input.len() {
        bail!("truncated bytes");
    }
    let s = &input[*off..*off + n];
    *off += n;
    Ok(s)
}

pub fn encode_index(idx: &Index) -> Vec<u8> {
    // Current stable encoding (IDX3) includes xattrs.
    // magic(4) entry_count(u32)
    // entries:
    //   path (len+bytes)
    //   kind(u8) 0=regular,1=symlink
    //   mode(u32) mtime(i64) size(u64)
    //   extent_count(u32) + extents
    //   link_target (len+bytes; 0 for regular)
    //   xattr_count(u32) + xattrs: name(len+bytes) value(len+bytes)
    let mut out = Vec::new();
    out.extend_from_slice(IDX_MAGIC_V3);
    put_u32(&mut out, idx.entries.len() as u32);

    for e in &idx.entries {
        put_len_bytes(&mut out, e.path.as_bytes());

        let kind = match e.kind {
            EntryKind::Regular => 0u8,
            EntryKind::Symlink => 1u8,
        };
        put_u8(&mut out, kind);

        put_u32(&mut out, e.mode);
        put_i64(&mut out, e.mtime);
        put_u64(&mut out, e.size);

        put_u32(&mut out, e.extents.len() as u32);
        for ex in &e.extents {
            put_u32(&mut out, ex.block_id);
            put_u64(&mut out, ex.offset);
            put_u64(&mut out, ex.len);
        }

        match e.kind {
            EntryKind::Regular => put_u32(&mut out, 0),
            EntryKind::Symlink => {
                let t = e.link_target.as_deref().unwrap_or("");
                put_len_bytes(&mut out, t.as_bytes());
            }
        }

        put_u32(&mut out, e.xattrs.len() as u32);
        for xa in &e.xattrs {
            put_len_bytes(&mut out, xa.name.as_bytes());
            put_len_bytes(&mut out, &xa.value);
        }
    }

    out
}

pub fn decode_index(bytes: &[u8]) -> Result<Index> {
    if bytes.len() < 8 {
        bail!("index too small");
    }
    let magic = &bytes[0..4];

    if magic == IDX_MAGIC_V3 {
        decode_idx3(bytes)
    } else if magic == IDX_MAGIC_V2 {
        decode_idx2(bytes)
    } else if magic == IDX_MAGIC_V1 {
        decode_idx1(bytes)
    } else {
        bail!("bad index magic");
    }
}

fn decode_idx3(bytes: &[u8]) -> Result<Index> {
    let mut off = 4usize;
    let count = get_u32(bytes, &mut off)? as usize;
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        let path = std::str::from_utf8(get_len_bytes(bytes, &mut off)?)?.to_string();
        let kind_u8 = get_u8(bytes, &mut off)?;
        let kind = match kind_u8 {
            0 => EntryKind::Regular,
            1 => EntryKind::Symlink,
            _ => bail!("unknown entry kind {}", kind_u8),
        };

        let mode = get_u32(bytes, &mut off)?;
        let mtime = get_i64(bytes, &mut off)?;
        let size = get_u64(bytes, &mut off)?;
        let ex_count = get_u32(bytes, &mut off)? as usize;

        let mut extents = Vec::with_capacity(ex_count);
        for _ in 0..ex_count {
            let block_id = get_u32(bytes, &mut off)?;
            let offset = get_u64(bytes, &mut off)?;
            let len = get_u64(bytes, &mut off)?;
            extents.push(Extent {
                block_id,
                offset,
                len,
            });
        }

        let link_target_bytes = get_len_bytes(bytes, &mut off)?;
        let link_target = if link_target_bytes.is_empty() {
            None
        } else {
            Some(std::str::from_utf8(link_target_bytes)?.to_string())
        };

        let xa_count = get_u32(bytes, &mut off)? as usize;
        let mut xattrs = Vec::with_capacity(xa_count);
        for _ in 0..xa_count {
            let name = std::str::from_utf8(get_len_bytes(bytes, &mut off)?)?.to_string();
            let val = get_len_bytes(bytes, &mut off)?.to_vec();
            xattrs.push(Xattr { name, value: val });
        }

        if kind == EntryKind::Symlink && !extents.is_empty() {
            bail!("symlink entry has extents");
        }

        entries.push(Entry {
            path,
            kind,
            mode,
            mtime,
            size,
            extents,
            link_target,
            xattrs,
        });
    }

    if off != bytes.len() {
        bail!("index has trailing bytes");
    }

    Ok(Index { entries })
}

fn decode_idx2(bytes: &[u8]) -> Result<Index> {
    // IDX2: kind + link_target, no xattrs
    let mut off = 4usize;
    let count = get_u32(bytes, &mut off)? as usize;
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        let path = std::str::from_utf8(get_len_bytes(bytes, &mut off)?)?.to_string();
        let kind_u8 = get_u8(bytes, &mut off)?;
        let kind = match kind_u8 {
            0 => EntryKind::Regular,
            1 => EntryKind::Symlink,
            _ => bail!("unknown entry kind {}", kind_u8),
        };

        let mode = get_u32(bytes, &mut off)?;
        let mtime = get_i64(bytes, &mut off)?;
        let size = get_u64(bytes, &mut off)?;
        let ex_count = get_u32(bytes, &mut off)? as usize;

        let mut extents = Vec::with_capacity(ex_count);
        for _ in 0..ex_count {
            let block_id = get_u32(bytes, &mut off)?;
            let offset = get_u64(bytes, &mut off)?;
            let len = get_u64(bytes, &mut off)?;
            extents.push(Extent {
                block_id,
                offset,
                len,
            });
        }

        let link_target_bytes = get_len_bytes(bytes, &mut off)?;
        let link_target = if link_target_bytes.is_empty() {
            None
        } else {
            Some(std::str::from_utf8(link_target_bytes)?.to_string())
        };

        if kind == EntryKind::Symlink && !extents.is_empty() {
            bail!("symlink entry has extents");
        }

        entries.push(Entry {
            path,
            kind,
            mode,
            mtime,
            size,
            extents,
            link_target,
            xattrs: Vec::new(),
        });
    }

    if off != bytes.len() {
        bail!("index has trailing bytes");
    }

    Ok(Index { entries })
}

fn decode_idx1(bytes: &[u8]) -> Result<Index> {
    // IDX1: regular only, no link_target, no xattrs.
    let mut off = 4usize;
    let count = get_u32(bytes, &mut off)? as usize;
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        let path = std::str::from_utf8(get_len_bytes(bytes, &mut off)?)?.to_string();
        let mode = get_u32(bytes, &mut off)?;
        let mtime = get_i64(bytes, &mut off)?;
        let size = get_u64(bytes, &mut off)?;
        let ex_count = get_u32(bytes, &mut off)? as usize;

        let mut extents = Vec::with_capacity(ex_count);
        for _ in 0..ex_count {
            let block_id = get_u32(bytes, &mut off)?;
            let offset = get_u64(bytes, &mut off)?;
            let len = get_u64(bytes, &mut off)?;
            extents.push(Extent {
                block_id,
                offset,
                len,
            });
        }

        entries.push(Entry {
            path,
            kind: EntryKind::Regular,
            mode,
            mtime,
            size,
            extents,
            link_target: None,
            xattrs: Vec::new(),
        });
    }

    if off != bytes.len() {
        bail!("index has trailing bytes");
    }

    Ok(Index { entries })
}
