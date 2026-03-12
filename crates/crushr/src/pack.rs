use crate::dict;
use crate::format::{
    classify_group, Entry, EntryKind, Extent, Index, Xattr, BLK_MAGIC_V2, CODEC_ZSTD, FTR_MAGIC_V2,
};
use crate::index_codec::encode_index;
use crate::progress::{ProgressEvent, ProgressOp, ProgressPhase, SharedSink};

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct InputItem {
    rel: String,
    src: PathBuf,
    is_symlink: bool,
}

fn to_rel(base: &Path, p: &Path) -> Result<String> {
    // Paths coming from WalkDir may be relative while `base` is canonical/absolute.
    // Be tolerant: attempt strip_prefix on the provided path, then on its canonical form.
    let rel = match p.strip_prefix(base) {
        Ok(r) => r.to_path_buf(),
        Err(_) => {
            let cp = std::fs::canonicalize(p)
                .with_context(|| format!("canonicalize {}", p.display()))?;
            cp.strip_prefix(base)
                .with_context(|| {
                    format!("path {} is not under base {}", cp.display(), base.display())
                })?
                .to_path_buf()
        }
    };
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn collect_input_entries(inputs: &[PathBuf], base: &Path) -> Result<Vec<InputItem>> {
    let mut out = Vec::new();
    for inp in inputs {
        let meta =
            std::fs::symlink_metadata(inp).with_context(|| format!("stat {}", inp.display()))?;

        if meta.is_dir() {
            for e in walkdir::WalkDir::new(inp).follow_links(false) {
                let e = e?;
                let ft = e.file_type();
                if !(ft.is_file() || ft.is_symlink()) {
                    continue;
                }
                let rel = to_rel(base, e.path())?;
                out.push(InputItem {
                    rel,
                    src: e.path().to_path_buf(),
                    is_symlink: ft.is_symlink(),
                });
            }
        } else if meta.is_file() || meta.file_type().is_symlink() {
            let rel = to_rel(base, inp)?;
            out.push(InputItem {
                rel,
                src: inp.clone(),
                is_symlink: meta.file_type().is_symlink(),
            });
        }
    }
    // Deterministic ordering.
    out.sort_by(|a, b| {
        classify_group(&a.rel)
            .cmp(&classify_group(&b.rel))
            .then_with(|| a.rel.cmp(&b.rel))
    });
    Ok(out)
}

fn write_u32_le(mut w: impl Write, v: u32) -> Result<()> {
    w.write_all(&v.to_le_bytes()).map_err(Into::into)
}
fn write_u64_le(mut w: impl Write, v: u64) -> Result<()> {
    w.write_all(&v.to_le_bytes()).map_err(Into::into)
}

#[allow(dead_code)]
fn read_u64_le(mut r: impl Read) -> Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

fn flush_block(
    out: &mut File,
    raw: &mut Vec<u8>,
    level: i32,
    dict_bytes: Option<&[u8]>,
) -> Result<()> {
    if raw.is_empty() {
        return Ok(());
    }
    let uncompressed_size = raw.len() as u64;

    let mut encoder = if let Some(d) = dict_bytes {
        zstd::Encoder::with_dictionary(Vec::new(), level, d)?
    } else {
        zstd::Encoder::new(Vec::new(), level)?
    };
    encoder.write_all(raw)?;
    let compressed = encoder.finish()?;
    let compressed_size = compressed.len() as u64;

    // Frame header:
    // magic (4) + codec (4) + raw_len (8) + comp_len (8)
    out.write_all(BLK_MAGIC_V2)?;
    write_u32_le(&mut *out, CODEC_ZSTD)?;
    write_u64_le(&mut *out, uncompressed_size)?;
    write_u64_le(&mut *out, compressed_size)?;
    out.write_all(&compressed)?;

    raw.clear();
    Ok(())
}

fn capture_xattrs(path: &Path) -> Result<Vec<Xattr>> {
    // On non-linux (or if disabled), this should be empty; callers decide policy.
    let mut out = Vec::new();
    let it = match xattr::list(path) {
        Ok(v) => v,
        Err(_) => return Ok(out),
    };
    for name_os in it {
        let name = name_os.to_string_lossy().to_string();
        if let Ok(Some(val)) = xattr::get(path, &name_os) {
            out.push(Xattr { name, value: val });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Packs a set of input paths (files and/or directories) into a crushr archive.
#[allow(clippy::too_many_arguments)]
pub fn pack_paths_with_dict_with_xattrs_progress(
    inputs: &[PathBuf],
    base: &Path,
    output: &Path,
    block_size: u64,
    level: i32,
    sink: SharedSink,
    dict_bytes: Option<&[u8]>,
    xattr_policy: &str,
) -> Result<()> {
    if block_size < 1024 * 1024 {
        bail!("block_size too small; use >= 1 MiB");
    }

    let items = collect_input_entries(inputs, base)?;
    if items.is_empty() {
        bail!("no input files");
    }

    let mut total_bytes: u64 = 0;
    for it in &items {
        if it.is_symlink {
            continue;
        }
        if let Ok(md) = std::fs::metadata(&it.src) {
            total_bytes = total_bytes.saturating_add(md.len());
        }
    }
    sink.on_event(ProgressEvent::Start {
        op: ProgressOp::Pack,
        phase: ProgressPhase::ScanInputs,
        total_bytes,
    });
    sink.on_event(ProgressEvent::Phase {
        phase: ProgressPhase::Compress,
        total_bytes: Some(total_bytes),
    });

    let mut out = File::create(output).with_context(|| format!("create {}", output.display()))?;
    let mut index = Index {
        entries: Vec::new(),
    };

    let mut cur_block_raw: Vec<u8> = Vec::with_capacity(block_size as usize);
    let mut block_id: u32 = 0;

    for it in items {
        if it.is_symlink {
            // Preserve symlink target (best effort)
            let target = std::fs::read_link(&it.src)
                .with_context(|| format!("readlink {}", it.src.display()))?
                .to_string_lossy()
                .to_string();

            index.entries.push(Entry {
                path: it.rel,
                kind: EntryKind::Symlink,
                mode: 0,
                mtime: 0,
                size: 0,
                extents: Vec::new(),
                link_target: Some(target),
                xattrs: Vec::new(),
            });
            continue;
        }

        let md =
            std::fs::metadata(&it.src).with_context(|| format!("stat {}", it.src.display()))?;
        let mode = 0u32; // TODO: preserve unix mode via std::os::unix::fs::MetadataExt when desired.
        let mtime = 0i64;

        let mut f = File::open(&it.src).with_context(|| format!("open {}", it.src.display()))?;
        let mut remaining = md.len();
        let mut extents: Vec<Extent> = Vec::new();

        while remaining > 0 {
            let room = (block_size as usize).saturating_sub(cur_block_raw.len());
            if room == 0 {
                flush_block(&mut out, &mut cur_block_raw, level, dict_bytes)?;
                block_id += 1;
                continue;
            }
            let want = std::cmp::min(room as u64, remaining) as usize;
            let off_in_block = cur_block_raw.len() as u64;

            let start_len = cur_block_raw.len();
            cur_block_raw.resize(start_len + want, 0u8);
            f.read_exact(&mut cur_block_raw[start_len..start_len + want])?;
            sink.on_event(ProgressEvent::AdvanceBytes { bytes: want as u64 });

            extents.push(Extent {
                block_id,
                offset: off_in_block,
                len: want as u64,
            });
            remaining -= want as u64;

            if cur_block_raw.len() as u64 >= block_size {
                flush_block(&mut out, &mut cur_block_raw, level, dict_bytes)?;
                block_id += 1;
            }
        }

        let xattrs = match xattr_policy {
            "none" => Vec::new(),
            "store" => capture_xattrs(&it.src)?,
            "store+best-effort" => capture_xattrs(&it.src).unwrap_or_default(),
            "basic" => capture_xattrs(&it.src).unwrap_or_default(),
            other => bail!("unknown xattr policy: {}", other),
        };

        index.entries.push(Entry {
            path: it.rel,
            kind: EntryKind::Regular,
            mode,
            mtime,
            size: md.len(),
            extents,
            link_target: None,
            xattrs,
        });
    }

    flush_block(&mut out, &mut cur_block_raw, level, dict_bytes)?;

    let blocks_end_offset = out.stream_position()?;

    sink.on_event(ProgressEvent::Phase {
        phase: ProgressPhase::BuildIndex,
        total_bytes: Some(0),
    });

    // Encode index + hash
    let index_bytes = encode_index(&index);
    let index_hash = blake3::hash(&index_bytes);
    let mut index_hash_bytes = [0u8; 32];
    index_hash_bytes.copy_from_slice(index_hash.as_bytes());

    let index_offset = out.stream_position()?;
    out.write_all(&index_bytes)?;
    let index_len = index_bytes.len() as u64;

    sink.on_event(ProgressEvent::Phase {
        phase: ProgressPhase::WriteTail,
        total_bytes: None,
    });

    // Footer v2: magic + blocks_end + index_offset + index_len + hash  (60 bytes)
    out.write_all(FTR_MAGIC_V2)?;
    write_u64_le(&mut out, blocks_end_offset)?;
    write_u64_le(&mut out, index_offset)?;
    write_u64_le(&mut out, index_len)?;
    out.write_all(&index_hash_bytes)?;

    sink.on_event(ProgressEvent::Finish { ok: true });
    Ok(())
}

/// "Auto dicts" MVP: currently uses a single (optional) externally supplied dict.
/// (We keep the signature stable, but don't do per-group dict tables yet.)
#[allow(clippy::too_many_arguments)]
pub fn pack_paths_with_auto_dicts_with_xattrs_progress(
    inputs: &[PathBuf],
    base: &Path,
    output: &Path,
    block_size: u64,
    level: i32,
    sink: SharedSink,
    dict_kib: u32,
    max_samples: usize,
    sample_bytes: usize,
    xattr_policy: &str,
) -> Result<()> {
    // Train one dictionary over all inputs (deterministic sampling) as a first step.
    let dict = if dict_kib == 0 {
        None
    } else {
        Some(dict::train_dict_for_paths_progress(
            inputs,
            base,
            dict_kib,
            sample_bytes,
            max_samples,
            sink.as_ref(),
        )?)
    };
    pack_paths_with_dict_with_xattrs_progress(
        inputs,
        base,
        output,
        block_size,
        level,
        sink,
        dict.as_deref(),
        xattr_policy,
    )
}

/// Append is not implemented in this MVP. This function exists to keep the CLI stable.
#[allow(clippy::too_many_arguments)]
pub fn append_paths_with_dict_with_xattrs_progress(
    _archive: &Path,
    _inputs: &[PathBuf],
    _base: &Path,
    _block_size: u64,
    _level: i32,
    _sink: SharedSink,
    _dict_bytes: Option<&[u8]>,
    _xattr_policy: &str,
) -> Result<()> {
    bail!("append is not implemented yet in this build; create a new archive instead")
}
