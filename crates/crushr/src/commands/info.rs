// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::cli_presentation::{BannerLevel, CliPresenter, StatusWord, group_u64};
use crate::extraction_payload_core::read_entry_bytes;
use crate::format::{
    Entry, EntryKind, Extent, IDX_MAGIC_V3, IDX_MAGIC_V4, IDX_MAGIC_V5, IDX_MAGIC_V6, IDX_MAGIC_V7,
    PreservationProfile,
};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
use crushr_core::verify::scan_blocks_v1;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    propagation::{
        ActivatedImpactKind, EntryImpactV1, EntryTrustClass as PropagationEntryTrustClass,
        FileDependencyV1, PropagationDependencyReason, PropagationImpactReason,
        PropagationReportV1, STRUCTURE_FTR4, STRUCTURE_IDX3, STRUCTURE_TAIL_FRAME,
        build_propagation_report_v1, build_structural_failure_report_v1,
    },
    snapshot::{info_envelope_from_open_archive, serialize_snapshot_json},
    verify::verify_block_payloads_v1,
};
use crushr_format::blk3::{BLK3_MAGIC, read_blk3_header};
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use crushr_format::tailframe::parse_tail_frame;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Cursor;

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

fn read_exact_at<R: ReadAt>(reader: &R, mut offset: u64, mut dst: &mut [u8]) -> Result<()> {
    while !dst.is_empty() {
        let read = reader.read_at(offset, dst)?;
        if read == 0 {
            bail!("unexpected EOF while reading archive");
        }
        let (_, rest) = dst.split_at_mut(read);
        dst = rest;
        offset = offset.checked_add(read as u64).context("offset overflow")?;
    }
    Ok(())
}

fn dependencies_from_index_bytes(idx3_bytes: &[u8]) -> Option<Vec<FileDependencyV1>> {
    let index = decode_index(idx3_bytes).ok()?;
    let mut deps = Vec::new();
    for entry in index.entries {
        if entry.kind != EntryKind::Regular {
            continue;
        }
        deps.push(FileDependencyV1 {
            file_path: entry.path,
            required_blocks: entry.extents.into_iter().map(|e| e.block_id).collect(),
        });
    }
    Some(deps)
}

struct IndexSummary {
    preservation_profile: PreservationProfile,
    regular_file_count: u64,
    directory_count: u64,
    symlink_count: u64,
    hardlink_count: u64,
    sparse_file_count: u64,
    fifo_count: u64,
    char_device_count: u64,
    block_device_count: u64,
    extent_count: u64,
    logical_bytes: u64,
    has_modes: bool,
    has_mtime: bool,
    has_xattrs: bool,
    has_ownership: bool,
    has_hardlinks: bool,
    has_sparse: bool,
    has_special: bool,
    has_acls: bool,
    has_selinux: bool,
    has_capabilities: bool,
}

fn summarize_index(idx3_bytes: &[u8]) -> Option<IndexSummary> {
    let index = decode_index(idx3_bytes).ok()?;
    let profile = index.preservation_profile;
    let ownership_supported = profile == PreservationProfile::Full
        && (idx3_bytes.starts_with(IDX_MAGIC_V4)
            || idx3_bytes.starts_with(IDX_MAGIC_V5)
            || idx3_bytes.starts_with(IDX_MAGIC_V6)
            || idx3_bytes.starts_with(IDX_MAGIC_V7));
    let mut regular_file_count = 0u64;
    let mut directory_count = 0u64;
    let mut symlink_count = 0u64;
    let mut fifo_count = 0u64;
    let mut char_device_count = 0u64;
    let mut block_device_count = 0u64;
    let mut hardlink_count = 0u64;
    let mut sparse_file_count = 0u64;
    let mut extent_count = 0u64;
    let mut logical_bytes = 0u64;
    let mut has_xattrs = false;
    let mut has_hardlinks = false;
    let mut has_ownership = false;
    let mut has_sparse = false;
    let mut has_special = false;
    let mut has_acls = false;
    let mut has_selinux = false;
    let mut has_capabilities = false;

    for entry in index.entries {
        match entry.kind {
            EntryKind::Regular => regular_file_count = regular_file_count.saturating_add(1),
            EntryKind::Directory => directory_count = directory_count.saturating_add(1),
            EntryKind::Symlink => symlink_count = symlink_count.saturating_add(1),
            EntryKind::Fifo => fifo_count = fifo_count.saturating_add(1),
            EntryKind::CharDevice => char_device_count = char_device_count.saturating_add(1),
            EntryKind::BlockDevice => block_device_count = block_device_count.saturating_add(1),
        }
        has_xattrs |= !entry.xattrs.is_empty();
        if entry.hardlink_group_id.is_some() {
            has_hardlinks = true;
            hardlink_count = hardlink_count.saturating_add(1);
        }
        has_ownership |= ownership_supported;
        if entry.sparse {
            has_sparse = true;
            sparse_file_count = sparse_file_count.saturating_add(1);
        }
        has_special |= matches!(
            entry.kind,
            EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice
        );
        has_acls |= entry.acl_access.is_some() || entry.acl_default.is_some();
        has_selinux |= entry.selinux_label.is_some();
        has_capabilities |= entry.linux_capability.is_some();
        if entry.kind != EntryKind::Regular {
            continue;
        }
        extent_count += entry.extents.len() as u64;
        logical_bytes = logical_bytes.saturating_add(entry.size);
    }

    Some(IndexSummary {
        preservation_profile: profile,
        regular_file_count,
        directory_count,
        symlink_count,
        hardlink_count,
        sparse_file_count,
        fifo_count,
        char_device_count,
        block_device_count,
        extent_count,
        logical_bytes,
        has_modes: profile != PreservationProfile::PayloadOnly
            && (regular_file_count > 0 || directory_count > 0 || symlink_count > 0),
        has_mtime: profile != PreservationProfile::PayloadOnly
            && (regular_file_count > 0 || directory_count > 0 || symlink_count > 0),
        has_xattrs,
        has_ownership,
        has_hardlinks,
        has_sparse,
        has_special,
        has_acls,
        has_selinux,
        has_capabilities,
    })
}

#[derive(Debug)]
struct CompressionSummary {
    method: String,
    level: Option<String>,
}

#[derive(serde::Serialize)]
struct InfoJsonStructureSummary {
    extents: u64,
    dictionaries: u64,
    has_tail_frame: bool,
}

#[derive(serde::Serialize)]
struct InfoJsonVerificationSummary {
    extents_valid: bool,
    dictionaries_valid: bool,
    tail_frame_valid: bool,
}

#[derive(serde::Serialize)]
struct InfoJsonTruthSurface {
    format_version: String,
    global_flags: String,
    preservation_profile: String,
    total_entries: u64,
    structure: InfoJsonStructureSummary,
    verification: InfoJsonVerificationSummary,
    strict_extraction_supported: bool,
}

fn strict_extraction_supported(
    extents_valid: bool,
    dictionaries_valid: bool,
    tail_frame_valid: bool,
) -> bool {
    extents_valid && dictionaries_valid && tail_frame_valid
}

