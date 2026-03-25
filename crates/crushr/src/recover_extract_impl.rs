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
    pub metadata_degraded_count: usize,
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
    MetadataDegraded,
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

const TRUST_CLASS_CONTRACT: [RecoveryKind; 5] = [
    RecoveryKind::Canonical,
    RecoveryKind::MetadataDegraded,
    RecoveryKind::RecoveredNamed,
    RecoveryKind::RecoveredAnonymous,
    RecoveryKind::Unrecoverable,
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum MetadataClass {
    Ownership,
    Acl,
    Selinux,
    Capability,
    Xattr,
    SpecialFile,
}

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
    trust_class: RecoveryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    missing_metadata_classes: Option<Vec<MetadataClass>>,
    failed_metadata_classes: Vec<MetadataClass>,
    degradation_reason: Option<String>,
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
    let metadata_degraded_dir = opts.out_dir.join("metadata_degraded");
    let recovered_named_dir = opts.out_dir.join("recovered_named");
    let recovery_root = opts.out_dir.join("_crushr_recovery");
    let anonymous_dir = recovery_root.join("anonymous");

    fs::create_dir_all(&canonical_dir)
        .with_context(|| format!("create {}", canonical_dir.display()))?;
    fs::create_dir_all(&metadata_degraded_dir)
        .with_context(|| format!("create {}", metadata_degraded_dir.display()))?;
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
    let mut hardlink_roots = BTreeMap::<u64, PathBuf>::new();
    let mut manifest_entries = Vec::new();
    let mut recovered_named_count = 0usize;
    let mut recovered_anonymous_count = 0usize;

    progress("canonical extraction");
    let mut canonical_count = 0usize;
    let mut metadata_degraded_count = 0usize;
    for entry in &selected_entries {
        let destination = resolve_confined_path(&canonical_dir, &entry.path)?;
        match entry.kind {
            EntryKind::Regular => {
                if safe_paths.contains(entry.path.as_str()) {
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("create {}", parent.display()))?;
                    }
                    if let Some(group_id) = entry.hardlink_group_id {
                        if let Some(root_path) = hardlink_roots.get(&group_id) {
                            if destination.exists() {
                                if opts.overwrite {
                                    fs::remove_file(&destination)
                                        .or_else(|_| fs::remove_dir_all(&destination))
                                        .ok();
                                } else {
                                    bail!(
                                        "destination exists (use --overwrite): {}",
                                        destination.display()
                                    );
                                }
                            }
                            fs::hard_link(root_path, &destination).with_context(|| {
                                format!(
                                    "hardlink {} -> {}",
                                    destination.display(),
                                    root_path.display()
                                )
                            })?;
                        } else {
                            if entry.sparse {
                                write_sparse_entry(
                                    &reader,
                                    entry,
                                    destination.as_path(),
                                    &blocks,
                                    opts.overwrite,
                                )?;
                            } else {
                                let bytes = read_entry_bytes_strict(&reader, entry, &blocks)?;
                                write_entry(destination.as_path(), &bytes, opts.overwrite)?;
                            }
                            hardlink_roots.insert(group_id, destination.clone());
                        }
                    } else if entry.sparse {
                        write_sparse_entry(
                            &reader,
                            entry,
                            destination.as_path(),
                            &blocks,
                            opts.overwrite,
                        )?;
                    } else {
                        let bytes = read_entry_bytes_strict(&reader, entry, &blocks)?;
                        write_entry(destination.as_path(), &bytes, opts.overwrite)?;
                    }
                    let failed_metadata = restore_regular_metadata(destination.as_path(), entry)?;
                    if failed_metadata.is_empty() {
                        canonical_count += 1;
                    } else {
                        let degraded_destination =
                            resolve_confined_path(&metadata_degraded_dir, &entry.path)?;
                        if let Some(parent) = degraded_destination.parent() {
                            fs::create_dir_all(parent)
                                .with_context(|| format!("create {}", parent.display()))?;
                        }
                        fs::rename(&destination, &degraded_destination).with_context(|| {
                            format!(
                                "move {} -> {}",
                                destination.display(),
                                degraded_destination.display()
                            )
                        })?;
                        metadata_degraded_count += 1;
                        manifest_entries.push(RecoveryManifestEntry {
                            recovery_id: format!("rec_{:06}", manifest_entries.len() + 1),
                            assigned_name: Some(entry.path.clone()),
                            size: entry.size,
                            hash: None,
                            recovery_kind: RecoveryKind::MetadataDegraded,
                            trust_class: RecoveryKind::MetadataDegraded,
                            missing_metadata_classes: None,
                            failed_metadata_classes: failed_metadata.clone(),
                            degradation_reason: Some(
                                "required metadata restoration failed".to_string(),
                            ),
                            classification: classify_content(&read_entry_bytes_strict(
                                &reader, entry, &blocks,
                            )?),
                            original_identity: OriginalIdentity {
                                path_status: IdentityStatus::Verified,
                                name_status: IdentityStatus::Verified,
                            },
                            recovery_reason: "metadata restoration failed".to_string(),
                        });
                    }
                }
            }
            EntryKind::Directory => {
                fs::create_dir_all(&destination)
                    .with_context(|| format!("create {}", destination.display()))?;
                restore_directory_metadata(destination.as_path(), entry)?;
                canonical_count += 1;
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
                let _ = restore_ownership(destination.as_path(), entry)?;
                let _ = restore_security_metadata(destination.as_path(), entry);
                canonical_count += 1;
            }
            EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice => {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("create {}", parent.display()))?;
                }
                if destination.exists() {
                    if opts.overwrite {
                        fs::remove_file(&destination)
                            .or_else(|_| fs::remove_dir_all(&destination))
                            .ok();
                    } else {
                        bail!(
                            "destination exists (use --overwrite): {}",
                            destination.display()
                        );
                    }
                }
                restore_special(destination.as_path(), entry)?;
                let _ = restore_ownership(destination.as_path(), entry)?;
                let _ = restore_security_metadata(destination.as_path(), entry);
                canonical_count += 1;
            }
        }
    }

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
                trust_class: RecoveryKind::Unrecoverable,
                missing_metadata_classes: None,
                failed_metadata_classes: Vec::new(),
                degradation_reason: None,
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
                trust_class: RecoveryKind::RecoveredNamed,
                missing_metadata_classes: None,
                failed_metadata_classes: Vec::new(),
                degradation_reason: None,
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
                trust_class: RecoveryKind::RecoveredAnonymous,
                missing_metadata_classes: None,
                failed_metadata_classes: Vec::new(),
                degradation_reason: None,
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

    let canonical_trust = if refused_files.is_empty() && metadata_degraded_count == 0 {
        "COMPLETE"
    } else if canonical_count == 0 {
        "FAILED"
    } else {
        "PARTIAL"
    };
    let (outcome_kind, report) = build_extraction_report(safe_files, refused_files);

    Ok(RecoverExtractRun {
        outcome_kind,
        report,
        manifest_path,
        canonical_count,
        metadata_degraded_count,
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

fn recover_partial_entry_bytes(
    reader: &FileReader,
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

fn write_sparse_entry(
    reader: &FileReader,
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

fn restore_special(path: &Path, entry: &Entry) -> Result<()> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn mkfifo(pathname: *const std::os::raw::c_char, mode: u32) -> std::os::raw::c_int;
            fn mknod(
                pathname: *const std::os::raw::c_char,
                mode: u32,
                dev: u64,
            ) -> std::os::raw::c_int;
        }
        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for special restore: {}", path.display()))?;
        let rc = match entry.kind {
            EntryKind::Fifo => unsafe { mkfifo(c_path.as_ptr(), entry.mode) },
            EntryKind::CharDevice | EntryKind::BlockDevice => {
                let mode = entry.mode
                    | if entry.kind == EntryKind::CharDevice {
                        0o020000
                    } else {
                        0o060000
                    };
                let major = entry.device_major.unwrap_or(0) as u64;
                let minor = entry.device_minor.unwrap_or(0) as u64;
                let dev = ((major & 0xfffff000) << 32)
                    | ((major & 0xfff) << 8)
                    | ((minor & 0xffffff00) << 12)
                    | (minor & 0xff);
                unsafe { mknod(c_path.as_ptr(), mode, dev) }
            }
            _ => 0,
        };
        if rc != 0 {
            eprintln!(
                "WARNING[special-restore]: could not restore '{}' at '{}': {}",
                entry.path,
                path.display(),
                std::io::Error::last_os_error()
            );
        } else {
            restore_mtime(path, entry.mtime)?;
            let _ = restore_xattrs(path, entry)?;
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!(
            "WARNING[special-restore]: skipped '{}' at '{}' (unsupported platform)",
            entry.path,
            path.display()
        );
    }
    Ok(())
}

fn restore_regular_metadata(path: &Path, entry: &Entry) -> Result<Vec<MetadataClass>> {
    let mut failed = Vec::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::set_permissions(path, fs::Permissions::from_mode(entry.mode)).is_err() {
            failed.push(MetadataClass::SpecialFile);
        }
    }
    if restore_mtime(path, entry.mtime).is_err() {
        failed.push(MetadataClass::SpecialFile);
    }
    if restore_xattrs(path, entry)? {
        failed.push(MetadataClass::Xattr);
    }
    if restore_ownership(path, entry)? {
        failed.push(MetadataClass::Ownership);
    }
    let security_failures = restore_security_metadata(path, entry);
    if security_failures.contains(&MetadataClass::Acl) {
        failed.push(MetadataClass::Acl);
    }
    if security_failures.contains(&MetadataClass::Selinux) {
        failed.push(MetadataClass::Selinux);
    }
    if security_failures.contains(&MetadataClass::Capability) {
        failed.push(MetadataClass::Capability);
    }
    failed.sort();
    failed.dedup();
    Ok(failed)
}

fn restore_directory_metadata(path: &Path, entry: &Entry) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(entry.mode)).ok();
    }
    restore_mtime(path, entry.mtime)?;
    let _ = restore_xattrs(path, entry)?;
    let _ = restore_ownership(path, entry)?;
    let _ = restore_security_metadata(path, entry);
    Ok(())
}

