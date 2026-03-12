use crate::format::{DCT_MAGIC_V1, FTR_MAGIC_V3};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FooterInfoV3 {
    pub blocks_end_offset: u64,
    pub dict_offset: u64,
    pub dict_len: u64,
    pub index_offset: u64,
    pub index_len: u64,
    pub index_hash: [u8; 32],
}

fn read_u64_le(mut r: impl Read) -> Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

fn parse_footer_at(f: &mut File, pos: u64) -> Result<FooterInfoV3> {
    // pos points to magic
    f.seek(SeekFrom::Start(pos))?;
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic)?;
    if &magic != FTR_MAGIC_V3 {
        bail!("not FTR3 at {}", pos);
    }
    let blocks_end_offset = read_u64_le(&mut *f)?;
    let dict_offset = read_u64_le(&mut *f)?;
    let dict_len = read_u64_le(&mut *f)?;
    let index_offset = read_u64_le(&mut *f)?;
    let index_len = read_u64_le(&mut *f)?;
    let mut index_hash = [0u8; 32];
    f.read_exact(&mut index_hash)?;
    Ok(FooterInfoV3 {
        blocks_end_offset,
        dict_offset,
        dict_len,
        index_offset,
        index_len,
        index_hash,
    })
}

fn validate_footer(f: &mut File, fi: &FooterInfoV3) -> Result<()> {
    let file_len = f.metadata()?.len();
    for (name, off, len) in [
        ("blocks_end_offset", 0u64, fi.blocks_end_offset),
        ("dict", fi.dict_offset, fi.dict_len),
        ("index", fi.index_offset, fi.index_len),
    ] {
        if off > file_len || off + len > file_len {
            bail!("{} region out of bounds", name);
        }
    }
    // verify dict table magic if present
    if fi.dict_len > 0 {
        f.seek(SeekFrom::Start(fi.dict_offset))?;
        let mut m = [0u8; 4];
        f.read_exact(&mut m)?;
        if &m != DCT_MAGIC_V1 {
            bail!("bad dict table magic");
        }
    }
    // verify index hash
    f.seek(SeekFrom::Start(fi.index_offset))?;
    let mut idx = vec![0u8; fi.index_len as usize];
    f.read_exact(&mut idx)?;
    let h = blake3::hash(&idx);
    if h.as_bytes() != &fi.index_hash {
        bail!("index hash mismatch");
    }
    Ok(())
}

/// Search the tail of the file for a valid FTR3 footer whose index hash verifies.
/// This recovers from tail corruption where the last N bytes are damaged or truncated.
pub fn find_latest_valid_footer(path: &Path, tail_scan_bytes: u64) -> Result<FooterInfoV3> {
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let len = f.metadata()?.len();
    let scan = tail_scan_bytes.min(len);
    let start = len - scan;

    f.seek(SeekFrom::Start(start))?;
    let mut buf = vec![0u8; scan as usize];
    f.read_exact(&mut buf)?;

    let mut best: Option<FooterInfoV3> = None;
    for i in (0..buf.len().saturating_sub(4)).rev() {
        if &buf[i..i + 4] != FTR_MAGIC_V3 {
            continue;
        }
        let _pos = start + i as u64;
        if let Ok(fi) = parse_footer_at(&mut f, _pos) {
            if validate_footer(&mut f, &fi).is_ok() {
                best = Some(fi);
                break; // scanning from end, first valid is latest
            }
        }
    }
    best.ok_or_else(|| anyhow::anyhow!("no valid footer found in last {} bytes", scan))
}