fn compression_summary_from_blocks<R: ReadAt + Len>(
    reader: &R,
    blocks_end_offset: u64,
) -> Result<Option<CompressionSummary>> {
    let blocks = scan_blocks_v1(reader, blocks_end_offset)?;
    if blocks.is_empty() {
        return Ok(None);
    }

    let mut codecs = BTreeSet::new();
    let mut levels = BTreeSet::new();
    for block in blocks {
        let mut header_prefix = [0u8; 6];
        read_exact_at(reader, block.header_offset, &mut header_prefix)?;
        if header_prefix[..4] != BLK3_MAGIC {
            bail!("invalid BLK3 magic while reading compression levels");
        }
        let header_len = u16::from_le_bytes([header_prefix[4], header_prefix[5]]) as usize;
        let mut header_bytes = vec![0u8; header_len];
        read_exact_at(reader, block.header_offset, &mut header_bytes)?;
        let header = read_blk3_header(Cursor::new(&header_bytes))
            .context("parse BLK3 header for compression levels")?;
        codecs.insert(header.codec);
        levels.insert(header.level);
    }

    let level = if levels.len() == 1 {
        levels.iter().next().map(|value| value.to_string())
    } else {
        Some(format!(
            "mixed ({})",
            levels
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    };
    let method = if codecs.len() == 1 {
        codec_name(*codecs.iter().next().unwrap_or(&0)).to_string()
    } else {
        format!(
            "mixed ({})",
            codecs
                .iter()
                .map(|codec| codec_name(*codec))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    Ok(Some(CompressionSummary { method, level }))
}

fn codec_name(codec: u32) -> &'static str {
    match codec {
        1 => "zstd",
        _ => "unknown",
    }
}

fn print_help() {
    let presenter = CliPresenter::new("crushr", "info", false);
    presenter.header();
    presenter.section("Usage");
    presenter.kv(
        "command",
        "crushr info <archive> [--json] [--list] [--flat] [--entry <path>] [--find <query>] [--find-mode substring] [--find-limit <n>] [--propagation]",
    );
    presenter.section("Flags");
    presenter.kv("--json", "emit machine-readable output");
    presenter.kv(
        "--list",
        "list metadata/index-proven contents without extraction",
    );
    presenter.kv("--flat", "list full paths (requires --list)");
    presenter.kv("--entry <path>", "inspect one logical entry by exact path");
    presenter.kv("--find <query>", "find logical entries by substring");
    presenter.kv(
        "--find-mode substring",
        "reserved search mode flag (substring supported)",
    );
    presenter.kv(
        "--find-limit <n>",
        "optional maximum result count for --find",
    );
    presenter.kv(
        "--propagation",
        "show propagation/dependency impact explanation",
    );
    presenter.kv("-h, --help", "print this help text");
    presenter.kv("--version, -V", "print version");
}

fn compression_level_display(level: Option<String>) -> String {
    level.unwrap_or_else(|| "unavailable".to_string())
}

fn compression_method_display(summary: Option<&CompressionSummary>) -> String {
    summary
        .map(|value| value.method.clone())
        .unwrap_or_else(|| "unavailable".to_string())
}

enum MetadataState {
    Present,
    NotPresent,
    OmittedByProfile,
}

impl MetadataState {
    fn as_display(&self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::NotPresent => "not present",
            Self::OmittedByProfile => "omitted by profile",
        }
    }
}

struct MetadataVisibilityFact {
    label: &'static str,
    full_supported: bool,
    basic_supported: bool,
    present: bool,
}

struct InfoTruthView {
    profile_contract: &'static str,
    archive_state_label: &'static str,
    metadata_rows: Vec<(&'static str, MetadataState)>,
    metadata_note: &'static str,
}

fn classify_metadata_state(
    profile: PreservationProfile,
    fact: &MetadataVisibilityFact,
) -> MetadataState {
    let included_by_profile = match profile {
        PreservationProfile::Full => fact.full_supported,
        PreservationProfile::Basic => fact.basic_supported,
        PreservationProfile::PayloadOnly => false,
    };
    if !included_by_profile {
        MetadataState::OmittedByProfile
    } else if fact.present {
        MetadataState::Present
    } else {
        MetadataState::NotPresent
    }
}

fn profile_contract_label(profile: PreservationProfile) -> &'static str {
    match profile {
        PreservationProfile::Full => "full-fidelity Linux-first",
        PreservationProfile::Basic => "basic Linux metadata",
        PreservationProfile::PayloadOnly => "content-oriented payload",
    }
}

fn classify_archive_state(summary: Option<&IndexSummary>) -> &'static str {
    if let Some(summary) = summary {
        if summary.extent_count == summary.regular_file_count {
            "file-level (1:1 file → unit)"
        } else {
            "file-level (see files vs file mappings)"
        }
    } else {
        "unavailable"
    }
}

fn build_info_truth_view(summary: Option<&IndexSummary>) -> InfoTruthView {
    let profile = summary
        .map(|value| value.preservation_profile)
        .unwrap_or(PreservationProfile::Full);
    let facts = [
        MetadataVisibilityFact {
            label: "modes",
            full_supported: true,
            basic_supported: true,
            present: summary.map(|s| s.has_modes).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "mtime",
            full_supported: true,
            basic_supported: true,
            present: summary.map(|s| s.has_mtime).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "xattrs",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_xattrs).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "ownership",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_ownership).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "hard links",
            full_supported: true,
            basic_supported: true,
            present: summary.map(|s| s.has_hardlinks).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "sparse files",
            full_supported: true,
            basic_supported: true,
            present: summary.map(|s| s.has_sparse).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "special files",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_special).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "ACLs",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_acls).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "SELinux labels",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_selinux).unwrap_or(false),
        },
        MetadataVisibilityFact {
            label: "capabilities",
            full_supported: true,
            basic_supported: false,
            present: summary.map(|s| s.has_capabilities).unwrap_or(false),
        },
    ];

    let metadata_rows = facts
        .iter()
        .map(|fact| (fact.label, classify_metadata_state(profile, fact)))
        .collect();

    InfoTruthView {
        profile_contract: profile_contract_label(profile),
        archive_state_label: classify_archive_state(summary),
        metadata_rows,
        metadata_note: "omitted by profile is intentional archive scope; metadata_degraded is an extraction outcome",
    }
}

