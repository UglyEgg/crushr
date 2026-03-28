// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction_path::resolve_confined_path;
use crate::extraction_payload_core::{
    read_entry_bytes, recover_partial_entry_bytes, write_entry_bytes, write_sparse_entry,
};
use crate::format::{Entry, EntryKind};
use crate::index_codec::decode_index;
use crate::restoration_core::{
    MetadataClass, RestorationPolicy, restore_entry_metadata, restore_special_filesystem_object,
};
use anyhow::{Context, Result, bail};
use crushr_core::{
    extraction::{ExtractionOutcomeKind, build_extraction_report, classify_refusal_paths},
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{BlockSpanV1, scan_blocks_v1, verify_block_payloads_v1},
};
use serde::Serialize;

use crate::recovery_classification::{
    ClassificationBasis, ContentClassification, RecoveryConfidence, classify_and_name,
    classify_content,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

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

const METADATA_DEGRADED_REASON: &str = "required metadata restoration failed";

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
    let preservation_profile = index.preservation_profile;
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
                                let bytes = read_entry_bytes(&reader, entry, &blocks)?;
                                write_entry_bytes(destination.as_path(), &bytes, opts.overwrite)?;
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
                        let bytes = read_entry_bytes(&reader, entry, &blocks)?;
                        write_entry_bytes(destination.as_path(), &bytes, opts.overwrite)?;
                    }
                    let failed_metadata =
                        restore_entry_metadata(destination.as_path(), entry, preservation_profile)?;
                    if route_metadata_degraded_entry(
                        &reader,
                        &blocks,
                        &metadata_degraded_dir,
                        entry,
                        &destination,
                        failed_metadata,
                        &mut manifest_entries,
                    )? {
                        metadata_degraded_count += 1;
                        if let Some(group_id) = entry.hardlink_group_id {
                            let degraded_destination =
                                resolve_confined_path(&metadata_degraded_dir, &entry.path)?;
                            hardlink_roots.insert(group_id, degraded_destination);
                        }
                    } else {
                        canonical_count += 1;
                    }
                }
            }
            EntryKind::Directory => {
                fs::create_dir_all(&destination)
                    .with_context(|| format!("create {}", destination.display()))?;
                let failed_metadata =
                    restore_entry_metadata(destination.as_path(), entry, preservation_profile)?;
                if route_metadata_degraded_entry(
                    &reader,
                    &blocks,
                    &metadata_degraded_dir,
                    entry,
                    &destination,
                    failed_metadata,
                    &mut manifest_entries,
                )? {
                    metadata_degraded_count += 1;
                } else {
                    canonical_count += 1;
                }
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
                let failed_metadata =
                    restore_entry_metadata(destination.as_path(), entry, preservation_profile)?;
                if route_metadata_degraded_entry(
                    &reader,
                    &blocks,
                    &metadata_degraded_dir,
                    entry,
                    &destination,
                    failed_metadata,
                    &mut manifest_entries,
                )? {
                    metadata_degraded_count += 1;
                } else {
                    canonical_count += 1;
                }
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
                let _ = restore_special_filesystem_object(
                    destination.as_path(),
                    entry,
                    RestorationPolicy::Recover,
                )?;
                let failed_metadata =
                    restore_entry_metadata(destination.as_path(), entry, preservation_profile)?;
                if route_metadata_degraded_entry(
                    &reader,
                    &blocks,
                    &metadata_degraded_dir,
                    entry,
                    &destination,
                    failed_metadata,
                    &mut manifest_entries,
                )? {
                    metadata_degraded_count += 1;
                } else {
                    canonical_count += 1;
                }
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
            write_entry_bytes(destination.as_path(), &recovered, opts.overwrite)?;
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
            write_entry_bytes(assigned_path.as_path(), &recovered, opts.overwrite)?;

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

fn route_metadata_degraded_entry(
    reader: &FileReader,
    blocks: &[BlockSpanV1],
    metadata_degraded_dir: &Path,
    entry: &Entry,
    canonical_destination: &Path,
    failed_metadata: Vec<MetadataClass>,
    manifest_entries: &mut Vec<RecoveryManifestEntry>,
) -> Result<bool> {
    if failed_metadata.is_empty() {
        return Ok(false);
    }

    let degraded_destination = resolve_confined_path(metadata_degraded_dir, &entry.path)?;
    if let Some(parent) = degraded_destination.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::rename(canonical_destination, &degraded_destination).with_context(|| {
        format!(
            "move {} -> {}",
            canonical_destination.display(),
            degraded_destination.display()
        )
    })?;

    manifest_entries.push(build_metadata_degraded_manifest_entry(
        reader,
        blocks,
        entry,
        failed_metadata,
        manifest_entries.len() + 1,
    )?);
    Ok(true)
}

fn build_metadata_degraded_manifest_entry(
    reader: &FileReader,
    blocks: &[BlockSpanV1],
    entry: &Entry,
    failed_metadata_classes: Vec<MetadataClass>,
    manifest_ordinal: usize,
) -> Result<RecoveryManifestEntry> {
    Ok(RecoveryManifestEntry {
        recovery_id: format!("rec_{manifest_ordinal:06}"),
        assigned_name: Some(entry.path.clone()),
        size: entry.size,
        hash: None,
        recovery_kind: RecoveryKind::MetadataDegraded,
        trust_class: RecoveryKind::MetadataDegraded,
        missing_metadata_classes: None,
        failed_metadata_classes,
        degradation_reason: Some(METADATA_DEGRADED_REASON.to_string()),
        classification: classify_content_from_entry(reader, entry, blocks)?,
        original_identity: OriginalIdentity {
            path_status: IdentityStatus::Verified,
            name_status: IdentityStatus::Verified,
        },
        recovery_reason: METADATA_DEGRADED_REASON.to_string(),
    })
}

fn classify_content_from_entry(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[BlockSpanV1],
) -> Result<ContentClassification> {
    if entry.kind == EntryKind::Regular {
        Ok(classify_content(&read_entry_bytes(reader, entry, blocks)?))
    } else {
        Ok(ContentClassification {
            kind: "bin".to_string(),
            confidence: RecoveryConfidence::Low,
            basis: ClassificationBasis::Heuristic,
            subtype: Some(format!("{:?}", entry.kind).to_lowercase()),
        })
    }
}