/// Write a repaired archive by copying the blocks region from the damaged file,
/// then rewriting dict table + index + fresh footer(s).
pub fn repair_archive(input: &Path, output: &Path, tail_scan_bytes: u64) -> Result<()> {
    let fi = find_latest_valid_footer(input, tail_scan_bytes)?;
    let mut src = File::open(input)?;
    let mut out = File::create(output).with_context(|| format!("create {}", output.display()))?;

    // Copy blocks region verbatim.
    src.seek(SeekFrom::Start(0))?;
    let mut remaining = fi.blocks_end_offset;
    let mut buf = vec![0u8; 1024 * 1024];
    while remaining > 0 {
        let n = (buf.len() as u64).min(remaining) as usize;
        src.read_exact(&mut buf[..n])?;
        out.write_all(&buf[..n])?;
        remaining -= n as u64;
    }

    // Copy dict table bytes (if any).
    let (dict_offset, dict_len) = if fi.dict_len > 0 {
        let dict_offset = out.stream_position()?;
        src.seek(SeekFrom::Start(fi.dict_offset))?;
        let mut db = vec![0u8; fi.dict_len as usize];
        src.read_exact(&mut db)?;
        out.write_all(&db)?;
        (dict_offset, fi.dict_len)
    } else {
        (0u64, 0u64)
    };

    // Copy index bytes (verified in find_latest_valid_footer).
    let index_offset = out.stream_position()?;
    src.seek(SeekFrom::Start(fi.index_offset))?;
    let mut idx = vec![0u8; fi.index_len as usize];
    src.read_exact(&mut idx)?;
    out.write_all(&idx)?;
    let index_len = fi.index_len;

    // Compute index hash for fresh footer.
    let h = blake3::hash(&idx);
    let mut hb = [0u8; 32];
    hb.copy_from_slice(h.as_bytes());

    // Write fresh primary footer (FTR3)
    out.write_all(FTR_MAGIC_V3)?;
    out.write_all(&fi.blocks_end_offset.to_le_bytes())?;
    out.write_all(&dict_offset.to_le_bytes())?;
    out.write_all(&dict_len.to_le_bytes())?;
    out.write_all(&index_offset.to_le_bytes())?;
    out.write_all(&index_len.to_le_bytes())?;
    out.write_all(&hb)?;

    // Write backup index + backup footer to tolerate small tail corruption.
    let backup_index_offset = out.stream_position()?;
    out.write_all(&idx)?;
    let backup_index_len = idx.len() as u64;
    let backup_hash = hb;
    out.write_all(FTR_MAGIC_V3)?;
    out.write_all(&fi.blocks_end_offset.to_le_bytes())?;
    out.write_all(&dict_offset.to_le_bytes())?;
    out.write_all(&dict_len.to_le_bytes())?;
    out.write_all(&backup_index_offset.to_le_bytes())?;
    out.write_all(&backup_index_len.to_le_bytes())?;
    out.write_all(&backup_hash)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct BlockInfo {
    pub block_id: u32,
    pub uncomp_len: u32,
}
fn read_u32_le(mut r: impl Read) -> Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

#[allow(clippy::type_complexity)]
fn scan_frames_for_salvage(
    path: &Path,
) -> Result<(
    u64,
    Vec<BlockInfo>,
    Vec<crate::format::Entry>,
    Option<Vec<u8>>,
)> {
    use crate::format::{BLK_MAGIC, BLK_MAGIC_V2, CODEC_ZSTD, EVT_MAGIC_V1};
    let mut f = File::open(path)?;
    let mut blocks: Vec<BlockInfo> = Vec::new();
    let mut entries: Vec<crate::format::Entry> = Vec::new();
    let mut dict_table: Option<Vec<u8>> = None;

    let mut _pos = 0u64;
    let mut next_block_id: u32 = 0;

    loop {
        let mut magic = [0u8; 4];
        if f.read_exact(&mut magic).is_err() {
            break;
        }
        _pos += 4;

        if &magic == BLK_MAGIC || &magic == BLK_MAGIC_V2 {
            if &magic == BLK_MAGIC_V2 {
                let _ = read_u32_le(&mut f)?;
            }
            let codec = read_u32_le(&mut f)?;
            if codec != CODEC_ZSTD {
                bail!("unsupported codec {} during salvage", codec);
            }
            let comp_len = read_u32_le(&mut f)?;
            let uncomp_len = read_u32_le(&mut f)?;
            // comp bytes
            f.seek(SeekFrom::Current(comp_len as i64))?;
            // hash
            f.seek(SeekFrom::Current(32))?;
            let block_end = f.stream_position()?;

            blocks.push(BlockInfo {
                block_id: next_block_id,
                uncomp_len,
            });
            next_block_id += 1;
            _pos = block_end;
            continue;
        }

        if &magic == EVT_MAGIC_V1 {
            let kind = read_u32_le(&mut f)?;
            let plen = read_u32_le(&mut f)? as usize;
            let mut h = [0u8; 32];
            f.read_exact(&mut h)?;
            let mut payload = vec![0u8; plen];
            f.read_exact(&mut payload)?;
            let hh = blake3::hash(&payload);
            if hh.as_bytes() != &h {
                // corrupted EVT; stop at first unknown/corrupt region
                break;
            }

            if kind == 2 {
                // dict table bytes (DCT1...)
                dict_table = Some(payload);
            } else if kind == 1 {
                // file events
                let mut off = 0usize;
                if plen < 4 {
                    continue;
                }
                let cnt = u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize;
                off += 4;
                for _ in 0..cnt {
                    if off + 4 + 4 + 1 + 4 + 8 + 8 + 4 > plen {
                        break;
                    }
                    let block_id = u32::from_le_bytes(payload[off..off + 4].try_into().unwrap());
                    off += 4;
                    let intra = u32::from_le_bytes(payload[off..off + 4].try_into().unwrap());
                    off += 4;
                    let fkind = payload[off];
                    off += 1;
                    let mode = u32::from_le_bytes(payload[off..off + 4].try_into().unwrap());
                    off += 4;
                    let mtime = i64::from_le_bytes(payload[off..off + 8].try_into().unwrap());
                    off += 8;
                    let size = u64::from_le_bytes(payload[off..off + 8].try_into().unwrap());
                    off += 8;

                    let plen_path =
                        u32::from_le_bytes(payload[off..off + 4].try_into().unwrap()) as usize;
                    off += 4;
                    if off + plen_path > plen {
                        break;
                    }
                    let path_s = std::str::from_utf8(&payload[off..off + plen_path])
                        .unwrap_or("")
                        .to_string();
                    off += plen_path;

                    let plen_link =
                        u32::from_le_bytes(payload[off..off + 4].try_into().unwrap()) as usize;
                    off += 4;
                    if off + plen_link > plen {
                        break;
                    }
                    let link_s = std::str::from_utf8(&payload[off..off + plen_link])
                        .unwrap_or("")
                        .to_string();
                    off += plen_link;

                    // Store as Entry with extents filled later by rebuild.
                    let kind_enum = if fkind == 1 {
                        crate::format::EntryKind::Symlink
                    } else {
                        crate::format::EntryKind::Regular
                    };
                    let e = crate::format::Entry {
                        path: path_s,
                        kind: kind_enum,
                        mode,
                        mtime,
                        size,
                        extents: vec![crate::format::Extent {
                            block_id,
                            offset: intra as u64,
                            len: size,
                        }], // placeholder; expanded later
                        link_target: if fkind == 1 { Some(link_s) } else { None },
                        xattrs: Vec::new(),
                    };
                    entries.push(e);
                }
            }
            _pos = f.stream_position()?;
            continue;
        }

        // Unknown frame => stop; treat remaining as damaged metadata/tail.
        break;
    }

    let blocks_end = f.stream_position()?;
    Ok((blocks_end, blocks, entries, dict_table))
}

fn expand_extents(
    blocks: &[BlockInfo],
    mut entries: Vec<crate::format::Entry>,
) -> Result<Vec<crate::format::Entry>> {
    // Convert placeholder extents (start block + intra + size) into real extents spanning blocks.
    let mut blk_uncomp: Vec<u32> = vec![0; blocks.len()];
    for b in blocks {
        if (b.block_id as usize) < blk_uncomp.len() {
            blk_uncomp[b.block_id as usize] = b.uncomp_len;
        }
    }

    for e in entries.iter_mut() {
        if e.kind == crate::format::EntryKind::Symlink {
            e.extents.clear();
            e.size = 0;
            continue;
        }
        if e.extents.is_empty() {
            continue;
        }
        let start = e.extents[0].clone();
        let mut remaining = e.size;
        let mut bid = start.block_id;
        let mut intra = start.offset;

        e.extents.clear();
        while remaining > 0 {
            let ulen = *blk_uncomp.get(bid as usize).unwrap_or(&0) as u64;
            if ulen == 0 {
                bail!("missing block {} during extent expansion", bid);
            }
            if intra >= ulen {
                bail!("intra offset out of range");
            }
            let avail = ulen - intra;
            let take = avail.min(remaining);
            e.extents.push(crate::format::Extent {
                block_id: bid,
                offset: intra,
                len: take,
            });
            remaining -= take;
            bid += 1;
            intra = 0;
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Salvage a damaged archive even if the footer/index are unrecoverable, using embedded EVT frames.
/// This rewrites a new archive with a rebuilt index and fresh tail frames.
pub fn salvage_archive(input: &Path, output: &Path) -> Result<()> {
    let (blocks_end_offset, blocks, entries_raw, dict_table_evt) = scan_frames_for_salvage(input)?;
    let entries = expand_extents(&blocks, entries_raw)?;

    // Build index
    let idx = crate::format::Index { entries };
    let index_bytes = crate::index_codec::encode_index(&idx);
    let index_hash = blake3::hash(&index_bytes);
    let mut index_hash_bytes = [0u8; 32];
    index_hash_bytes.copy_from_slice(index_hash.as_bytes());

    // Copy blocks region (including EVT frames) verbatim
    let mut src = File::open(input)?;
    let mut out = File::create(output)?;
    src.seek(SeekFrom::Start(0))?;
    let mut remaining = blocks_end_offset;
    let mut buf = vec![0u8; 1024 * 1024];
    while remaining > 0 {
        let n = (buf.len() as u64).min(remaining) as usize;
        src.read_exact(&mut buf[..n])?;
        out.write_all(&buf[..n])?;
        remaining -= n as u64;
    }

    // Write dict table region if present (prefer EVT copy, else none)
    let (dict_offset, dict_len) = if let Some(dt) = dict_table_evt {
        let dict_offset = out.stream_position()?;
        out.write_all(&dt)?;
        (dict_offset, dt.len() as u64)
    } else {
        (0u64, 0u64)
    };

    // Write index
    let index_offset = out.stream_position()?;
    out.write_all(&index_bytes)?;
    let index_len = index_bytes.len() as u64;

    // Write primary footer
    out.write_all(FTR_MAGIC_V3)?;
    out.write_all(&blocks_end_offset.to_le_bytes())?;
    out.write_all(&dict_offset.to_le_bytes())?;
    out.write_all(&dict_len.to_le_bytes())?;
    out.write_all(&index_offset.to_le_bytes())?;
    out.write_all(&index_len.to_le_bytes())?;
    out.write_all(&index_hash_bytes)?;

    // Tail redundancy frames (2 extra copies)
    let pad = 4096usize;
    for _ in 0..2 {
        out.write_all(&vec![0u8; pad])?;
        let copy_index_offset = out.stream_position()?;
        out.write_all(&index_bytes)?;
        let copy_index_len = index_bytes.len() as u64;
        out.write_all(FTR_MAGIC_V3)?;
        out.write_all(&blocks_end_offset.to_le_bytes())?;
        out.write_all(&dict_offset.to_le_bytes())?;
        out.write_all(&dict_len.to_le_bytes())?;
        out.write_all(&copy_index_offset.to_le_bytes())?;
        out.write_all(&copy_index_len.to_le_bytes())?;
        out.write_all(&index_hash_bytes)?;
    }

    Ok(())
}