fn propagation_report_with_structural_fallback<R: ReadAt + Len>(
    reader: &R,
) -> Result<PropagationReportV1> {
    let mut corrupted_structures = BTreeSet::new();
    let mut corrupted_blocks = BTreeSet::new();
    let mut file_dependencies = Vec::new();

    let archive_len = reader.len().context("read archive length")?;
    if archive_len < FTR4_LEN as u64 {
        let report = build_structural_failure_report_v1(&[
            STRUCTURE_FTR4,
            STRUCTURE_TAIL_FRAME,
            STRUCTURE_IDX3,
        ]);
        return Ok(report);
    }

    let footer_offset = archive_len - FTR4_LEN as u64;
    let mut footer_bytes = vec![0u8; FTR4_LEN];
    if read_exact_at(reader, footer_offset, &mut footer_bytes).is_err() {
        let report = build_structural_failure_report_v1(&[
            STRUCTURE_FTR4,
            STRUCTURE_TAIL_FRAME,
            STRUCTURE_IDX3,
        ]);
        return Ok(report);
    }

    let footer = match Ftr4::read_from(Cursor::new(&footer_bytes)) {
        Ok(value) => value,
        Err(_) => {
            let report = build_structural_failure_report_v1(&[
                STRUCTURE_FTR4,
                STRUCTURE_TAIL_FRAME,
                STRUCTURE_IDX3,
            ]);
            return Ok(report);
        }
    };

    let tail_frame_len = archive_len
        .checked_sub(footer.blocks_end_offset)
        .context("tail frame length underflow")?;
    let mut tail_frame_bytes = vec![0u8; tail_frame_len as usize];
    let tail_ok = read_exact_at(reader, footer.blocks_end_offset, &mut tail_frame_bytes)
        .ok()
        .and_then(|_| parse_tail_frame(&tail_frame_bytes).ok())
        .is_some();
    if !tail_ok {
        corrupted_structures.insert(STRUCTURE_TAIL_FRAME.to_string());
    }

    if footer.index_len == 0 || footer.index_offset.saturating_add(footer.index_len) > archive_len {
        corrupted_structures.insert(STRUCTURE_IDX3.to_string());
    } else {
        let mut idx3_bytes = vec![0u8; footer.index_len as usize];
        if read_exact_at(reader, footer.index_offset, &mut idx3_bytes).is_err() {
            corrupted_structures.insert(STRUCTURE_IDX3.to_string());
        } else {
            let hash_ok = *blake3::hash(&idx3_bytes).as_bytes() == footer.index_hash;
            let magic_ok = idx3_bytes.starts_with(IDX_MAGIC_V3)
                || idx3_bytes.starts_with(IDX_MAGIC_V4)
                || idx3_bytes.starts_with(IDX_MAGIC_V5)
                || idx3_bytes.starts_with(IDX_MAGIC_V6)
                || idx3_bytes.starts_with(IDX_MAGIC_V7);
            if !hash_ok || !magic_ok {
                corrupted_structures.insert(STRUCTURE_IDX3.to_string());
            }
            if let Some(deps) = dependencies_from_index_bytes(&idx3_bytes) {
                file_dependencies = deps;
            } else {
                corrupted_structures.insert(STRUCTURE_IDX3.to_string());
            }
        }
    }

    if footer.blocks_end_offset <= archive_len
        && let Ok(values) = verify_block_payloads_v1(reader, footer.blocks_end_offset)
    {
        corrupted_blocks = values;
    }

    Ok(build_propagation_report_v1(
        &file_dependencies,
        &corrupted_structures,
        &corrupted_blocks,
    ))
}

fn propagation_reason_str(reason: &PropagationDependencyReason) -> &'static str {
    match reason {
        PropagationDependencyReason::RequiresFooterReachability => "requires archive footer",
        PropagationDependencyReason::RequiresTailFrame => "requires tail frame",
        PropagationDependencyReason::RequiresIndex => "requires index",
        PropagationDependencyReason::RequiresMetadataMapping => "requires metadata mapping",
        PropagationDependencyReason::RequiresBlockPayload => "requires block payload",
    }
}

fn propagation_impact_reason_str(reason: &PropagationImpactReason) -> &'static str {
    match reason {
        PropagationImpactReason::CorruptedRequiredStructure => "corrupted required structure",
        PropagationImpactReason::CorruptedRequiredBlock => "corrupted required block",
    }
}

fn propagation_trust_class_str(trust: &PropagationEntryTrustClass) -> &'static str {
    match trust {
        PropagationEntryTrustClass::Canonical => "canonical",
        PropagationEntryTrustClass::MetadataDegraded => "metadata_degraded",
        PropagationEntryTrustClass::RecoveredNamed => "recovered_named",
        PropagationEntryTrustClass::RecoveredAnonymous => "recovered_anonymous",
        PropagationEntryTrustClass::Unrecoverable => "unrecoverable",
    }
}

fn propagation_impact_kind_str(kind: &ActivatedImpactKind) -> &'static str {
    match kind {
        ActivatedImpactKind::BlocksCanonicalExtraction => "blocks canonical extraction",
        ActivatedImpactKind::CausesMetadataDegraded => "causes metadata-degraded outcomes",
        ActivatedImpactKind::LeavesUnrecoverable => "can leave data unrecoverable",
    }
}

fn structure_label(node: &str) -> String {
    match node {
        STRUCTURE_FTR4 => "archive footer".to_string(),
        STRUCTURE_TAIL_FRAME => "tail frame".to_string(),
        STRUCTURE_IDX3 => "index".to_string(),
        _ => node.strip_prefix("structure:").unwrap_or(node).to_string(),
    }
}

const HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT: usize = 4;

fn format_human_dependency_lines(entry: &EntryImpactV1) -> Vec<String> {
    let shown = entry
        .dependencies
        .iter()
        .take(HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT)
        .map(|dep| {
            format!(
                "{} ({})",
                structure_label(&dep.node),
                propagation_reason_str(&dep.reason)
            )
        })
        .collect::<Vec<_>>();
    let remaining = entry
        .dependencies
        .len()
        .saturating_sub(HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT);
    if remaining == 0 {
        shown
    } else {
        shown
            .into_iter()
            .chain(std::iter::once(format!("+ {remaining} more")))
            .collect()
    }
}

fn format_entry_reasons(entry: &EntryImpactV1) -> String {
    let mut reasons = entry
        .activated_causes
        .iter()
        .map(|cause| propagation_impact_reason_str(&cause.reason))
        .collect::<Vec<_>>();
    reasons.sort_unstable();
    reasons.dedup();
    if reasons.is_empty() {
        "none".to_string()
    } else {
        reasons.join(", ")
    }
}

fn format_entry_consequences(entry: &EntryImpactV1) -> String {
    let mut out = Vec::new();
    if entry.canonical_blocked {
        out.push("blocks canonical extraction");
    }
    if entry
        .supported_trust_classes
        .iter()
        .any(|trust| matches!(trust, PropagationEntryTrustClass::MetadataDegraded))
    {
        out.push("can cause metadata-degraded outcomes");
    }
    if entry
        .supported_trust_classes
        .iter()
        .any(|trust| matches!(trust, PropagationEntryTrustClass::Unrecoverable))
    {
        out.push("can leave data unrecoverable");
    }
    if out.is_empty() {
        "none".to_string()
    } else {
        out.join(", ")
    }
}