fn restore_mtime(path: &Path, mtime_secs: i64) -> Result<()> {
    #[cfg(unix)]
    {
        if mtime_secs < 0 {
            return Ok(());
        }
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::io::RawFd;
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        unsafe extern "C" {
            fn utimensat(
                dirfd: RawFd,
                pathname: *const std::os::raw::c_char,
                times: *const Timespec,
                flags: std::os::raw::c_int,
            ) -> std::os::raw::c_int;
        }
        const AT_FDCWD: RawFd = -100;
        const UTIME_OMIT: i64 = 1_073_741_822;
        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for mtime restore: {}", path.display()))?;
        let times = [
            Timespec {
                tv_sec: 0,
                tv_nsec: UTIME_OMIT,
            },
            Timespec {
                tv_sec: mtime_secs,
                tv_nsec: 0,
            },
        ];
        let rc = unsafe { utimensat(AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("set mtime {}", path.display()));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mtime_secs);
    }
    Ok(())
}

fn restore_xattrs(path: &Path, entry: &Entry) -> Result<bool> {
    let mut failed = false;
    #[cfg(unix)]
    {
        for xa in &entry.xattrs {
            if let Err(err) = xattr::set(path, &xa.name, &xa.value) {
                failed = true;
                eprintln!(
                    "WARNING[xattr-restore]: could not restore '{}' on '{}': {err}",
                    xa.name,
                    path.display()
                );
            }
        }
    }
    #[cfg(not(unix))]
    {
        if !entry.xattrs.is_empty() {
            failed = true;
            eprintln!(
                "WARNING[xattr-restore]: skipped {} xattrs on '{}' (unsupported platform)",
                entry.xattrs.len(),
                path.display()
            );
        }
    }
    Ok(failed)
}

