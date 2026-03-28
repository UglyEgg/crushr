// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction_path::resolve_confined_path;
use crate::extraction_payload_core::{
    read_entry_bytes, validate_entry_bytes, write_entry_bytes, write_sparse_entry,
};
use crate::format::{Entry, EntryKind, PreservationProfile};
use crate::index_codec::decode_index;
use crate::restoration_core::{
    MetadataClass, RestorationPolicy, metadata_required_by_profile, restore_entry_metadata,
    restore_special_filesystem_object,
};
use anyhow::{Context, Result, bail};
use crushr_core::{
    extraction::{ExtractionOutcomeKind, build_extraction_report, classify_refusal_paths},
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{scan_blocks_v1, verify_block_payloads_v1},
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
pub struct StrictExtractOptions {
    pub archive: PathBuf,
    pub out_dir: PathBuf,
    pub overwrite: bool,
    pub selected_paths: Option<Vec<String>>,
    pub verify_only: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct StrictExtractRun {
    pub outcome_kind: ExtractionOutcomeKind,
    pub report: crushr_core::extraction::ExtractionReport,
}

pub fn run_strict_extract(opts: &StrictExtractOptions) -> Result<StrictExtractRun> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };

    let opened = open_archive_v1(&reader)?;
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    let index = decode_index(&opened.tail.idx3_bytes).context("decode IDX3")?;
    let preservation_profile = index.preservation_profile;
    let corrupted = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;

    if !opts.verify_only {
        fs::create_dir_all(&opts.out_dir)
            .with_context(|| format!("create {}", opts.out_dir.display()))?;
    }

    let mut entries: Vec<Entry> = index.entries;
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let selected = opts
        .selected_paths
        .as_ref()
        .map(|paths| paths.iter().cloned().collect::<BTreeSet<_>>());

    let mut required_blocks_by_path = BTreeMap::<String, Vec<u32>>::new();
    for entry in &entries {
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
    let mut metadata_failures: Vec<(String, Vec<MetadataClass>)> = Vec::new();

    for entry in entries {
        if entry.kind == EntryKind::Regular && !safe_paths.contains(entry.path.as_str()) {
            continue;
        }

        if opts.verify_only {
            validate_entry_bytes(&reader, &entry, &blocks)?;
        } else {
            let destination = resolve_confined_path(&opts.out_dir, &entry.path)?;
            write_entry(
                &reader,
                &entry,
                destination.as_path(),
                &blocks,
                opts.overwrite,
                preservation_profile,
                &mut hardlink_roots,
            )
            .map(|failed| {
                let failed = failed
                    .into_iter()
                    .filter(|class| {
                        metadata_required_by_profile(preservation_profile, &entry, *class)
                    })
                    .collect::<Vec<_>>();
                if !failed.is_empty() {
                    metadata_failures.push((entry.path.clone(), failed));
                }
            })?;
        }
    }

    if !metadata_failures.is_empty() {
        metadata_failures.sort_by(|a, b| a.0.cmp(&b.0));
        let (path, classes) = &metadata_failures[0];
        let class_list = classes
            .iter()
            .map(|class| format!("{class:?}").to_lowercase())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "strict extraction refused: metadata restoration failed for {} entries (first: '{}' failed [{}])",
            metadata_failures.len(),
            path,
            class_list
        );
    }

    let (outcome_kind, report) = build_extraction_report(safe_files, refused_files);

    Ok(StrictExtractRun {
        outcome_kind,
        report,
    })
}

fn write_entry(
    reader: &FileReader,
    entry: &Entry,
    path: &Path,
    blocks: &[crushr_core::verify::BlockSpanV1],
    overwrite: bool,
    preservation_profile: PreservationProfile,
    hardlink_roots: &mut BTreeMap<u64, PathBuf>,
) -> Result<Vec<MetadataClass>> {
    match entry.kind {
        EntryKind::Directory => {
            fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
            restore_entry_metadata(path, entry, preservation_profile)
        }
        EntryKind::Symlink => {
            if path.exists() {
                if overwrite {
                    fs::remove_file(path)
                        .or_else(|_| fs::remove_dir_all(path))
                        .ok();
                } else {
                    bail!("destination exists (use --overwrite): {}", path.display());
                }
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            let target = entry.link_target.clone().unwrap_or_default();
            crate::extraction_path::validate_symlink_target(&target)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, path)
                .with_context(|| format!("symlink {} -> {}", path.display(), target))?;
            #[cfg(not(unix))]
            bail!("symlink extraction is unsupported on this platform");
            restore_entry_metadata(path, entry, preservation_profile)
        }
        EntryKind::Regular => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            if let Some(group_id) = entry.hardlink_group_id {
                if let Some(root_path) = hardlink_roots.get(&group_id) {
                    if path.exists() {
                        if overwrite {
                            fs::remove_file(path)
                                .or_else(|_| fs::remove_dir_all(path))
                                .ok();
                        } else {
                            bail!("destination exists (use --overwrite): {}", path.display());
                        }
                    }
                    fs::hard_link(root_path, path).with_context(|| {
                        format!("hardlink {} -> {}", path.display(), root_path.display())
                    })?;
                } else {
                    if entry.sparse {
                        write_sparse_entry(reader, entry, path, blocks, overwrite)?;
                    } else {
                        let bytes = read_entry_bytes(reader, entry, blocks)?;
                        write_entry_bytes(path, &bytes, overwrite)?;
                    }
                    hardlink_roots.insert(group_id, path.to_path_buf());
                }
            } else if entry.sparse {
                write_sparse_entry(reader, entry, path, blocks, overwrite)?;
            } else {
                let bytes = read_entry_bytes(reader, entry, blocks)?;
                write_entry_bytes(path, &bytes, overwrite)?;
            }
            restore_entry_metadata(path, entry, preservation_profile)
        }
        EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            if path.exists() {
                if overwrite {
                    fs::remove_file(path)
                        .or_else(|_| fs::remove_dir_all(path))
                        .ok();
                } else {
                    bail!("destination exists (use --overwrite): {}", path.display());
                }
            }
            let mut failed = Vec::new();
            failed.extend(restore_special_filesystem_object(
                path,
                entry,
                RestorationPolicy::Strict,
            )?);
            failed.extend(restore_entry_metadata(path, entry, preservation_profile)?);
            failed.sort();
            failed.dedup();
            Ok(failed)
        }
    }
}