fn print_propagation_human(archive: &str, report: &PropagationReportV1) {
    let presenter = CliPresenter::new("crushr", "propagation", false);
    presenter.header();

    presenter.section("Archive");
    presenter.kv("path", archive);
    presenter.kv("format family", &report.format_family);
    presenter.kv("report version", report.report_version);

    presenter.section("Detected corruption");
    presenter.kv(
        "corrupted structures",
        if report.detected_corruption.structure_nodes.is_empty() {
            "none".to_string()
        } else {
            report
                .detected_corruption
                .structure_nodes
                .iter()
                .map(|node| structure_label(node))
                .collect::<Vec<_>>()
                .join(", ")
        },
    );
    presenter.kv(
        "corrupted blocks",
        if report.detected_corruption.blocks.is_empty() {
            "none".to_string()
        } else {
            report
                .detected_corruption
                .blocks
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        },
    );

    let impacted_entries = report
        .entry_impacts
        .iter()
        .filter(|entry| !entry.activated_causes.is_empty())
        .collect::<Vec<_>>();
    let blocked_entries = impacted_entries
        .iter()
        .filter(|entry| entry.canonical_blocked)
        .count();

    presenter.section("Impact summary");
    presenter.kv_number("total entries", report.entry_impacts.len() as u64);
    presenter.kv_number("impacted entries", impacted_entries.len() as u64);
    presenter.kv_number("blocked canonical entries", blocked_entries as u64);
    presenter.kv_number(
        "activated impact causes",
        report.activated_impacts.len() as u64,
    );

    if !report.required_structures.is_empty() {
        presenter.section("Required structures");
        for required in &report.required_structures {
            presenter.kv(
                &structure_label(&required.structure_node),
                propagation_reason_str(&required.reason),
            );
        }
    }

    if !report.activated_impacts.is_empty() {
        presenter.section("Activated impacts");
        for impact in &report.activated_impacts {
            let detail = format!(
                "reason={} consequence={} affected_entries={}",
                propagation_impact_reason_str(&impact.reason),
                propagation_impact_kind_str(&impact.impact_kind),
                impact.affected_entries.len()
            );
            presenter.kv(&structure_label(&impact.cause_node), detail);
        }
    }

    presenter.section("Entry impacts");
    if impacted_entries.is_empty() {
        presenter.info_note("no currently activated entry impact from detected corruption");
    } else {
        for entry in impacted_entries {
            presenter.kv("entry", &entry.file_path);
            presenter.kv("reasons", format_entry_reasons(entry));
            presenter.kv("consequence", format_entry_consequences(entry));
            presenter.kv(
                "canonical extraction blocked",
                if entry.canonical_blocked {
                    "true"
                } else {
                    "false"
                },
            );
            let trust = entry
                .supported_trust_classes
                .iter()
                .map(propagation_trust_class_str)
                .collect::<Vec<_>>()
                .join(", ");
            presenter.kv("supported trust classes", trust);
            let dependency_lines = format_human_dependency_lines(entry);
            if dependency_lines.is_empty() {
                presenter.kv("dependencies", "none");
            } else {
                presenter.kv("dependencies", dependency_lines[0].as_str());
                for line in dependency_lines.iter().skip(1) {
                    presenter.kv("", line.as_str());
                }
            }
            println!();
        }
    }

    presenter.result_summary(
        StatusWord::Complete,
        "propagation impact inspection completed",
        &[],
    );
}

#[cfg(test)]
mod tests {
    use super::{HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT, format_human_dependency_lines};
    use crushr_core::propagation::{
        DependencyLinkKind, EntryDependencyV1, EntryImpactV1, EntryTrustClass, FileImpactCause,
        PropagationDependencyReason, PropagationImpactReason,
    };

    fn sample_entry_with_dependencies(count: usize) -> EntryImpactV1 {
        EntryImpactV1 {
            file_path: "alpha.txt".to_string(),
            dependencies: (0..count)
                .map(|idx| EntryDependencyV1 {
                    node: format!("BLK3[{idx}]"),
                    link_kind: DependencyLinkKind::Direct,
                    reason: PropagationDependencyReason::RequiresBlockPayload,
                })
                .collect(),
            activated_causes: vec![FileImpactCause {
                cause_node: "BLK3[0]".to_string(),
                reason: PropagationImpactReason::CorruptedRequiredBlock,
            }],
            canonical_blocked: true,
            canonical_blocked_reasons: vec![PropagationImpactReason::CorruptedRequiredBlock],
            supported_trust_classes: vec![EntryTrustClass::Unrecoverable],
        }
    }

    #[test]
    fn human_dependency_lines_show_remainder_count_when_summarized() {
        let entry = sample_entry_with_dependencies(HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT + 3);
        let lines = format_human_dependency_lines(&entry);
        assert_eq!(lines.len(), HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT + 1);
        assert_eq!(lines.last().expect("last"), "+ 3 more");
    }

    #[test]
    fn human_dependency_lines_are_unsummarized_at_or_below_limit() {
        let entry = sample_entry_with_dependencies(HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT);
        let lines = format_human_dependency_lines(&entry);
        assert_eq!(lines.len(), HUMAN_PROPAGATION_DEPENDENCY_VISIBLE_LIMIT);
        assert!(!lines.iter().any(|line| line.starts_with("+ ")));
    }
}

#[derive(Default)]
struct TreeNode {
    dirs: BTreeMap<String, TreeNode>,
    files: BTreeSet<String>,
}

fn split_path_components(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect()
}

fn insert_tree_path(tree: &mut TreeNode, path: &str) {
    let components = split_path_components(path);
    if components.is_empty() {
        return;
    }
    let mut current = tree;
    for part in &components[..components.len().saturating_sub(1)] {
        current = current.dirs.entry((*part).to_string()).or_default();
    }
    if let Some(last) = components.last() {
        current.files.insert((*last).to_string());
    }
}

fn render_tree_lines(tree: &TreeNode) -> Vec<String> {
    let mut lines = Vec::new();
    let mut roots = Vec::new();
    roots.extend(tree.dirs.keys().map(|name| (name.clone(), true)));
    roots.extend(tree.files.iter().map(|name| (name.clone(), false)));

    for (idx, (name, is_dir)) in roots.iter().enumerate() {
        let is_last = idx + 1 == roots.len();
        let connector = if is_last { "└──" } else { "├──" };
        if *is_dir {
            lines.push(format!("{connector} {name}/"));
            if let Some(child) = tree.dirs.get(name) {
                render_tree_children(child, &mut lines, if is_last { "    " } else { "│   " });
            }
        } else {
            lines.push(format!("{connector} {name}"));
        }
    }

    lines
}

fn render_tree_children(tree: &TreeNode, lines: &mut Vec<String>, prefix: &str) {
    let mut items = Vec::new();
    items.extend(tree.dirs.keys().map(|name| (name.clone(), true)));
    items.extend(tree.files.iter().map(|name| (name.clone(), false)));

    for (idx, (name, is_dir)) in items.iter().enumerate() {
        let is_last = idx + 1 == items.len();
        let connector = if is_last { "└──" } else { "├──" };
        if *is_dir {
            lines.push(format!("{prefix}{connector} {name}/"));
            if let Some(child) = tree.dirs.get(name) {
                let next_prefix = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                render_tree_children(child, lines, &next_prefix);
            }
        } else {
            lines.push(format!("{prefix}{connector} {name}"));
        }
    }
}

