// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::cli_presentation::{CliPresenter, StatusWord, group_u64};
use crate::format::{EntryKind, IDX_MAGIC_V3};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
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
        println!("usage: crushr-info <archive> [--json] [--report propagation]");
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

    presenter.section("Structure");
    presenter.kv("has footer", snapshot.payload.summary.has_footer);
    presenter.kv("has dct1", snapshot.payload.summary.has_dct1);
    presenter.kv("has ldg1", snapshot.payload.summary.has_ldg1);
    presenter.kv(
        "tail frames",
        group_u64(snapshot.payload.tail_frames.len() as u64),
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
