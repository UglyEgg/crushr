// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! Public API for embedding Crushr functionality in other applications.
//!
//! Design goals:
//! - Stable, small surface area (options structs + functions).
//! - No CLI dependencies (no clap, no indicatif).
//! - Errors are `anyhow::Error` for simplicity in early MVP; can evolve to a typed error later.

use anyhow::Result;
use std::path::PathBuf;

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

/// Extract all files from an archive into a directory using strict authoritative semantics.
pub fn extract_all(opts: &ExtractOptions) -> Result<()> {
    crate::strict_extract_impl::run_strict_extract(&crate::strict_extract_impl::StrictExtractOptions {
        archive: opts.archive.clone(),
        out_dir: opts.output_dir.clone(),
        overwrite: opts.overwrite,
        selected_paths: None,
        verify_only: false,
    })
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{extract_all, ExtractOptions};
    use crate::format::{Entry, EntryKind, Extent, Index, BLK_MAGIC_V2, CODEC_ZSTD, FTR_MAGIC_V2};
    use crate::index_codec::encode_index;
    use std::fs;
    use std::io::Write;

    fn build_archive(archive: &std::path::Path, path: &str) {
        let mut f = fs::File::create(archive).expect("create archive");
        let payload = b"hello";
        let comp = zstd::encode_all(&payload[..], 3).expect("compress payload");

        f.write_all(BLK_MAGIC_V2).expect("write BLK magic");
        f.write_all(&CODEC_ZSTD.to_le_bytes()).expect("write codec");
        f.write_all(&(payload.len() as u64).to_le_bytes())
            .expect("write raw len");
        f.write_all(&(comp.len() as u64).to_le_bytes())
            .expect("write comp len");
        f.write_all(&comp).expect("write payload");

        let blocks_end_offset = 4 + 4 + 8 + 8 + comp.len() as u64;
        let index = Index {
            entries: vec![Entry {
                path: path.to_string(),
                kind: EntryKind::Regular,
                mode: 0,
                mtime: 0,
                size: payload.len() as u64,
                extents: vec![Extent {
                    block_id: 0,
                    offset: 0,
                    len: payload.len() as u64,
                }],
                link_target: None,
                xattrs: vec![],
                uid: 0,
                gid: 0,
                uname: None,
                gname: None,
                hardlink_group_id: None,
            }],
        };
        let idx_bytes = encode_index(&index);
        let index_offset = blocks_end_offset;
        let index_len = idx_bytes.len() as u64;
        f.write_all(&idx_bytes).expect("write index");

        let index_hash = blake3::hash(&idx_bytes);
        f.write_all(FTR_MAGIC_V2).expect("write footer magic");
        f.write_all(&blocks_end_offset.to_le_bytes())
            .expect("write blocks_end_offset");
        f.write_all(&index_offset.to_le_bytes())
            .expect("write index_offset");
        f.write_all(&index_len.to_le_bytes()).expect("write index_len");
        f.write_all(index_hash.as_bytes()).expect("write index hash");
    }

    #[test]
    fn public_api_extract_uses_strict_authoritative_behavior() {
        let td = tempfile::TempDir::new().unwrap();
        let archive = td.path().join("api-bad.crs");
        build_archive(&archive, "safe/file.txt");

        let opts = ExtractOptions {
            progress: None,
            archive,
            output_dir: td.path().join("out"),
            overwrite: false,
        };

        extract_all(&opts).expect("strict API extraction succeeds for safe paths");
        assert_eq!(
            fs::read(td.path().join("out/safe/file.txt")).expect("read extracted file"),
            b"hello"
        );
    }
}