fn build_flat_listing(paths: &[String]) -> Vec<String> {
    let mut directories = BTreeSet::new();
    for path in paths {
        let components = split_path_components(path);
        for idx in 1..components.len() {
            directories.insert(format!("{}/", components[..idx].join("/")));
        }
    }

    let mut out = Vec::new();
    out.extend(directories);
    out.extend(paths.iter().cloned());
    out
}

struct ListingLoad {
    paths: Vec<String>,
    preservation_profile: PreservationProfile,
    non_regular_entries: u64,
    omitted_non_regular_entries: u64,
    warnings: Vec<String>,
    degraded: bool,
}

struct ListingTruthView {
    status: StatusWord,
    result_message: &'static str,
}

#[allow(dead_code)]
#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum EntryTrustClass {
    Canonical,
    MetadataDegraded,
    RecoveredNamed,
    RecoveredAnonymous,
    Unrecoverable,
}

impl EntryTrustClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::MetadataDegraded => "metadata_degraded",
            Self::RecoveredNamed => "recovered_named",
            Self::RecoveredAnonymous => "recovered_anonymous",
            Self::Unrecoverable => "unrecoverable",
        }
    }
}

#[derive(Clone)]
struct EntryIntrospectionRecord {
    path: String,
    trust_class: EntryTrustClass,
    payload_verified: bool,
    metadata_complete: bool,
    extent_count: u64,
    size_bytes: u64,
    payload_blake3: String,
    logical_range: EntryLogicalRange,
    identity_source: String,
    reason: Option<String>,
    strict_extraction_supported: bool,
}

#[derive(Clone, serde::Serialize)]
struct EntryLogicalRange {
    start: u64,
    end: u64,
}

#[derive(serde::Serialize)]
struct EntryIntrospectionJson {
    path: String,
    trust_class: EntryTrustClass,
    payload_verified: bool,
    metadata_complete: bool,
    extent_count: u64,
    size_bytes: u64,
    payload_blake3: String,
    logical_range: EntryLogicalRange,
    identity_source: String,
    reason: Option<String>,
    strict_extraction_supported: bool,
}

#[derive(serde::Serialize)]
struct EntryLookupNotFoundJson {
    found: bool,
    path: String,
}

#[derive(serde::Serialize)]
struct EntryFindJsonRow {
    path: String,
    trust_class: EntryTrustClass,
}

fn entry_records_from_index_bytes<R: ReadAt + Len>(
    reader: &R,
    idx3_bytes: &[u8],
    degraded: bool,
) -> Result<Vec<EntryIntrospectionRecord>> {
    let index = decode_index(idx3_bytes).context("decode IDX3 index")?;
    let payload_validity = if degraded {
        None
    } else {
        let opened = open_archive_v1(reader)?;
        let invalid_blocks =
            verify_block_payloads_v1(reader, opened.tail.footer.blocks_end_offset)?;
        Some(invalid_blocks)
    };
    let trust_class = if degraded {
        EntryTrustClass::MetadataDegraded
    } else {
        EntryTrustClass::Canonical
    };
    let reason = if degraded {
        Some(
            "archive has structural damage outside IDX3; entry evidence is index-proven only"
                .to_string(),
        )
    } else {
        None
    };
    let mut records = Vec::new();
    for entry in index.entries {
        let logical_range = logical_range_from_extents(entry.size, entry.sparse, &entry.extents);
        let payload_verified = if entry.kind == EntryKind::Regular {
            payload_validity.as_ref().is_some_and(|bad_blocks| {
                entry
                    .extents
                    .iter()
                    .all(|extent| !bad_blocks.contains(&extent.block_id))
            })
        } else {
            true
        };
        let metadata_complete = !degraded;
        let strict_extraction_supported = payload_verified && metadata_complete;
        let identity_source = if degraded {
            "idx3_fallback".to_string()
        } else {
            "canonical_index".to_string()
        };
        let payload_blake3 = entry_payload_blake3(reader, &entry, degraded)
            .unwrap_or_else(|| "unavailable".to_string());
        records.push(EntryIntrospectionRecord {
            path: entry.path.clone(),
            trust_class,
            payload_verified,
            metadata_complete,
            extent_count: entry.extents.len() as u64,
            size_bytes: entry.size,
            payload_blake3,
            logical_range,
            identity_source,
            reason: reason.clone(),
            strict_extraction_supported,
        });
    }
    records.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(records)
}

fn logical_range_from_extents(
    size_bytes: u64,
    sparse: bool,
    extents: &[Extent],
) -> EntryLogicalRange {
    if extents.is_empty() {
        return EntryLogicalRange { start: 0, end: 0 };
    }
    let start = if sparse {
        extents
            .iter()
            .map(|extent| extent.logical_offset)
            .min()
            .unwrap_or(0)
    } else {
        0
    };
    let end = if sparse {
        extents
            .iter()
            .filter_map(|extent| extent.logical_offset.checked_add(extent.len))
            .max()
            .unwrap_or(size_bytes)
    } else {
        size_bytes
    };
    EntryLogicalRange { start, end }
}

fn entry_payload_blake3<R: ReadAt + Len>(
    reader: &R,
    entry: &Entry,
    degraded: bool,
) -> Option<String> {
    if degraded || entry.kind != EntryKind::Regular {
        return None;
    }
    let opened = open_archive_v1(reader).ok()?;
    let blocks = scan_blocks_v1(reader, opened.tail.footer.blocks_end_offset).ok()?;
    let bytes = read_entry_bytes(reader, entry, &blocks).ok()?;
    Some(blake3::hash(&bytes).to_hex().to_string())
}

fn load_entry_records<R: ReadAt + Len>(reader: &R) -> Result<Vec<EntryIntrospectionRecord>> {
    match open_archive_v1(reader) {
        Ok(opened) => entry_records_from_index_bytes(reader, &opened.tail.idx3_bytes, false),
        Err(_) => {
            let idx3_bytes = read_idx3_bytes_from_footer(reader)?;
            entry_records_from_index_bytes(reader, &idx3_bytes, true)
        }
    }
}

fn build_listing_truth_view(listing: &ListingLoad) -> ListingTruthView {
    if listing.degraded {
        ListingTruthView {
            status: StatusWord::Degraded,
            result_message: "content listing completed with degraded coverage",
        }
    } else {
        ListingTruthView {
            status: StatusWord::Complete,
            result_message: "content listing completed",
        }
    }
}

fn listing_paths_from_index_bytes(idx3_bytes: &[u8]) -> Result<(Vec<String>, u64)> {
    let index = decode_index(idx3_bytes).context("decode IDX3 index")?;
    let mut paths = Vec::new();
    let mut omitted_non_regular_entries = 0u64;
    for entry in index.entries {
        if entry.kind == EntryKind::Regular {
            paths.push(entry.path);
        } else {
            omitted_non_regular_entries = omitted_non_regular_entries.saturating_add(1);
        }
    }
    paths.sort();
    Ok((paths, omitted_non_regular_entries))
}

