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
struct SalvagePlanV1 {
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

#[derive(Debug, Serialize)]
struct BlockCandidate {
    scan_offset: u64,
    parse_status: &'static str,
    parse_reason: &'static str,
    payload_bounds_ok: bool,
    payload_hash_status: &'static str,
    payload_hash_reason: &'static str,
    dictionary_required: bool,
    dictionary_id: Option<u32>,
    usable_for_indexed_planning: bool,
}

#[derive(Debug, Serialize)]
struct FilePlan {
    file_path: String,
    status: &'static str,
    reason: &'static str,
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

#[derive(Debug, Clone)]
struct BlockVerification {
    payload_verified: bool,
    raw_verified: bool,
    dictionary_ok: bool,
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

fn scan_blk3_candidates(bytes: &[u8]) -> Vec<BlockCandidate> {
    let mut candidates = Vec::new();
    let mut offset = 0usize;

    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] == BLK3_MAGIC {
            let mut parse_status = "invalid";
            let mut parse_reason = "header_parse_failed";
            let mut payload_bounds_ok = false;
            let mut payload_hash_status = "unavailable";
            let mut payload_hash_reason = "header_unavailable";
            let mut dictionary_required = false;
            let mut dictionary_id = None;
            let mut usable_for_indexed_planning = false;

            if offset + 6 <= bytes.len() {
                let header_len =
                    u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]) as usize;
                if offset + header_len <= bytes.len() {
                    let parsed = read_blk3_header(Cursor::new(&bytes[offset..offset + header_len]));
                    if let Ok(header) = parsed {
                        parse_status = "valid";
                        parse_reason = "ok";
                        dictionary_required = header.flags.uses_dict();
                        dictionary_id = if dictionary_required {
                            Some(header.dict_id)
                        } else {
                            None
                        };

                        let payload_offset = offset + header.header_len as usize;
                        if let Some(payload_end) =
                            payload_offset.checked_add(header.comp_len as usize)
                        {
                            if payload_end <= bytes.len() {
                                payload_bounds_ok = true;
                                payload_hash_reason = "hash_not_present";
                                payload_hash_status = "unavailable";
                                if let Some(expected) = header.payload_hash {
                                    let actual = blake3::hash(&bytes[payload_offset..payload_end]);
                                    if actual.as_bytes() == &expected {
                                        payload_hash_status = "verified";
                                        payload_hash_reason = "ok";
                                    } else {
                                        payload_hash_status = "failed";
                                        payload_hash_reason = "hash_mismatch";
                                    }
                                }
                            } else {
                                payload_hash_reason = "payload_out_of_bounds";
                                payload_hash_status = "failed";
                            }
                        } else {
                            payload_hash_reason = "payload_end_overflow";
                            payload_hash_status = "failed";
                        }

                        usable_for_indexed_planning = parse_status == "valid"
                            && payload_bounds_ok
                            && payload_hash_status == "verified";
                    }
                } else {
                    parse_reason = "header_out_of_bounds";
                }
            } else {
                parse_reason = "header_prefix_out_of_bounds";
            }

            candidates.push(BlockCandidate {
                scan_offset: offset as u64,
                parse_status,
                parse_reason,
                payload_bounds_ok,
                payload_hash_status,
                payload_hash_reason,
                dictionary_required,
                dictionary_id,
                usable_for_indexed_planning,
            });
        }

        offset += 1;
    }

    candidates
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn build_plan(opts: &CliOptions) -> Result<SalvagePlanV1> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };
    let archive_bytes =
        fs::read(&opts.archive).with_context(|| format!("read {}", opts.archive.display()))?;
    let archive_size = archive_bytes.len() as u64;

    let candidates = scan_blk3_candidates(&archive_bytes);

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
                                &archive_bytes,
                                &tail.footer,
                                &dictionary_analysis.verified_dict_ids,
                                &mut mapped_candidate_offsets,
                            );

                            for entry in index.entries {
                                if entry.kind != EntryKind::Regular {
                                    continue;
                                }

                                let required_block_ids =
                                    entry.extents.iter().map(|e| e.block_id).collect::<Vec<_>>();
                                let (status, reason) =
                                    classify_file(&required_block_ids, &block_verification);
                                file_plans.push(FilePlan {
                                    file_path: entry.path,
                                    status,
                                    reason,
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

    Ok(SalvagePlanV1 {
        schema_version: "crushr-salvage-plan.v1",
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

fn build_block_verification<R: ReadAt + Len>(
    reader: &R,
    archive_bytes: &[u8],
    footer: &Ftr4,
    verified_dict_ids: &[u32],
    mapped_candidate_offsets: &mut BTreeSet<u64>,
) -> BTreeMap<u32, BlockVerification> {
    let mut out = BTreeMap::new();
    let spans = match scan_blocks_v1(reader, footer.blocks_end_offset) {
        Ok(v) => v,
        Err(_) => return out,
    };

    let verified_dict_set: BTreeSet<u32> = verified_dict_ids.iter().copied().collect();

    for span in spans {
        mapped_candidate_offsets.insert(span.header_offset);
        let header_len = (span.payload_offset - span.header_offset) as usize;
        let header_start = span.header_offset as usize;
        let header_end = header_start.saturating_add(header_len);
        if header_end > archive_bytes.len() {
            continue;
        }

        let header = match read_blk3_header(Cursor::new(&archive_bytes[header_start..header_end])) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let payload_start = span.payload_offset as usize;
        let payload_end = payload_start.saturating_add(span.comp_len as usize);
        if payload_end > archive_bytes.len() {
            continue;
        }

        let payload_verified = if let Some(expected) = span.payload_hash {
            blake3::hash(&archive_bytes[payload_start..payload_end]).as_bytes() == &expected
        } else {
            false
        };

        let dictionary_ok = if header.flags.uses_dict() {
            verified_dict_set.contains(&header.dict_id)
        } else {
            true
        };

        let raw_verified = if let Some(raw_hash) = header.raw_hash {
            if header.codec != 1 {
                false
            } else {
                zstd::decode_all(Cursor::new(&archive_bytes[payload_start..payload_end]))
                    .map(|raw| blake3::hash(&raw).as_bytes() == &raw_hash)
                    .unwrap_or(false)
            }
        } else {
            true
        };

        out.insert(
            span.block_id,
            BlockVerification {
                payload_verified,
                raw_verified,
                dictionary_ok,
            },
        );
    }

    out
}

fn classify_file(
    required_block_ids: &[u32],
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> (&'static str, &'static str) {
    if required_block_ids.is_empty() {
        return ("UNSALVAGEABLE", "no_required_blocks");
    }

    for block_id in required_block_ids {
        let state = match block_verification.get(block_id) {
            Some(v) => v,
            None => return ("UNSALVAGEABLE", "required_block_unmapped"),
        };

        if !state.payload_verified {
            return ("UNSALVAGEABLE", "required_block_payload_unverified");
        }
        if !state.dictionary_ok {
            return ("UNSALVAGEABLE", "required_dictionary_missing_or_unverified");
        }
        if !state.raw_verified {
            return ("UNSALVAGEABLE", "required_block_raw_unverified");
        }
    }

    ("SALVAGEABLE", "all_required_dependencies_verified")
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
