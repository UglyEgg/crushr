use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TuneRequest {
    pub inputs: Vec<PathBuf>,
    pub base: PathBuf,
    pub time_budget_ms: u64,
    pub max_samples: usize,
    pub sample_bytes: usize,
    pub dict_sizes: Vec<usize>,
    pub levels: Vec<i32>,
    pub block_mibs: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TuneCandidate {
    pub block_mib: u64,
    pub level: i32,
    pub dict_size: usize,
    pub sample_input_bytes: u64,
    pub compressed_bytes: u64,
    pub encode_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TuneResult {
    pub chosen: TuneCandidate,
    pub top: Vec<TuneCandidate>,
}

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
                // ensure under base
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
fn stratified_sample_order(mut files: Vec<PathBuf>, base: &Path) -> Vec<PathBuf> {
    // Deterministic stratified ordering: bucket by coarse content group inferred from relative path,
    // then interleave buckets round-robin. This improves dictionary training and autotune stability
    // across mixed-content trees (docs, code, assets, etc.).
    files.sort();
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
}

fn sample_bytes_for_file(path: &Path, cap: usize) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buf = vec![0u8; cap];
    let n = f.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

fn simulate_blocks_and_compress(data: &[u8], block_size: usize, level: i32, dict: Option<&[u8]>) -> Result<(u64, u64)> {
    use std::io::Write;
    let start = Instant::now();
    let mut total_c: u64 = 0;
    for chunk in data.chunks(block_size) {
        let mut enc = if let Some(d) = dict {
            zstd::Encoder::with_dictionary(Vec::new(), level, d)?
        } else {
            zstd::Encoder::new(Vec::new(), level)?
        };
        enc.write_all(chunk)?;
        let c = enc.finish()?;
        total_c += c.len() as u64;
    }
    let ms = start.elapsed().as_millis() as u64;
    Ok((total_c, ms))
}

pub fn tune(req: &TuneRequest) -> Result<TuneResult> {
    // Collect files and build a deterministic sample corpus.
    let files = stratified_sample_order(collect_files(&req.inputs, &req.base)?, req.base.as_path());
    if files.is_empty() { bail!("no files to sample"); }

    let mut samples: Vec<Vec<u8>> = Vec::new();
    let mut total_in: u64 = 0;
    for p in files.into_iter().take(req.max_samples) {
        let b = sample_bytes_for_file(&p, req.sample_bytes)?;
        if b.is_empty() { continue; }
        total_in += b.len() as u64;
        samples.push(b);
    }
    if samples.is_empty() { bail!("no non-empty samples"); }

    // Concatenate samples for simulation.
    let mut corpus: Vec<u8> = Vec::with_capacity(total_in as usize);
    for s in &samples { corpus.extend_from_slice(s); }

    let mut candidates: Vec<TuneCandidate> = Vec::new();
    let deadline = Instant::now() + std::time::Duration::from_millis(req.time_budget_ms);

    // Pre-train dicts per dict_size.
    let mut dicts: std::collections::HashMap<usize, Vec<u8>> = std::collections::HashMap::new();
    for &ds in &req.dict_sizes {
        if ds == 0 { continue; }
        if Instant::now() >= deadline { break; }
        let dict = crate::dict::train_from_samples(&samples, ds)?;
        dicts.insert(ds, dict);
    }

    for &block_mib in &req.block_mibs {
        for &level in &req.levels {
            for &ds in &req.dict_sizes {
                if Instant::now() >= deadline { break; }
                let dict = if ds == 0 { None } else { dicts.get(&ds).map(|v| v.as_slice()) };
                let block_size = (block_mib * 1024 * 1024) as usize;
                let (cbytes, ms) = simulate_blocks_and_compress(&corpus, block_size, level, dict)?;
                candidates.push(TuneCandidate {
                    block_mib,
                    level,
                    dict_size: ds,
                    sample_input_bytes: total_in,
                    compressed_bytes: cbytes,
                    encode_ms: ms,
                });
            }
        }
    }

    if candidates.is_empty() { bail!("no candidates evaluated"); }

    // Score: balanced => minimize (compressed_bytes * encode_ms) (ratio per time)
    candidates.sort_by(|a,b| {
        let sa = (a.compressed_bytes as u128) * ((a.encode_ms.max(1)) as u128);
        let sb = (b.compressed_bytes as u128) * ((b.encode_ms.max(1)) as u128);
        sa.cmp(&sb)
            .then_with(|| a.compressed_bytes.cmp(&b.compressed_bytes))
    });

    let chosen = candidates[0].clone();
    let top = candidates.iter().take(5).cloned().collect();
    Ok(TuneResult { chosen, top })
}


/// Convenience wrapper used by the CLI. Uses conservative defaults and a time budget.
pub fn autotune(inputs: &[PathBuf], base: &Path, time_budget_ms: u64) -> Result<TuneResult> {
    let req = TuneRequest {
        inputs: inputs.to_vec(),
        base: base.to_path_buf(),
        time_budget_ms,
        max_samples: 64,
        sample_bytes: 128 * 1024,
        dict_sizes: vec![0, 16 * 1024, 32 * 1024],
        levels: vec![5, 10, 15],
        block_mibs: vec![1, 4, 8],
    };
    tune(&req)
}