use anyhow::{bail, Context, Result};
use blake3::Hash;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const DICT_MAGIC: &[u8; 4] = b"ZDCT";
const DICT_VERSION: u32 = 1;

/// A portable dictionary file format for crushr.
/// This is NOT the zstd "raw dict" format; it wraps raw bytes with metadata and a checksum.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DictFile {
    pub dict_bytes: Vec<u8>,
    pub dict_hash: [u8; 32],
    pub dict_size: u32,
    pub sample_count: u32,
    pub sample_bytes: u64,
}

#[allow(dead_code)]
fn put_u32(out: &mut Vec<u8>, v: u32) { out.extend_from_slice(&v.to_le_bytes()); }
#[allow(dead_code)]
fn put_u64(out: &mut Vec<u8>, v: u64) { out.extend_from_slice(&v.to_le_bytes()); }

fn read_u32(r: &mut impl Read) -> Result<u32> { let mut b=[0u8;4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn read_u64(r: &mut impl Read) -> Result<u64> { let mut b=[0u8;8]; r.read_exact(&mut b)?; Ok(u64::from_le_bytes(b)) }

#[allow(dead_code)]
pub fn write_dict(path: &Path, df: &DictFile) -> Result<()> {
    let mut out = Vec::new();
    out.extend_from_slice(DICT_MAGIC);
    put_u32(&mut out, DICT_VERSION);
    put_u32(&mut out, df.dict_size);
    put_u32(&mut out, df.sample_count);
    put_u64(&mut out, df.sample_bytes);
    out.extend_from_slice(&df.dict_hash);
    put_u32(&mut out, df.dict_bytes.len() as u32);
    out.extend_from_slice(&df.dict_bytes);

    let mut f = File::create(path).with_context(|| format!("create {}", path.display()))?;
    f.write_all(&out)?;
    Ok(())
}

pub fn read_dict(path: &Path) -> Result<DictFile> {
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic)?;
    if &magic != DICT_MAGIC { bail!("not a crushr dict file"); }
    let ver = read_u32(&mut f)?;
    if ver != DICT_VERSION { bail!("unsupported dict version {}", ver); }

    let dict_size = read_u32(&mut f)?;
    let sample_count = read_u32(&mut f)?;
    let sample_bytes = read_u64(&mut f)?;
    let mut dict_hash = [0u8; 32];
    f.read_exact(&mut dict_hash)?;
    let dict_len = read_u32(&mut f)? as usize;
    let mut dict_bytes = vec![0u8; dict_len];
    f.read_exact(&mut dict_bytes)?;

    let h = blake3::hash(&dict_bytes);
    if h.as_bytes() != &dict_hash { bail!("dict checksum mismatch"); }

    Ok(DictFile { dict_bytes, dict_hash, dict_size, sample_count, sample_bytes })
}

/// Train a zstd dictionary from samples (already collected).
pub fn train_from_samples(samples: &[Vec<u8>], dict_size: usize) -> Result<Vec<u8>> {
    // zstd dict builder expects a slice-of-slices.
    let refs: Vec<&[u8]> = samples.iter().map(|s| s.as_slice()).collect();
    let dict = zstd::dict::from_samples(&refs, dict_size)
        .context("zstd dict training failed")?;
    Ok(dict)
}

#[allow(dead_code)]
pub fn hash_dict(dict_bytes: &[u8]) -> Hash { blake3::hash(dict_bytes) }


use std::path::PathBuf;

fn to_rel(base: &Path, p: &Path) -> Result<String> {
    let rel = p.strip_prefix(base)
        .with_context(|| format!("path {} is not under base {}", p.display(), base.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn collect_files(inputs: &[PathBuf], base: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for inp in inputs {
        let meta = std::fs::symlink_metadata(inp)
            .with_context(|| format!("stat {}", inp.display()))?;
        if meta.is_dir() {
            for e in walkdir::WalkDir::new(inp).follow_links(false) {
                let e = e?;
                if !e.file_type().is_file() { continue; }
                let _ = to_rel(base, e.path())?;
                out.push(e.path().to_path_buf());
            }
        } else if meta.is_file() {
            let _ = to_rel(base, inp)?;
            out.push(inp.clone());
        }
    }
    out.sort_by(|a,b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(out)
}

fn read_prefix(path: &Path, cap: usize) -> Result<Vec<u8>> {
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buf = vec![0u8; cap];
    let n = f.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

/// Train a single zstd dictionary over a set of inputs (files and/or directories).
pub fn train_dict_for_paths(
    inputs: &[PathBuf],
    base: &Path,
    dict_kib: u32,
    max_samples: usize,
    sample_bytes: usize,
) -> Result<Vec<u8>> {
    let files = collect_files(inputs, base)?;
    let mut samples: Vec<Vec<u8>> = Vec::new();
    for p in files.into_iter().take(max_samples) {
        let b = read_prefix(&p, sample_bytes)?;
        if !b.is_empty() { samples.push(b); }
    }
    if samples.is_empty() { bail!("no samples available for dictionary training"); }
    train_from_samples(&samples, (dict_kib as usize) * 1024)
}pub fn train_dict_for_paths_progress(
    inputs: &[PathBuf],
    base: &Path,
    dict_kib: u32,
    sample_bytes: usize,
    max_samples: usize,
    sink: &dyn crate::progress::ProgressSink,
) -> Result<Vec<u8>> {
    use crate::progress::ProgressEvent;
    // Enumerate candidate files under base, deterministic order.
    let mut files: Vec<PathBuf> = Vec::new();
    for input in inputs {
        for e in WalkDir::new(input).follow_links(false) {
            let e = e?;
            if !e.file_type().is_file() { continue; }
            files.push(e.path().to_path_buf());
        }
    }
    files.sort();

    // Stratified ordering improves dict quality on mixed trees.
    let files = {
        let mut buckets: [Vec<PathBuf>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for p in files {
            let rel = p.strip_prefix(base).unwrap_or(&p);
            let g = crate::format::classify_group(&rel.to_string_lossy());
            buckets[g as usize].push(p);
        }
        let mut out = Vec::new();
        let mut idx = [0usize; 4];
        loop {
            let mut progressed = false;
            for g in 0..4 {
                if idx[g] < buckets[g].len() {
                    out.push(buckets[g][idx[g]].clone());
                    idx[g] += 1;
                    progressed = true;
                }
            }
            if !progressed { break; }
        }
        out
    };

    // Total bytes for progress ~= bytes actually sampled (bounded).
    let total = (files.len().min(max_samples) as u64) * (sample_bytes as u64);
    sink.on_event(ProgressEvent::Phase { phase: crate::progress::ProgressPhase::TrainDict, total_bytes: Some(total) });

    let mut samples: Vec<u8> = Vec::new();
    samples.reserve(files.len().min(max_samples) * sample_bytes);

    let mut taken = 0usize;
    for p in files.into_iter() {
        if taken >= max_samples { break; }
        let mut f = std::fs::File::open(&p)?;
        let mut buf = vec![0u8; sample_bytes];
        let n = f.read(&mut buf)?;
        buf.truncate(n);
        samples.extend_from_slice(&buf);
        sink.on_event(ProgressEvent::AdvanceBytes { bytes: n as u64 });
        taken += 1;
    }

    let dict_size = (dict_kib as usize) * 1024;
    let dict = zstd::dict::from_continuous(&samples, dict_size)?;
    Ok(dict)
}