fn read_idx3_bytes_from_footer<R: ReadAt + Len>(reader: &R) -> Result<Vec<u8>> {
    let archive_len = reader.len().context("read archive length")?;
    if archive_len < FTR4_LEN as u64 {
        bail!("archive too small for FTR4 footer");
    }

    let footer_offset = archive_len - FTR4_LEN as u64;
    let mut footer_bytes = vec![0u8; FTR4_LEN];
    read_exact_at(reader, footer_offset, &mut footer_bytes)
        .context("read FTR4 footer for index listing")?;
    let footer = Ftr4::read_from(Cursor::new(&footer_bytes)).context("parse FTR4 footer")?;

    if footer.index_len == 0 {
        bail!("index length is zero");
    }
    if footer.index_offset.saturating_add(footer.index_len) > archive_len {
        bail!("index range exceeds archive length");
    }

    let mut idx3_bytes = vec![0u8; footer.index_len as usize];
    read_exact_at(reader, footer.index_offset, &mut idx3_bytes)
        .context("read IDX3 index bytes for listing")?;

    if !idx3_bytes.starts_with(IDX_MAGIC_V3)
        && !idx3_bytes.starts_with(IDX_MAGIC_V4)
        && !idx3_bytes.starts_with(IDX_MAGIC_V5)
        && !idx3_bytes.starts_with(IDX_MAGIC_V6)
        && !idx3_bytes.starts_with(IDX_MAGIC_V7)
    {
        bail!("IDX magic mismatch");
    }
    if *blake3::hash(&idx3_bytes).as_bytes() != footer.index_hash {
        bail!("IDX hash mismatch");
    }

    Ok(idx3_bytes)
}

fn load_listing_paths<R: ReadAt + Len>(reader: &R) -> Result<ListingLoad> {
    match open_archive_v1(reader) {
        Ok(opened) => {
            let (paths, omitted_non_regular_entries) =
                listing_paths_from_index_bytes(&opened.tail.idx3_bytes)?;
            Ok(ListingLoad {
                paths,
                preservation_profile: decode_index(&opened.tail.idx3_bytes)
                    .map(|index| index.preservation_profile)
                    .unwrap_or(PreservationProfile::Full),
                non_regular_entries: omitted_non_regular_entries,
                omitted_non_regular_entries,
                warnings: Vec::new(),
                degraded: false,
            })
        }
        Err(open_err) => {
            let idx3_bytes = match read_idx3_bytes_from_footer(reader) {
                Ok(value) => value,
                Err(idx_err) => {
                    return Ok(ListingLoad {
                        paths: Vec::new(),
                        preservation_profile: PreservationProfile::Full,
                        non_regular_entries: 0,
                        omitted_non_regular_entries: 0,
                        warnings: vec![
                            format!(
                                "archive structure is degraded; listing unavailable ({open_err:#})"
                            ),
                            format!("IDX3 could not be proven for listing ({idx_err:#})"),
                            "for recovery-oriented evidence, run `crushr extract --recover <archive>`"
                                .to_string(),
                        ],
                        degraded: true,
                    });
                }
            };
            let (paths, omitted_non_regular_entries) = listing_paths_from_index_bytes(&idx3_bytes)?;
            let warnings = vec![
                "archive has structural damage outside IDX3; listing shows only CANONICAL index-proven paths".to_string(),
            ];
            let preservation_profile = decode_index(&idx3_bytes)
                .map(|index| index.preservation_profile)
                .unwrap_or(PreservationProfile::Full);

            Ok(ListingLoad {
                paths,
                preservation_profile,
                non_regular_entries: omitted_non_regular_entries,
                omitted_non_regular_entries,
                warnings,
                degraded: true,
            })
        }
    }
}