fn restore_security_metadata(path: &Path, entry: &Entry) -> Vec<MetadataClass> {
    let mut failed = Vec::new();
    #[cfg(unix)]
    {
        if restore_single_xattr(
            path,
            "acl-restore",
            "system.posix_acl_access",
            entry.acl_access.as_deref(),
        ) {
            failed.push(MetadataClass::Acl);
        }
        if restore_single_xattr(
            path,
            "acl-restore",
            "system.posix_acl_default",
            entry.acl_default.as_deref(),
        ) {
            failed.push(MetadataClass::Acl);
        }
        if restore_single_xattr(
            path,
            "selinux-restore",
            "security.selinux",
            entry.selinux_label.as_deref(),
        ) {
            failed.push(MetadataClass::Selinux);
        }
        if restore_single_xattr(
            path,
            "capability-restore",
            "security.capability",
            entry.linux_capability.as_deref(),
        ) {
            failed.push(MetadataClass::Capability);
        }
    }
    #[cfg(not(unix))]
    {
        if entry.acl_access.is_some() || entry.acl_default.is_some() {
            failed.push(MetadataClass::Acl);
            eprintln!(
                "WARNING[acl-restore]: skipped ACL metadata on '{}' (unsupported platform)",
                path.display()
            );
        }
        if entry.selinux_label.is_some() {
            failed.push(MetadataClass::Selinux);
            eprintln!(
                "WARNING[selinux-restore]: skipped SELinux label on '{}' (unsupported platform)",
                path.display()
            );
        }
        if entry.linux_capability.is_some() {
            failed.push(MetadataClass::Capability);
            eprintln!(
                "WARNING[capability-restore]: skipped Linux capabilities on '{}' (unsupported platform)",
                path.display()
            );
        }
    }
    failed
}

#[cfg(unix)]
fn restore_single_xattr(path: &Path, warning_code: &str, name: &str, value: Option<&[u8]>) -> bool {
    if let Some(value) = value
        && let Err(err) = xattr::set(path, name, value)
    {
        eprintln!(
            "WARNING[{warning_code}]: could not restore '{name}' on '{}': {err}",
            path.display()
        );
        return true;
    }
    false
}

fn restore_ownership(path: &Path, entry: &Entry) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn lchown(
                path: *const std::os::raw::c_char,
                owner: u32,
                group: u32,
            ) -> std::os::raw::c_int;
        }
        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for ownership restore: {}", path.display()))?;
        let rc = unsafe { lchown(c_path.as_ptr(), entry.uid, entry.gid) };
        if rc != 0 {
            let label = entry
                .uname
                .as_ref()
                .zip(entry.gname.as_ref())
                .map(|(u, g)| format!("{u}:{g}"))
                .unwrap_or_else(|| format!("{}:{}", entry.uid, entry.gid));
            eprintln!(
                "WARNING[ownership-restore]: could not restore '{}' on '{}': {}",
                label,
                path.display(),
                std::io::Error::last_os_error()
            );
            return Ok(true);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, entry);
    }
    Ok(false)
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
