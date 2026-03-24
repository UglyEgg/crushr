// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::cli_presentation::{CliPresenter, StatusWord, group_u64};
use crate::format::{EntryKind, IDX_MAGIC_V3};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
use crushr_core::verify::scan_blocks_v1;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    propagation::{
        FileDependencyV1, STRUCTURE_FTR4, STRUCTURE_IDX3, STRUCTURE_TAIL_FRAME,
        build_propagation_report_v1, build_structural_failure_report_v1,
    },
    snapshot::{info_envelope_from_open_archive, serialize_snapshot_json},
    verify::verify_block_payloads_v1,
};
use crushr_format::blk3::{BLK3_MAGIC, read_blk3_header};
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use crushr_format::tailframe::parse_tail_frame;
use std::collections::BTreeSet;
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
    regular_file_count: u64,
    extent_count: u64,
    logical_bytes: u64,
}

fn summarize_index(idx3_bytes: &[u8]) -> Option<IndexSummary> {
    let index = decode_index(idx3_bytes).ok()?;
    let mut regular_file_count = 0u64;
    let mut extent_count = 0u64;
    let mut logical_bytes = 0u64;

    for entry in index.entries {
        if entry.kind != EntryKind::Regular {
            continue;
        }
        regular_file_count += 1;
        extent_count += entry.extents.len() as u64;
        logical_bytes = logical_bytes.saturating_add(entry.size);
    }

    Some(IndexSummary {
        regular_file_count,
        extent_count,
        logical_bytes,
    })
}

#[derive(Debug)]
struct CompressionSummary {
    method: String,
    level: Option<String>,
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
    let presenter = CliPresenter::new("crushr-info", "help", false);
    presenter.header();
    presenter.section("Usage");
    presenter.kv(
        "command",
        "crushr-info <archive> [--json] [--report propagation]",
    );
    presenter.section("Flags");
    presenter.kv("--json", "emit machine-readable output");
    presenter.kv("--report propagation", "emit propagation/dependency report");
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

fn propagation_report_with_structural_fallback<R: ReadAt + Len>(reader: &R) -> Result<String> {
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
        return Ok(serialize_snapshot_json(&report)?);
    }

    let footer_offset = archive_len - FTR4_LEN as u64;
    let mut footer_bytes = vec![0u8; FTR4_LEN];
    if read_exact_at(reader, footer_offset, &mut footer_bytes).is_err() {
        let report = build_structural_failure_report_v1(&[
            STRUCTURE_FTR4,
            STRUCTURE_TAIL_FRAME,
            STRUCTURE_IDX3,
        ]);
        return Ok(serialize_snapshot_json(&report)?);
    }

    let footer = match Ftr4::read_from(Cursor::new(&footer_bytes)) {
        Ok(value) => value,
        Err(_) => {
            let report = build_structural_failure_report_v1(&[
                STRUCTURE_FTR4,
                STRUCTURE_TAIL_FRAME,
                STRUCTURE_IDX3,
            ]);
            return Ok(serialize_snapshot_json(&report)?);
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
            let magic_ok = idx3_bytes.starts_with(IDX_MAGIC_V3);
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

    let report =
        build_propagation_report_v1(&file_dependencies, &corrupted_structures, &corrupted_blocks);
    Ok(serialize_snapshot_json(&report)?)
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
    let mut report = None;

    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--json" {
            json = true;
        } else if arg == "--report" {
            report = Some(args.next().context("missing value for --report")?);
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(arg);
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    let archive =
        archive.context("usage: crushr-info <archive> [--json] [--report propagation]")?;

    let reader = FileReader {
        file: File::open(&archive).with_context(|| format!("open {archive}"))?,
    };

    if let Some(report_kind) = report {
        if report_kind != "propagation" {
            bail!("unsupported report: {report_kind} (expected propagation)");
        }
        println!("{}", propagation_report_with_structural_fallback(&reader)?);
        return Ok(());
    }

    let opened = open_archive_v1(&reader)?;
    let snapshot =
        info_envelope_from_open_archive(&opened, crate::product_version(), "1970-01-01T00:00:00Z");
    let rendered = serialize_snapshot_json(&snapshot)?;
    if json {
        println!("{rendered}");
        return Ok(());
    }

    let archive_blake3 = snapshot.archive_fingerprint.0.clone();

    let presenter = CliPresenter::new("crushr-info", "info", false);
    presenter.header();
    presenter.section("Archive");
    presenter.kv("path", &archive);
    presenter.kv(
        "size bytes",
        group_u64(snapshot.payload.summary.archive_len),
    );
    presenter.kv("blake3", archive_blake3);
    presenter.kv("format markers", "FTR4 + IDX3");

    presenter.section("Structure");
    let index_summary = summarize_index(&opened.tail.idx3_bytes);
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
    presenter.kv("block model", "file-level (1:1 file → unit)");
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
                || msg.contains("missing value for --report")
                || msg.contains("unsupported report")
                || msg.contains("unsupported flag")
                || msg.contains("unexpected argument")
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