fn run(raw_args: Vec<String>) -> Result<()> {
    let early_args = raw_args.clone();
    if matches!(
        early_args.first().map(String::as_str),
        Some("--help" | "-h")
    ) {
        print_help();
        return Ok(());
    }
    if matches!(
        early_args.first().map(String::as_str),
        Some("--version" | "-V")
    ) {
        println!("{}", crate::product_version());
        return Ok(());
    }

    let mut archive = None;
    let mut json = false;
    let mut propagation = false;
    let mut list = false;
    let mut flat = false;
    let mut entry = None;
    let mut find = None;
    let mut find_mode = "substring".to_string();
    let mut find_limit = None;

    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--json" {
            json = true;
        } else if arg == "--list" {
            list = true;
        } else if arg == "--flat" {
            flat = true;
        } else if arg == "--propagation" {
            propagation = true;
        } else if arg == "--report" {
            bail!("--report is retired; use --propagation");
        } else if arg == "--entry" {
            entry = Some(args.next().context("missing value for --entry")?);
        } else if arg == "--find" {
            find = Some(args.next().context("missing value for --find")?);
        } else if arg == "--find-mode" {
            find_mode = args.next().context("missing value for --find-mode")?;
        } else if arg == "--find-limit" {
            let raw_limit = args.next().context("missing value for --find-limit")?;
            find_limit = Some(
                raw_limit
                    .parse::<usize>()
                    .context("invalid value for --find-limit (expected integer)")?,
            );
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(arg);
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    if flat && !list {
        bail!("--flat requires --list");
    }
    if list && json {
        bail!("--json cannot be combined with --list");
    }
    if list && propagation {
        bail!("--propagation cannot be combined with --list");
    }
    if list && entry.is_some() {
        bail!("--entry cannot be combined with --list");
    }
    if list && find.is_some() {
        bail!("--find cannot be combined with --list");
    }
    if propagation && entry.is_some() {
        bail!("--entry cannot be combined with --propagation");
    }
    if propagation && find.is_some() {
        bail!("--find cannot be combined with --propagation");
    }
    if entry.is_some() && find.is_some() {
        bail!("--entry cannot be combined with --find");
    }
    if find.is_none() && (find_limit.is_some() || find_mode != "substring") {
        bail!("--find-mode/--find-limit require --find");
    }
    if find_mode != "substring" {
        bail!("unsupported find mode: {find_mode} (expected substring)");
    }

    let archive = archive.context(
        "usage: crushr info <archive> [--json] [--list] [--flat] [--entry <path>] [--find <query>] [--find-mode substring] [--find-limit <n>] [--propagation]",
    )?;

    let reader = FileReader {
        file: File::open(&archive).with_context(|| format!("open {archive}"))?,
    };

    if propagation {
        let report = propagation_report_with_structural_fallback(&reader)?;
        if json {
            println!("{}", serialize_snapshot_json(&report)?);
        } else {
            print_propagation_human(&archive, &report);
        }
        return Ok(());
    }

    if list {
        let listing = load_listing_paths(&reader)?;
        let listing_truth = build_listing_truth_view(&listing);
        let presenter = CliPresenter::new("crushr", "list", false);
        presenter.header();
        presenter.section("Archive");
        presenter.kv("path", &archive);
        presenter.kv("mode", if flat { "flat" } else { "tree" });
        presenter.kv("profile", listing.preservation_profile.as_str());
        presenter.kv("scope", "regular files (metadata/index proven)");

        presenter.section("Contents");
        if listing.paths.is_empty() {
            println!("  (no provable paths)");
        } else if flat {
            for path in build_flat_listing(&listing.paths) {
                println!("  {path}");
            }
        } else {
            let mut tree = TreeNode::default();
            for path in &listing.paths {
                insert_tree_path(&mut tree, path);
            }
            for line in render_tree_lines(&tree) {
                println!("  {line}");
            }
        }

        for warning in &listing.warnings {
            presenter.banner(BannerLevel::Warning, warning);
        }

        if listing.non_regular_entries > 0 {
            presenter.info_note(&format!(
                "{} non-regular index entries are outside --list scope",
                group_u64(listing.non_regular_entries)
            ));
        }
        if listing.omitted_non_regular_entries > 0 {
            presenter.info_note(&format!(
                "omitted {} non-regular index entries",
                group_u64(listing.omitted_non_regular_entries)
            ));
        }

        let mut rows = vec![("listed files", group_u64(listing.paths.len() as u64))];
        if listing.omitted_non_regular_entries > 0 {
            rows.push((
                "omitted entries",
                group_u64(listing.omitted_non_regular_entries),
            ));
        }

        presenter.result_summary(listing_truth.status, listing_truth.result_message, &rows);

        return Ok(());
    }

    if let Some(entry_path) = entry {
        let records = load_entry_records(&reader)?;
        if let Some(record) = records
            .iter()
            .find(|candidate| candidate.path == entry_path)
        {
            if json {
                let row = EntryIntrospectionJson {
                    path: record.path.clone(),
                    trust_class: record.trust_class,
                    payload_verified: record.payload_verified,
                    metadata_complete: record.metadata_complete,
                    extent_count: record.extent_count,
                    size_bytes: record.size_bytes,
                    payload_blake3: record.payload_blake3.clone(),
                    logical_range: record.logical_range.clone(),
                    identity_source: record.identity_source.clone(),
                    reason: record.reason.clone(),
                    strict_extraction_supported: record.strict_extraction_supported,
                };
                println!("{}", serialize_snapshot_json(&row)?);
                return Ok(());
            }

            let presenter = CliPresenter::new("crushr", "entry", false);
            presenter.header();
            presenter.section("Archive");
            presenter.kv("path", &archive);
            presenter.section("Entry");
            presenter.kv("logical path", &record.path);
            presenter.kv("trust class", record.trust_class.as_str());
            presenter.kv(
                "payload verified",
                if record.payload_verified {
                    "true"
                } else {
                    "false"
                },
            );
            presenter.kv(
                "metadata complete",
                if record.metadata_complete {
                    "true"
                } else {
                    "false"
                },
            );
            presenter.kv("extent count", group_u64(record.extent_count));
            presenter.kv("size bytes", group_u64(record.size_bytes));
            presenter.kv("payload blake3", &record.payload_blake3);
            presenter.kv(
                "logical range",
                format!(
                    "0x{:016x}..0x{:016x} ({}..{})",
                    record.logical_range.start,
                    record.logical_range.end,
                    group_u64(record.logical_range.start),
                    group_u64(record.logical_range.end)
                ),
            );
            presenter.kv("identity source", &record.identity_source);
            presenter.kv(
                "strict extraction supported",
                if record.strict_extraction_supported {
                    "true"
                } else {
                    "false"
                },
            );
            if let Some(reason) = &record.reason {
                presenter.kv("reason", reason);
            }
            presenter.result_summary(StatusWord::Complete, "entry inspection completed", &[]);
            return Ok(());
        }

        if json {
            let not_found = EntryLookupNotFoundJson {
                found: false,
                path: entry_path,
            };
            println!("{}", serialize_snapshot_json(&not_found)?);
            return Ok(());
        }

        let presenter = CliPresenter::new("crushr", "entry", false);
        presenter.header();
        presenter.section("Archive");
        presenter.kv("path", &archive);
        presenter.section("Entry");
        presenter.kv("logical path", &entry_path);
        presenter.result_summary(StatusWord::Degraded, "entry not found", &[]);
        return Ok(());
    }

    if let Some(query) = find {
        let mut matches: Vec<EntryFindJsonRow> = load_entry_records(&reader)?
            .into_iter()
            .filter(|record| {
                matches!(
                    record.trust_class,
                    EntryTrustClass::Canonical
                        | EntryTrustClass::MetadataDegraded
                        | EntryTrustClass::RecoveredNamed
                ) && record.path.contains(&query)
            })
            .map(|record| EntryFindJsonRow {
                path: record.path,
                trust_class: record.trust_class,
            })
            .collect();
        matches.sort_by(|a, b| a.path.cmp(&b.path));
        if let Some(limit) = find_limit {
            matches.truncate(limit);
        }

        if json {
            println!("{}", serialize_snapshot_json(&matches)?);
            return Ok(());
        }

        let presenter = CliPresenter::new("crushr", "find", false);
        presenter.header();
        presenter.section("Archive");
        presenter.kv("path", &archive);
        presenter.kv("query", &query);
        presenter.kv("mode", &find_mode);
        presenter.section("Matches");
        if matches.is_empty() {
            println!("  (no matching entries)");
            presenter.result_summary(StatusWord::Degraded, "no matches", &[]);
        } else {
            for entry in &matches {
                println!("  {:<48} {}", entry.path, entry.trust_class.as_str());
            }
            presenter.result_summary(StatusWord::Complete, "search completed", &[]);
        }
        return Ok(());
    }

    let opened = open_archive_v1(&reader)?;
    let snapshot =
        info_envelope_from_open_archive(&opened, crate::product_version(), "1970-01-01T00:00:00Z");

    let index_summary = summarize_index(&opened.tail.idx3_bytes);
    let extent_count = index_summary.as_ref().map(|s| s.extent_count).unwrap_or(0);
    let total_entries = index_summary
        .as_ref()
        .map(|s| {
            s.regular_file_count
                .saturating_add(s.directory_count)
                .saturating_add(s.symlink_count)
                .saturating_add(s.fifo_count)
                .saturating_add(s.char_device_count)
                .saturating_add(s.block_device_count)
        })
        .unwrap_or(0);
    let extents_valid = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)
        .map(|bad| bad.is_empty())
        .unwrap_or(false);
    let dictionaries_valid = opened.tail.dct1.is_some() || !snapshot.payload.summary.has_dct1;
    let tail_frame_valid = !snapshot.payload.tail_frames.is_empty();
    let strict_supported =
        strict_extraction_supported(extents_valid, dictionaries_valid, tail_frame_valid);

    if json {
        let format_version = if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V7) {
            "v7"
        } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V6) {
            "v6"
        } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V5) {
            "v5"
        } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V4) {
            "v4"
        } else {
            "v3"
        };
        let truth = InfoJsonTruthSurface {
            format_version: format_version.to_string(),
            global_flags: "none".to_string(),
            preservation_profile: index_summary
                .as_ref()
                .map(|summary| summary.preservation_profile.as_str().to_string())
                .unwrap_or_else(|| PreservationProfile::Full.as_str().to_string()),
            total_entries,
            structure: InfoJsonStructureSummary {
                extents: extent_count,
                dictionaries: snapshot.payload.dicts.count as u64,
                has_tail_frame: !snapshot.payload.tail_frames.is_empty(),
            },
            verification: InfoJsonVerificationSummary {
                extents_valid,
                dictionaries_valid,
                tail_frame_valid,
            },
            strict_extraction_supported: strict_supported,
        };
        println!("{}", serialize_snapshot_json(&truth)?);
        return Ok(());
    }

    let archive_blake3 = snapshot.archive_fingerprint.0.clone();

    let presenter = CliPresenter::new("crushr", "info", false);
    presenter.header();
    presenter.section("Archive");
    presenter.kv("path", &archive);
    presenter.kv(
        "size bytes",
        group_u64(snapshot.payload.summary.archive_len),
    );
    presenter.kv("blake3", archive_blake3);
    let idx_marker = if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V7) {
        "IDX7"
    } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V6) {
        "IDX6"
    } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V5) {
        "IDX5"
    } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V4) {
        "IDX4"
    } else if opened.tail.idx3_bytes.starts_with(IDX_MAGIC_V3) {
        "IDX3"
    } else {
        "IDX?"
    };
    presenter.kv("format markers", format!("FTR4 + {idx_marker}"));
    presenter.kv("global flags", "none");
    let info_truth = build_info_truth_view(index_summary.as_ref());
    presenter.section("Preservation");
    presenter.kv(
        "profile",
        index_summary
            .as_ref()
            .map(|summary| summary.preservation_profile.as_str())
            .unwrap_or(PreservationProfile::Full.as_str()),
    );
    presenter.kv("contract", info_truth.profile_contract);

    presenter.section("Structure");
    presenter.kv(
        "files",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.regular_file_count),
        ),
    );
    let payload_units = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)
        .map(|blocks| group_u64(blocks.len() as u64))
        .unwrap_or_else(|_| "unavailable".to_string());
    presenter.kv("compressed units", payload_units);
    presenter.kv(
        "file mappings",
        index_summary
            .as_ref()
            .map_or_else(|| "unavailable".to_string(), |s| group_u64(s.extent_count)),
    );
    presenter.kv("block model", info_truth.archive_state_label);
    presenter.kv(
        "logical bytes",
        index_summary
            .as_ref()
            .map_or_else(|| "unavailable".to_string(), |s| group_u64(s.logical_bytes)),
    );
    presenter.kv("has footer", snapshot.payload.summary.has_footer);
    presenter.kv(
        "tail frames",
        group_u64(snapshot.payload.tail_frames.len() as u64),
    );
    let dictionary_summary = if snapshot.payload.summary.has_dct1 {
        format!(
            "present ({} entries)",
            group_u64(snapshot.payload.dicts.count as u64)
        )
    } else {
        "not present".to_string()
    };
    presenter.kv("dictionary table", dictionary_summary);
    presenter.kv(
        "dictionary ledger",
        if snapshot.payload.summary.has_ldg1 {
            "present"
        } else {
            "not present"
        },
    );
    presenter.section("Entry kinds");
    presenter.kv(
        "regular files",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.regular_file_count),
        ),
    );
    presenter.kv(
        "directories",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.directory_count),
        ),
    );
    presenter.kv(
        "symlinks",
        index_summary
            .as_ref()
            .map_or_else(|| "unavailable".to_string(), |s| group_u64(s.symlink_count)),
    );
    presenter.kv(
        "hard links",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.hardlink_count),
        ),
    );
    presenter.kv(
        "sparse files",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.sparse_file_count),
        ),
    );
    presenter.kv(
        "FIFOs",
        index_summary
            .as_ref()
            .map_or_else(|| "unavailable".to_string(), |s| group_u64(s.fifo_count)),
    );
    presenter.kv(
        "char devices",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.char_device_count),
        ),
    );
    presenter.kv(
        "block devices",
        index_summary.as_ref().map_or_else(
            || "unavailable".to_string(),
            |s| group_u64(s.block_device_count),
        ),
    );
    presenter.section("Metadata");
    for (label, state) in &info_truth.metadata_rows {
        presenter.kv(label, state.as_display());
    }
    presenter.info_note(info_truth.metadata_note);
    let compression =
        compression_summary_from_blocks(&reader, opened.tail.footer.blocks_end_offset)
            .ok()
            .flatten();
    presenter.section("Compression");
    presenter.kv("method", compression_method_display(compression.as_ref()));
    presenter.kv(
        "level",
        compression_level_display(compression.and_then(|summary| summary.level)),
    );

    presenter.section("Verification");
    presenter.kv(
        "extent verification",
        if extents_valid { "valid" } else { "invalid" },
    );
    presenter.kv(
        "dictionary validity",
        if dictionaries_valid {
            "valid"
        } else {
            "invalid"
        },
    );
    presenter.kv(
        "tail-frame validity",
        if tail_frame_valid { "valid" } else { "invalid" },
    );

    presenter.section("Extraction viability");
    presenter.kv(
        "strict_extraction_supported",
        if strict_supported { "true" } else { "false" },
    );
    if !strict_supported {
        presenter
            .info_note("strict extraction is not supported; recovery is required for any output");
    }

    presenter.result_summary(StatusWord::Complete, "archive inspection completed", &[]);
    Ok(())
}

pub fn dispatch(args: Vec<String>) -> i32 {
    match run(args) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err:#}");
            let msg = format!("{err:#}");
            if msg.contains("usage:")
                || msg.contains("missing value for --entry")
                || msg.contains("missing value for --find")
                || msg.contains("missing value for --find-mode")
                || msg.contains("missing value for --find-limit")
                || msg.contains("--report is retired")
                || msg.contains("unsupported find mode")
                || msg.contains("unsupported flag")
                || msg.contains("invalid value for --find-limit")
                || msg.contains("unexpected argument")
                || msg.contains("--flat requires --list")
                || msg.contains("--json cannot be combined with --list")
                || msg.contains("--propagation cannot be combined with --list")
                || msg.contains("--entry cannot be combined with --list")
                || msg.contains("--find cannot be combined with --list")
                || msg.contains("--entry cannot be combined with --propagation")
                || msg.contains("--find cannot be combined with --propagation")
                || msg.contains("--entry cannot be combined with --find")
                || msg.contains("--find-mode/--find-limit require --find")
            {
                1
            } else {
                2
            }
        }
    }
}

pub fn dispatch_from_env() -> i32 {
    dispatch(std::env::args().skip(1).collect())
}
