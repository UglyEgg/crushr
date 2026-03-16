use anyhow::{bail, Context, Result};
use crushr::format::{EntryKind, Extent, IDX_MAGIC_V3};
use crushr::index_codec::decode_index;
use crushr_core::{
    io::{Len, ReadAt},
    verify::scan_blocks_v1,
};
use crushr_format::{
    blk3::{read_blk3_header, BLK3_MAGIC},
    ftr4::{Ftr4, FTR4_LEN},
    tailframe::parse_tail_frame,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const USAGE: &str =
    "usage: crushr-salvage <archive> [--json-out <path>] [--export-fragments <dir>]";
const RESEARCH_LABEL: &str = "UNVERIFIED_RESEARCH_OUTPUT";
const PRIMARY_INDEX_PATH: &str = "PRIMARY_INDEX_PATH";
const REDUNDANT_VERIFIED_MAP_PATH: &str = "REDUNDANT_VERIFIED_MAP_PATH";
const CHECKPOINT_MAP_PATH: &str = "CHECKPOINT_MAP_PATH";
const SELF_DESCRIBING_EXTENT_PATH: &str = "SELF_DESCRIBING_EXTENT_PATH";
const FILE_IDENTITY_EXTENT_PATH: &str = "FILE_IDENTITY_EXTENT_PATH";
const FILE_IDENTITY_EXTENT_PATH_ANONYMOUS: &str = "FILE_IDENTITY_EXTENT_PATH_ANONYMOUS";
const PAYLOAD_BLOCK_IDENTITY_PATH: &str = "PAYLOAD_BLOCK_IDENTITY_PATH";
const PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS: &str = "PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS";

// Internal responsibility split for safer iterative salvage changes.
// cli => argument parsing, discovery => block scan/verification,
// metadata => metadata decode + planning, artifacts => output/export helpers.
#[path = "crushr_salvage/artifacts.rs"]
mod artifacts;
#[path = "crushr_salvage/cli.rs"]
mod cli;
#[path = "crushr_salvage/discovery.rs"]
mod discovery;
#[path = "crushr_salvage/metadata.rs"]
mod metadata;

use artifacts::{export_artifacts, write_json_output};
use cli::parse_cli_options;
use discovery::{build_block_verification, classify_file, scan_blk3_candidates, to_hex};
use metadata::{
    parse_checkpoint_extent_records, parse_experimental_metadata_records,
    parse_file_identity_extent_records, parse_payload_block_identity_records,
    parse_redundant_map_files, parse_self_describing_extent_records,
    verify_and_plan_experimental_records, verify_and_plan_file_identity_extent_records,
    verify_and_plan_payload_block_identity_records, verify_and_plan_redundant_map,
};

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

#[derive(Debug)]
struct CliOptions {
    archive: PathBuf,
    json_out: Option<PathBuf>,
    export_fragments: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct SalvagePlan {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    verification_contract_label: &'static str,
    archive: ArchiveIdentity,
    footer_analysis: FooterAnalysis,
    index_analysis: IndexAnalysis,
    dictionary_analysis: DictionaryAnalysis,
    redundant_map_analysis: RedundantMapAnalysis,
    bootstrap_anchor_analysis: BootstrapAnchorAnalysis,
    block_candidates: Vec<BlockCandidate>,
    file_plans: Vec<FilePlan>,
    orphan_candidate_summary: OrphanCandidateSummary,
    summary: PlanSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    exported_artifacts: Option<ExportedArtifacts>,
}

#[derive(Debug, Serialize, Default)]
struct ExportedArtifacts {
    exported_block_artifacts: Vec<String>,
    exported_fragment_artifacts: Vec<String>,
    exported_complete_file_artifacts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ArchiveIdentity {
    archive_path: String,
    archive_size: u64,
    archive_blake3: String,
}

#[derive(Debug, Serialize)]
struct FooterAnalysis {
    status: &'static str,
    reason: &'static str,
    blocks_end_offset: Option<u64>,
    footer_offset: Option<u64>,
}

#[derive(Debug, Serialize)]
struct IndexAnalysis {
    status: &'static str,
    reason: &'static str,
    index_offset: Option<u64>,
    index_len: Option<u64>,
    entry_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictionaryAnalysis {
    status: &'static str,
    reason: &'static str,
    verified_dict_ids: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct RedundantMapAnalysis {
    status: &'static str,
    reason: &'static str,
    file_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct BootstrapAnchorAnalysis {
    status: &'static str,
    reason: &'static str,
    verified_anchor_count: u64,
}

#[derive(Debug)]
struct RedundantMapFile {
    path: String,
    size: u64,
    extents: Vec<Extent>,
}

#[derive(Debug)]
struct ExperimentalExtentRecord {
    path: String,
    size: u64,
    extent: Extent,
}

#[derive(Debug)]
struct FileIdentityExtentRecord {
    file_id: u32,
    size: u64,
    extent_ordinal: u64,
    extent: Extent,
    block_scan_offset: Option<u64>,
    path_digest_blake3: String,
    payload_hash_blake3: String,
    raw_hash_blake3: String,
}

#[derive(Debug)]
struct PayloadBlockIdentityRecord {
    archive_identity: String,
    file_id: u32,
    block_index: u64,
    total_block_count: u64,
    full_file_size: u64,
    logical_offset: u64,
    logical_length: u64,
    payload_codec: u32,
    payload_length: u64,
    block_id: u32,
    block_scan_offset: Option<u64>,
    payload_hash_blake3: String,
    raw_hash_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct BlockCandidate {
    scan_offset: u64,
    mapped_block_id: Option<u32>,
    structural_status: &'static str,
    header_status: &'static str,
    header_reason: &'static str,
    payload_bounds_status: &'static str,
    payload_hash_status: &'static str,
    dictionary_required: bool,
    dictionary_id: Option<u32>,
    dictionary_dependency_status: &'static str,
    decompression_status: &'static str,
    raw_hash_status: &'static str,
    content_verification_status: &'static str,
    content_verification_reasons: Vec<&'static str>,
    usable_for_indexed_planning: bool,
    verified_raw_len: Option<u64>,
}

#[derive(Debug, Serialize)]
struct FilePlan {
    mapping_provenance: &'static str,
    file_path: String,
    status: &'static str,
    reason: &'static str,
    failure_reasons: Vec<&'static str>,
    required_block_ids: Vec<u32>,
    #[serde(skip_serializing)]
    extents: Vec<Extent>,
    #[serde(skip_serializing)]
    file_size: u64,
}

#[derive(Debug, Serialize)]
struct OrphanCandidateSummary {
    total_candidates: u64,
    usable_candidates: u64,
    mapped_candidates: u64,
    orphan_unmappable_candidates: u64,
}

#[derive(Debug, Serialize)]
struct PlanSummary {
    salvageable_files: u64,
    unsalvageable_files: u64,
    unmappable_files: u64,
}

#[derive(Debug, Clone)]
struct BlockVerification {
    content_verified: bool,
    verified_raw_len: Option<u64>,
}

type PayloadIdentityGroup = (String, u64, u64, Vec<(u64, Extent)>);
struct BlockExportData {
    archive_offset: u64,
    block_id: u32,
    codec: u32,
    dictionary_dependency_status: &'static str,
    raw_hash: Option<String>,
    payload: Vec<u8>,
}

fn build_plan(opts: &CliOptions) -> Result<(SalvagePlan, Vec<u8>)> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };
    let archive_bytes =
        fs::read(&opts.archive).with_context(|| format!("read {}", opts.archive.display()))?;
    let archive_size = archive_bytes.len() as u64;

    let mut footer_analysis = FooterAnalysis {
        status: "missing",
        reason: "archive_too_short",
        blocks_end_offset: None,
        footer_offset: None,
    };
    let mut index_analysis = IndexAnalysis {
        status: "unavailable",
        reason: "tail_frame_unavailable",
        index_offset: None,
        index_len: None,
        entry_count: None,
    };
    let mut dictionary_analysis = DictionaryAnalysis {
        status: "unavailable",
        reason: "tail_frame_unavailable",
        verified_dict_ids: Vec::new(),
    };
    let mut redundant_map_analysis = RedundantMapAnalysis {
        status: "unavailable",
        reason: "tail_frame_unavailable",
        file_count: None,
    };
    let mut bootstrap_anchor_analysis = BootstrapAnchorAnalysis {
        status: "unavailable",
        reason: "tail_frame_unavailable",
        verified_anchor_count: 0,
    };

    let mut file_plans = Vec::new();
    let mut mapped_candidate_offsets = BTreeSet::new();

    if archive_size >= FTR4_LEN as u64 {
        let footer_offset = archive_size - FTR4_LEN as u64;
        footer_analysis.footer_offset = Some(footer_offset);

        let parsed_footer = Ftr4::read_from(Cursor::new(&archive_bytes[footer_offset as usize..]));
        if let Ok(footer) = parsed_footer {
            footer_analysis.status = "valid";
            footer_analysis.reason = "ok";
            footer_analysis.blocks_end_offset = Some(footer.blocks_end_offset);

            if footer.blocks_end_offset <= archive_size {
                let tail_bytes = &archive_bytes[footer.blocks_end_offset as usize..];
                if let Ok(tail) = parse_tail_frame(tail_bytes) {
                    let verified_dict_ids = tail
                        .dct1
                        .as_ref()
                        .map(|d| {
                            d.entries
                                .iter()
                                .map(|entry| entry.dict_id)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    dictionary_analysis = DictionaryAnalysis {
                        status: "available",
                        reason: "ok",
                        verified_dict_ids,
                    };

                    let block_verification = build_block_verification(
                        &reader,
                        &tail.footer,
                        &archive_bytes,
                        &dictionary_analysis.verified_dict_ids,
                    );

                    if tail.idx3_bytes.starts_with(IDX_MAGIC_V3) {
                        if let Ok(index) = decode_index(&tail.idx3_bytes) {
                            index_analysis = IndexAnalysis {
                                status: "valid",
                                reason: "ok",
                                index_offset: Some(tail.footer.index_offset),
                                index_len: Some(tail.footer.index_len),
                                entry_count: Some(index.entries.len() as u64),
                            };

                            for entry in index.entries {
                                if entry.kind != EntryKind::Regular {
                                    continue;
                                }

                                let required_block_ids =
                                    entry.extents.iter().map(|e| e.block_id).collect::<Vec<_>>();
                                let (status, reason, failure_reasons) = classify_file(
                                    &entry.extents,
                                    &required_block_ids,
                                    &block_verification,
                                );
                                file_plans.push(FilePlan {
                                    mapping_provenance: PRIMARY_INDEX_PATH,
                                    file_path: entry.path,
                                    status,
                                    reason,
                                    failure_reasons,
                                    required_block_ids,
                                    extents: entry.extents,
                                    file_size: entry.size,
                                });
                            }
                        } else {
                            index_analysis = IndexAnalysis {
                                status: "invalid",
                                reason: "idx3_decode_failed",
                                index_offset: Some(tail.footer.index_offset),
                                index_len: Some(tail.footer.index_len),
                                entry_count: None,
                            };
                        }
                    } else {
                        index_analysis = IndexAnalysis {
                            status: "invalid",
                            reason: "idx3_bad_magic",
                            index_offset: Some(tail.footer.index_offset),
                            index_len: Some(tail.footer.index_len),
                            entry_count: None,
                        };
                    }

                    if index_analysis.status != "valid" {
                        if let Some(ledger) = tail.ldg1 {
                            match parse_redundant_map_files(&ledger.json).and_then(|files| {
                                let count = files.len() as u64;
                                verify_and_plan_redundant_map(files, &block_verification)
                                    .map(|plans| (count, plans))
                            }) {
                                Ok((count, plans)) => {
                                    redundant_map_analysis = RedundantMapAnalysis {
                                        status: "valid",
                                        reason: "ok",
                                        file_count: Some(count),
                                    };
                                    file_plans = plans;
                                }
                                Err(_) => {
                                    redundant_map_analysis = RedundantMapAnalysis {
                                        status: "invalid",
                                        reason: "redundant_map_verification_failed",
                                        file_count: None,
                                    };
                                }
                            }
                        } else {
                            redundant_map_analysis = RedundantMapAnalysis {
                                status: "missing",
                                reason: "ledger_absent",
                                file_count: None,
                            };
                        }

                        if file_plans.is_empty() {
                            let experimental_values = parse_experimental_metadata_records(
                                &archive_bytes,
                                &block_verification,
                            );

                            if let Ok(checkpoint_plans) = verify_and_plan_experimental_records(
                                parse_checkpoint_extent_records(&experimental_values),
                                &block_verification,
                                CHECKPOINT_MAP_PATH,
                            ) {
                                if !checkpoint_plans.is_empty() {
                                    file_plans = checkpoint_plans;
                                }
                            }

                            if file_plans.is_empty() {
                                if let Ok(file_identity_plans) =
                                    verify_and_plan_file_identity_extent_records(
                                        parse_file_identity_extent_records(&experimental_values),
                                        &experimental_values,
                                        &block_verification,
                                        &BTreeSet::new(),
                                    )
                                {
                                    if !file_identity_plans.is_empty() {
                                        file_plans = file_identity_plans;
                                    }
                                }
                            }

                            if file_plans.is_empty() {
                                if let Ok(payload_identity_plans) =
                                    verify_and_plan_payload_block_identity_records(
                                        parse_payload_block_identity_records(&experimental_values),
                                        &experimental_values,
                                        &block_verification,
                                        &BTreeSet::new(),
                                    )
                                {
                                    if !payload_identity_plans.is_empty() {
                                        file_plans = payload_identity_plans;
                                    }
                                }
                            }

                            if file_plans.is_empty() {
                                if let Ok(extent_plans) = verify_and_plan_experimental_records(
                                    parse_self_describing_extent_records(&experimental_values),
                                    &block_verification,
                                    SELF_DESCRIBING_EXTENT_PATH,
                                ) {
                                    if !extent_plans.is_empty() {
                                        file_plans = extent_plans;
                                    }
                                }
                            }
                        }
                    } else {
                        redundant_map_analysis = RedundantMapAnalysis {
                            status: "not_used",
                            reason: "primary_index_available",
                            file_count: None,
                        };
                    }
                } else {
                    footer_analysis.status = "invalid";
                    footer_analysis.reason = "tail_frame_parse_failed";
                }
            } else {
                footer_analysis.status = "invalid";
                footer_analysis.reason = "blocks_end_offset_out_of_bounds";
            }
        } else {
            footer_analysis.status = "invalid";
            footer_analysis.reason = "ftr4_parse_failed";
        }
    }

    let mut candidates = scan_blk3_candidates(
        &archive_bytes,
        &dictionary_analysis.verified_dict_ids,
        dictionary_analysis.status == "available",
    );

    if footer_analysis.status == "valid" {
        if let Some(blocks_end_offset) = footer_analysis.blocks_end_offset {
            if let Ok(spans) = scan_blocks_v1(&reader, blocks_end_offset) {
                let index_by_offset = candidates
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (c.scan_offset, i))
                    .collect::<BTreeMap<_, _>>();
                for span in spans {
                    mapped_candidate_offsets.insert(span.header_offset);
                    if let Some(ix) = index_by_offset.get(&span.header_offset) {
                        candidates[*ix].mapped_block_id = Some(span.block_id);
                    }
                }
            }
        }
    }

    if file_plans.is_empty() && index_analysis.status != "valid" {
        let mut synthesized_block_verification = BTreeMap::new();
        let mut verified_candidate_offsets = BTreeSet::new();
        let mut ordinal = 0u32;
        for candidate in &candidates {
            if candidate.content_verification_status == "content_verified" {
                synthesized_block_verification.insert(
                    ordinal,
                    BlockVerification {
                        content_verified: true,
                        verified_raw_len: candidate.verified_raw_len,
                    },
                );
                verified_candidate_offsets.insert(candidate.scan_offset);
                ordinal = ordinal.saturating_add(1);
            }
        }
        let experimental_values =
            parse_experimental_metadata_records(&archive_bytes, &synthesized_block_verification);
        let verified_anchor_count = experimental_values
            .iter()
            .filter(|v| {
                v.get("schema").and_then(|x| x.as_str()) == Some("crushr-bootstrap-anchor.v1")
            })
            .count() as u64;
        bootstrap_anchor_analysis = if verified_anchor_count > 0 {
            BootstrapAnchorAnalysis {
                status: "available",
                reason: "verified_anchor_records_found",
                verified_anchor_count,
            }
        } else {
            BootstrapAnchorAnalysis {
                status: "missing",
                reason: "no_verified_anchor_records",
                verified_anchor_count: 0,
            }
        };

        if let Ok(file_identity_plans) = verify_and_plan_file_identity_extent_records(
            parse_file_identity_extent_records(&experimental_values),
            &experimental_values,
            &synthesized_block_verification,
            &verified_candidate_offsets,
        ) {
            if !file_identity_plans.is_empty() {
                file_plans = file_identity_plans;
            }
        }

        if file_plans.is_empty() {
            if let Ok(payload_identity_plans) = verify_and_plan_payload_block_identity_records(
                parse_payload_block_identity_records(&experimental_values),
                &experimental_values,
                &synthesized_block_verification,
                &verified_candidate_offsets,
            ) {
                if !payload_identity_plans.is_empty() {
                    file_plans = payload_identity_plans;
                }
            }
        }
    }

    file_plans.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    let usable_candidates = candidates
        .iter()
        .filter(|c| c.usable_for_indexed_planning)
        .count() as u64;
    let mapped_candidates = candidates
        .iter()
        .filter(|c| mapped_candidate_offsets.contains(&c.scan_offset))
        .count() as u64;
    let orphan_unmappable = candidates.len() as u64 - mapped_candidates;

    let salvageable_files = file_plans
        .iter()
        .filter(|f| f.status == "SALVAGEABLE")
        .count() as u64;
    let unsalvageable_files = file_plans
        .iter()
        .filter(|f| f.status == "UNSALVAGEABLE")
        .count() as u64;
    let unmappable_files = if file_plans.is_empty() && index_analysis.status != "valid" {
        1
    } else {
        file_plans
            .iter()
            .filter(|f| f.status == "UNMAPPABLE")
            .count() as u64
    };

    Ok((
        SalvagePlan {
            schema_version: "crushr-salvage-plan.v3",
            tool: "crushr-salvage",
            tool_version: env!("CARGO_PKG_VERSION"),
            verification_contract_label: "UNVERIFIED_RESEARCH_OUTPUT_NOT_CANONICAL_EXTRACTION",
            archive: ArchiveIdentity {
                archive_path: opts.archive.display().to_string(),
                archive_size,
                archive_blake3: to_hex(blake3::hash(&archive_bytes).as_bytes()),
            },
            footer_analysis,
            index_analysis,
            dictionary_analysis,
            redundant_map_analysis,
            bootstrap_anchor_analysis,
            block_candidates: candidates,
            file_plans,
            orphan_candidate_summary: OrphanCandidateSummary {
                total_candidates: archive_bytes
                    .windows(4)
                    .filter(|w| *w == BLK3_MAGIC)
                    .count() as u64,
                usable_candidates,
                mapped_candidates,
                orphan_unmappable_candidates: orphan_unmappable,
            },
            summary: PlanSummary {
                salvageable_files,
                unsalvageable_files,
                unmappable_files,
            },
            exported_artifacts: None,
        },
        archive_bytes,
    ))
}

fn run() -> Result<()> {
    let opts = parse_cli_options()?;
    let (mut plan, archive_bytes) = build_plan(&opts)?;

    if let Some(export_dir) = &opts.export_fragments {
        let exported = export_artifacts(
            export_dir,
            &archive_bytes,
            &plan.block_candidates,
            &plan.file_plans,
        )?;
        plan.exported_artifacts = Some(exported);
    }

    let rendered = serde_json::to_string_pretty(&plan)?;
    if let Some(path) = opts.json_out {
        write_json_output(&path, &rendered)?;
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}
