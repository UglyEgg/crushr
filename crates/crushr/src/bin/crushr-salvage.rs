use anyhow::{bail, Context, Result};
use crushr::format::{EntryKind, IDX_MAGIC_V3};
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
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: crushr-salvage <archive> [--json-out <path>]";

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
}

#[derive(Debug, Serialize)]
struct SalvagePlanV2 {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    verification_contract_label: &'static str,
    archive: ArchiveIdentity,
    footer_analysis: FooterAnalysis,
    index_analysis: IndexAnalysis,
    dictionary_analysis: DictionaryAnalysis,
    block_candidates: Vec<BlockCandidate>,
    file_plans: Vec<FilePlan>,
    orphan_candidate_summary: OrphanCandidateSummary,
    summary: PlanSummary,
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
    file_path: String,
    status: &'static str,
    reason: &'static str,
    failure_reasons: Vec<&'static str>,
    required_block_ids: Vec<u32>,
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

fn parse_cli_options() -> Result<CliOptions> {
    let mut archive = None;
    let mut json_out = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--json-out" {
            let path = args.next().context(USAGE)?;
            json_out = Some(PathBuf::from(path));
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(PathBuf::from(arg));
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    Ok(CliOptions {
        archive: archive.context(USAGE)?,
        json_out,
    })
}

fn scan_blk3_candidates(
    bytes: &[u8],
    verified_dict_ids: &[u32],
    dictionary_available: bool,
) -> Vec<BlockCandidate> {
    let mut candidates = Vec::new();
    let mut offset = 0usize;
    let verified_dict_set: BTreeSet<u32> = verified_dict_ids.iter().copied().collect();

    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] == BLK3_MAGIC {
            let mut candidate = BlockCandidate {
                scan_offset: offset as u64,
                mapped_block_id: None,
                structural_status: "detected",
                header_status: "invalid",
                header_reason: "header_parse_failed",
                payload_bounds_status: "unknown",
                payload_hash_status: "unavailable",
                dictionary_required: false,
                dictionary_id: None,
                dictionary_dependency_status: "not_required",
                decompression_status: "not_attempted",
                raw_hash_status: "not_attempted",
                content_verification_status: "not_content_verified",
                content_verification_reasons: vec!["header_invalid"],
                usable_for_indexed_planning: false,
                verified_raw_len: None,
            };

            if offset + 6 <= bytes.len() {
                let header_len =
                    u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]) as usize;
                if offset + header_len <= bytes.len() {
                    if let Ok(header) =
                        read_blk3_header(Cursor::new(&bytes[offset..offset + header_len]))
                    {
                        candidate.header_status = "valid";
                        candidate.header_reason = "ok";
                        candidate.dictionary_required = header.flags.uses_dict();
                        if candidate.dictionary_required {
                            candidate.dictionary_id = Some(header.dict_id);
                            candidate.dictionary_dependency_status =
                                if verified_dict_set.contains(&header.dict_id) {
                                    "satisfied"
                                } else if dictionary_available {
                                    "missing"
                                } else {
                                    "unresolved"
                                };
                        }

                        let payload_offset = offset + header.header_len as usize;
                        if let Some(payload_end) =
                            payload_offset.checked_add(header.comp_len as usize)
                        {
                            if payload_end <= bytes.len() {
                                candidate.payload_bounds_status = "in_bounds";
                                candidate.payload_hash_status = "unavailable";
                                if let Some(expected) = header.payload_hash {
                                    let actual = blake3::hash(&bytes[payload_offset..payload_end]);
                                    candidate.payload_hash_status =
                                        if actual.as_bytes() == &expected {
                                            "verified"
                                        } else {
                                            "mismatch"
                                        };
                                }

                                if candidate.dictionary_dependency_status == "satisfied"
                                    || candidate.dictionary_dependency_status == "not_required"
                                {
                                    if header.codec != 1 {
                                        candidate.decompression_status = "unsupported_codec";
                                    } else {
                                        match zstd::decode_all(Cursor::new(
                                            &bytes[payload_offset..payload_end],
                                        )) {
                                            Ok(raw) => {
                                                candidate.decompression_status = "success";
                                                if raw.len() as u64 == header.raw_len {
                                                    candidate.verified_raw_len =
                                                        Some(raw.len() as u64);
                                                }
                                                candidate.raw_hash_status = if let Some(raw_hash) =
                                                    header.raw_hash
                                                {
                                                    if blake3::hash(&raw).as_bytes() == &raw_hash {
                                                        "verified"
                                                    } else {
                                                        "mismatch"
                                                    }
                                                } else {
                                                    "unavailable"
                                                };
                                            }
                                            Err(_) => {
                                                candidate.decompression_status = "failed";
                                                candidate.raw_hash_status =
                                                    if header.raw_hash.is_some() {
                                                        "not_attempted"
                                                    } else {
                                                        "unavailable"
                                                    };
                                            }
                                        }
                                    }
                                } else {
                                    candidate.decompression_status = "not_attempted";
                                    candidate.raw_hash_status = if header.raw_hash.is_some() {
                                        "not_attempted"
                                    } else {
                                        "unavailable"
                                    };
                                }
                            } else {
                                candidate.payload_bounds_status = "out_of_bounds";
                            }
                        } else {
                            candidate.payload_bounds_status = "out_of_bounds";
                        }

                        let mut reasons = BTreeSet::new();
                        if candidate.payload_bounds_status != "in_bounds" {
                            reasons.insert("payload_out_of_bounds");
                        }
                        if candidate.payload_hash_status == "mismatch" {
                            reasons.insert("payload_hash_mismatch");
                        }
                        if matches!(
                            candidate.dictionary_dependency_status,
                            "missing" | "invalid" | "unresolved"
                        ) {
                            reasons.insert("dictionary_dependency_unsatisfied");
                        }
                        if candidate.decompression_status != "success" {
                            reasons.insert("decompression_not_successful");
                        }
                        if candidate.raw_hash_status == "mismatch" {
                            reasons.insert("raw_hash_mismatch");
                        }

                        if reasons.is_empty() {
                            candidate.content_verification_status = "content_verified";
                            candidate.content_verification_reasons =
                                vec!["all_required_checks_passed"];
                        } else {
                            candidate.content_verification_reasons = reasons.into_iter().collect();
                        }

                        candidate.usable_for_indexed_planning =
                            candidate.content_verification_status == "content_verified";
                    }
                } else {
                    candidate.header_reason = "header_out_of_bounds";
                    candidate.content_verification_reasons = vec!["header_out_of_bounds"];
                }
            } else {
                candidate.header_reason = "header_prefix_out_of_bounds";
                candidate.content_verification_reasons = vec!["header_prefix_out_of_bounds"];
            }

            candidates.push(candidate);
        }

        offset += 1;
    }

    candidates
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn build_plan(opts: &CliOptions) -> Result<SalvagePlanV2> {
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

                    if tail.idx3_bytes.starts_with(IDX_MAGIC_V3) {
                        if let Ok(index) = decode_index(&tail.idx3_bytes) {
                            index_analysis = IndexAnalysis {
                                status: "valid",
                                reason: "ok",
                                index_offset: Some(tail.footer.index_offset),
                                index_len: Some(tail.footer.index_len),
                                entry_count: Some(index.entries.len() as u64),
                            };

                            let block_verification = build_block_verification(
                                &reader,
                                &tail.footer,
                                &archive_bytes,
                                &dictionary_analysis.verified_dict_ids,
                            );

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
                                    file_path: entry.path,
                                    status,
                                    reason,
                                    failure_reasons,
                                    required_block_ids,
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

    Ok(SalvagePlanV2 {
        schema_version: "crushr-salvage-plan.v2",
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
    })
}

#[derive(Debug, Clone)]
struct BlockVerification {
    content_verified: bool,
    verified_raw_len: Option<u64>,
}

fn build_block_verification<R: ReadAt + Len>(
    reader: &R,
    footer: &Ftr4,
    archive_bytes: &[u8],
    verified_dict_ids: &[u32],
) -> BTreeMap<u32, BlockVerification> {
    let mut out = BTreeMap::new();
    let spans = match scan_blocks_v1(reader, footer.blocks_end_offset) {
        Ok(v) => v,
        Err(_) => return out,
    };

    let candidates = scan_blk3_candidates(archive_bytes, verified_dict_ids, true)
        .into_iter()
        .map(|c| (c.scan_offset, c))
        .collect::<BTreeMap<_, _>>();

    for span in spans {
        if let Some(candidate) = candidates.get(&span.header_offset) {
            out.insert(
                span.block_id,
                BlockVerification {
                    content_verified: candidate.content_verification_status == "content_verified",
                    verified_raw_len: candidate.verified_raw_len,
                },
            );
        }
    }

    out
}

fn classify_file(
    extents: &[crushr::format::Extent],
    required_block_ids: &[u32],
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> (&'static str, &'static str, Vec<&'static str>) {
    if required_block_ids.is_empty() {
        return (
            "UNSALVAGEABLE",
            "no_required_blocks",
            vec!["no_required_blocks"],
        );
    }

    let mut failure_reasons = BTreeSet::new();

    for block_id in required_block_ids {
        let state = match block_verification.get(block_id) {
            Some(v) => v,
            None => {
                failure_reasons.insert("required_block_unmapped");
                continue;
            }
        };

        if !state.content_verified {
            failure_reasons.insert("required_block_not_content_verified");
        }
    }

    for extent in extents {
        if let Some(state) = block_verification.get(&extent.block_id) {
            if let Some(raw_len) = state.verified_raw_len {
                let end = extent.offset.saturating_add(extent.len);
                if end > raw_len {
                    failure_reasons.insert("required_extent_out_of_bounds");
                }
            }
        }
    }

    if failure_reasons.is_empty() {
        (
            "SALVAGEABLE",
            "all_required_dependencies_verified",
            Vec::new(),
        )
    } else {
        let reasons = failure_reasons.into_iter().collect::<Vec<_>>();
        ("UNSALVAGEABLE", reasons[0], reasons)
    }
}

fn run() -> Result<()> {
    let opts = parse_cli_options()?;
    let plan = build_plan(&opts)?;
    let rendered = serde_json::to_string_pretty(&plan)?;

    if let Some(path) = opts.json_out {
        write_json_output(&path, &rendered)?;
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn write_json_output(path: &Path, rendered: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, rendered).with_context(|| format!("write {}", path.display()))?;
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
