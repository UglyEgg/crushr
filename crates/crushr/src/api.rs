//! Public API for embedding Crushr functionality in other applications.
//!
//! Design goals:
//! - Stable, small surface area (options structs + functions).
//! - No CLI dependencies (no clap, no indicatif).
//! - Errors are `anyhow::Error` for simplicity in early MVP; can evolve to a typed error later.

use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PackOptions {
    pub progress: Option<crate::progress::SharedSink>,
    pub base: PathBuf,
    pub output: PathBuf,
    pub block_size: u64,
    pub level: i32,
    pub dict_path: Option<PathBuf>,
    pub auto_dict: bool,
    pub auto_dict_kib: u32,
    pub auto_dict_max_samples: u32,
    pub auto_dict_sample_kib: u32,
    pub xattr_policy: String,
}

#[derive(Debug, Clone)]
pub struct AppendOptions {
    pub progress: Option<crate::progress::SharedSink>,
    pub archive: PathBuf,
    pub base: PathBuf,
    pub block_size: u64,
    pub level: i32,
    pub dict_path: Option<PathBuf>,
    pub xattr_policy: String,
}

#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub progress: Option<crate::progress::SharedSink>,
    pub archive: PathBuf,
    pub output_dir: PathBuf,
    pub overwrite: bool,
}

#[derive(Debug, Clone)]
pub struct RecoverOptions {
    pub progress: Option<crate::progress::SharedSink>,
    pub input: PathBuf,
    pub output: PathBuf,
    pub tail_scan_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct SalvageOptions {
    pub progress: Option<crate::progress::SharedSink>,
    pub input: PathBuf,
    pub output: PathBuf,
}

/// Pack files/dirs into an archive.
pub fn pack(inputs: &[PathBuf], opts: &PackOptions) -> Result<()> {
    let sink = opts.progress.clone().unwrap_or_else(|| std::sync::Arc::new(crate::progress::NullProgressSink));
    if opts.auto_dict && opts.dict_path.is_none() {
        return crate::pack::pack_paths_with_auto_dicts_with_xattrs_progress(
            inputs,
            &opts.base,
            &opts.output,
            opts.block_size,
            opts.level,
            sink.clone(),
            opts.auto_dict_kib,
            opts.auto_dict_max_samples as usize,
            (opts.auto_dict_sample_kib as usize) * 1024,
            &opts.xattr_policy,
        );
    }

    let dict_bytes = if let Some(ref p) = opts.dict_path {
        Some(crate::dict::read_dict(p)?.dict_bytes)
    } else {
        None
    };

    crate::pack::pack_paths_with_dict_with_xattrs_progress(
        inputs,
        &opts.base,
        &opts.output,
        opts.block_size,
        opts.level,
        sink.clone(),
        dict_bytes.as_deref(),
        &opts.xattr_policy,
    )
}

/// Append files/dirs into an existing archive.
pub fn append(inputs: &[PathBuf], opts: &AppendOptions) -> Result<()> {
    let sink = opts.progress.clone().unwrap_or_else(|| std::sync::Arc::new(crate::progress::NullProgressSink));
    let dict_bytes = if let Some(ref p) = opts.dict_path {
        Some(crate::dict::read_dict(p)?.dict_bytes)
    } else {
        None
    };

    crate::pack::append_paths_with_dict_with_xattrs_progress(
        &opts.archive,
        inputs,
        &opts.base,
        opts.block_size,
        opts.level,
        sink.clone(),
        dict_bytes.as_deref(),
        &opts.xattr_policy,
    )
}

/// Extract all files from an archive into a directory.
pub fn extract_all(opts: &ExtractOptions) -> Result<()> {
    let sink = opts.progress.clone().unwrap_or_else(|| std::sync::Arc::new(crate::progress::NullProgressSink));
    crate::extract::extract_all_progress(&opts.archive, &opts.output_dir, opts.overwrite, sink)
}

/// Attempt tail-based repair using redundant tail frames.
pub fn recover(opts: &RecoverOptions) -> Result<()> {
    let sink = opts.progress.clone().unwrap_or_else(|| std::sync::Arc::new(crate::progress::NullProgressSink));
    sink.on_event(crate::progress::ProgressEvent::Start { op: crate::progress::ProgressOp::Recover, phase: crate::progress::ProgressPhase::Other, total_bytes: 0 });
    { let r = crate::recovery::repair_archive(&opts.input, &opts.output, opts.tail_scan_bytes); sink.on_event(crate::progress::ProgressEvent::Finish { ok: r.is_ok() }); r }
}

/// Salvage rebuild by scanning embedded EVT frames.
pub fn salvage(opts: &SalvageOptions) -> Result<()> {
    let sink = opts.progress.clone().unwrap_or_else(|| std::sync::Arc::new(crate::progress::NullProgressSink));
    sink.on_event(crate::progress::ProgressEvent::Start { op: crate::progress::ProgressOp::Salvage, phase: crate::progress::ProgressPhase::Other, total_bytes: 0 });
    { let r = crate::recovery::salvage_archive(&opts.input, &opts.output); sink.on_event(crate::progress::ProgressEvent::Finish { ok: r.is_ok() }); r }
}
