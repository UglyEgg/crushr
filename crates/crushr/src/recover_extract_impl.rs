// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction_path::resolve_confined_path;
use crate::format::{Entry, EntryKind};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
use crushr_core::{
    extraction::{ExtractionOutcomeKind, build_extraction_report, classify_refusal_paths},
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{BlockSpanV1, scan_blocks_v1, verify_block_payloads_v1},
};
use crushr_format::blk3::read_blk3_header;
use serde::Serialize;

use crate::recovery_classification::{
    ClassificationBasis, ContentClassification, RecoveryConfidence, classify_and_name,
    classify_content,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::commands::salvage::RecoveryAnalysis;

struct FileReader {
    file: File,
}

impl ReadAt for FileReader {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        use std::os::unix::fs::FileExt;
        Ok(self.file.read_at(buf, offset)?)
    }
}

impl Len for FileReader {
    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

#[derive(Debug, Clone)]
pub struct RecoverExtractOptions {
    pub archive: PathBuf,
    pub out_dir: PathBuf,
    pub overwrite: bool,
    pub selected_paths: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct RecoverExtractRun {
    pub outcome_kind: ExtractionOutcomeKind,
    pub report: crushr_core::extraction::ExtractionReport,
    pub manifest_path: PathBuf,
    pub canonical_count: usize,
    pub recovered_named_count: usize,
    pub recovered_anonymous_count: usize,
    pub unrecoverable_count: usize,
    pub canonical_trust: &'static str,
    pub recovery_trust: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecoveryKind {
    Canonical,
    RecoveredNamed,
    RecoveredAnonymous,
    Unrecoverable,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum IdentityStatus {
    Verified,
    Untrusted,
    Unknown,
    Lost,
}

const TRUST_CLASS_CONTRACT: [RecoveryKind; 4] = [
    RecoveryKind::Canonical,
    RecoveryKind::RecoveredNamed,
    RecoveryKind::RecoveredAnonymous,
    RecoveryKind::Unrecoverable,
];

const IDENTITY_STATUS_CONTRACT: [IdentityStatus; 4] = [
    IdentityStatus::Verified,
    IdentityStatus::Untrusted,
    IdentityStatus::Unknown,
    IdentityStatus::Lost,
];

#[derive(Debug, Serialize)]
struct RecoveryManifest {
    schema_version: &'static str,
    mode: &'static str,
    entries: Vec<RecoveryManifestEntry>,
}

#[derive(Debug, Serialize)]
struct RecoveryManifestEntry {
    recovery_id: String,
    assigned_name: Option<String>,
    size: u64,
    hash: Option<String>,
    recovery_kind: RecoveryKind,
    classification: ContentClassification,
    original_identity: OriginalIdentity,
    recovery_reason: String,
}

#[derive(Debug, Serialize)]
struct OriginalIdentity {
    path_status: IdentityStatus,
    name_status: IdentityStatus,
}

pub fn run_recover_extract_with_progress<F>(
    opts: &RecoverExtractOptions,
    mut progress: F,
) -> Result<RecoverExtractRun>
where
    F: FnMut(&'static str),
{
    let _ = TRUST_CLASS_CONTRACT;
    let _ = IDENTITY_STATUS_CONTRACT;
    progress("archive open");
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };

    let opened = open_archive_v1(&reader)?;
    progress("metadata scan");
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    let index = decode_index(&opened.tail.idx3_bytes).context("decode IDX3")?;
    let corrupted = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;

    let canonical_dir = opts.out_dir.join("canonical");
    let recovered_named_dir = opts.out_dir.join("recovered_named");
    let recovery_root = opts.out_dir.join("_crushr_recovery");
    let anonymous_dir = recovery_root.join("anonymous");

    fs::create_dir_all(&canonical_dir)
        .with_context(|| format!("create {}", canonical_dir.display()))?;
    fs::create_dir_all(&recovered_named_dir)
        .with_context(|| format!("create {}", recovered_named_dir.display()))?;
    fs::create_dir_all(&anonymous_dir)
        .with_context(|| format!("create {}", anonymous_dir.display()))?;

    let mut entries: Vec<Entry> = index.entries;
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let selected = opts
        .selected_paths
        .as_ref()
        .map(|paths| paths.iter().cloned().collect::<BTreeSet<_>>());

    let mut required_blocks_by_path = BTreeMap::<String, Vec<u32>>::new();
    let mut selected_entries = Vec::new();
    for entry in entries {
        if let Some(selected_paths) = &selected
            && !selected_paths.contains(&entry.path)
        {
            continue;
        }
        if entry.kind == EntryKind::Regular {
            required_blocks_by_path.insert(
                entry.path.clone(),
                entry.extents.iter().map(|extent| extent.block_id).collect(),
            );
        }
        selected_entries.push(entry);
    }

    let candidate_paths = required_blocks_by_path.keys().cloned().collect::<Vec<_>>();
    let (safe_files, refused_files) = classify_refusal_paths(candidate_paths, &corrupted, |path| {
        required_blocks_by_path
            .get(path)
            .cloned()
            .unwrap_or_default()
    });

    let safe_paths = safe_files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();

    progress("canonical extraction");
    for entry in &selected_entries {
        let destination = resolve_confined_path(&canonical_dir, &entry.path)?;
        match entry.kind {
            EntryKind::Regular => {
                if safe_paths.contains(entry.path.as_str()) {
                    let bytes = read_entry_bytes_strict(&reader, entry, &blocks)?;
                    write_entry(destination.as_path(), &bytes, opts.overwrite)?;
                }
            }
            EntryKind::Directory => {
                fs::create_dir_all(&destination)
                    .with_context(|| format!("create {}", destination.display()))?;
            }
            EntryKind::Symlink => {
                let target = entry.link_target.clone().unwrap_or_default();
                crate::extraction_path::validate_symlink_target(&target)?;
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("create {}", parent.display()))?;
                }
                #[cfg(unix)]
                std::os::unix::fs::symlink(&target, &destination)
                    .with_context(|| format!("symlink {} -> {}", destination.display(), target))?;
            }
        }
    }

    let mut manifest_entries = Vec::new();
    let mut recovered_named_count = 0usize;
    let mut recovered_anonymous_count = 0usize;

    let refused_paths = refused_files
        .iter()
        .map(|f| f.path.as_str())
        .collect::<BTreeSet<_>>();

    for entry in selected_entries
        .iter()
        .filter(|entry| refused_paths.contains(entry.path.as_str()))
    {
        let recovery_id = format!("rec_{:06}", manifest_entries.len() + 1);
        let recovered = recover_partial_entry_bytes(&reader, entry, &blocks)?;

        if recovered.is_empty() {
            manifest_entries.push(RecoveryManifestEntry {
                recovery_id,
                assigned_name: None,
                size: 0,
                hash: None,
                recovery_kind: RecoveryKind::Unrecoverable,
                classification: ContentClassification {
                    kind: "bin".to_string(),
                    confidence: RecoveryConfidence::Low,
                    basis: ClassificationBasis::Heuristic,
                    subtype: None,
                },
                original_identity: OriginalIdentity {
                    path_status: IdentityStatus::Lost,
                    name_status: IdentityStatus::Lost,
                },
                recovery_reason: "all required extents failed strict verification".to_string(),
            });
            continue;
        }

        if recovered.len() as u64 == entry.size {
            let destination = resolve_confined_path(&recovered_named_dir, &entry.path)?;
            write_entry(destination.as_path(), &recovered, opts.overwrite)?;
            recovered_named_count += 1;
            manifest_entries.push(RecoveryManifestEntry {
                recovery_id,
                assigned_name: Some(entry.path.clone()),
                size: recovered.len() as u64,
                hash: Some(format!("blake3:{}", blake3::hash(&recovered).to_hex())),
                recovery_kind: RecoveryKind::RecoveredNamed,
                classification: classify_content(&recovered),
                original_identity: OriginalIdentity {
                    path_status: IdentityStatus::Untrusted,
                    name_status: IdentityStatus::Untrusted,
                },
                recovery_reason:
                    "canonical path refused; full payload recovered with untrusted identity"
                        .to_string(),
            });
        } else {
            let naming = classify_and_name(&recovered, recovered_anonymous_count + 1);
            let assigned_path = anonymous_dir.join(&naming.assigned_name);
            write_entry(assigned_path.as_path(), &recovered, opts.overwrite)?;

            recovered_anonymous_count += 1;
            manifest_entries.push(RecoveryManifestEntry {
                recovery_id,
                assigned_name: Some(naming.assigned_name),
                size: recovered.len() as u64,
                hash: Some(format!("blake3:{}", blake3::hash(&recovered).to_hex())),
                recovery_kind: RecoveryKind::RecoveredAnonymous,
                classification: naming.classification,
                original_identity: OriginalIdentity {
                    path_status: IdentityStatus::Lost,
                    name_status: IdentityStatus::Lost,
                },
                recovery_reason:
                    "canonical path refused; recovered verified extents without trusted identity"
                        .to_string(),
            });
        }
    }
    progress("recovery extraction");
    let unrecoverable_count = manifest_entries
        .iter()
        .filter(|entry| matches!(entry.recovery_kind, RecoveryKind::Unrecoverable))
        .count();

    let manifest = RecoveryManifest {
        schema_version: "crushr-recovery-manifest.v1",
        mode: "recover_extract",
        entries: manifest_entries,
    };

    let manifest_path = recovery_root.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize recovery manifest"),
    )
    .with_context(|| format!("write {}", manifest_path.display()))?;
    progress("manifest/report finalization");

    let canonical_count = safe_files.len();
    let canonical_trust = if refused_files.is_empty() {
        "COMPLETE"
    } else {
        "PARTIAL"
    };
    let (outcome_kind, report) = build_extraction_report(safe_files, refused_files);

    Ok(RecoverExtractRun {
        outcome_kind,
        report,
        manifest_path,
        canonical_count,
        recovered_named_count,
        recovered_anonymous_count,
        unrecoverable_count,
        canonical_trust,
        recovery_trust: if unrecoverable_count == 0 {
            "COMPLETE"
        } else {
            "PARTIAL"
        },
    })
}

pub fn run_recovery_analysis(archive: &Path) -> Result<RecoveryAnalysis> {
    crate::commands::salvage::build_recovery_analysis(archive)
}

fn read_entry_bytes_strict(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(entry.size as usize);

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

        out.extend_from_slice(&raw[begin..end]);
    }

    if out.len() as u64 != entry.size {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(out)
}

fn recover_partial_entry_bytes(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<Vec<u8>> {
    let mut out = Vec::new();

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

        out.extend_from_slice(&raw[begin..end]);
    }

    Ok(out)
}

fn block_raw_payload(reader: &FileReader, block: &BlockSpanV1) -> Result<Vec<u8>> {
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

fn write_entry(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    if path.exists() && !overwrite {
        bail!("destination exists (use --overwrite): {}", path.display());
    }

    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
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
