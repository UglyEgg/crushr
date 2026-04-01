// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction_payload_core::read_entry_bytes;
use crate::format::{
    Entry, EntryKind, Extent, IDX_MAGIC_V3, IDX_MAGIC_V4, IDX_MAGIC_V5, IDX_MAGIC_V6, IDX_MAGIC_V7,
    PreservationProfile,
};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    propagation::{
        FileDependencyV1, PropagationReportV1, STRUCTURE_FTR4, STRUCTURE_IDX3,
        STRUCTURE_TAIL_FRAME, build_propagation_report_v1, build_structural_failure_report_v1,
    },
    snapshot::info_envelope_from_open_archive,
    verify::{scan_blocks_v1, verify_block_payloads_v1},
};
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use crushr_format::tailframe::parse_tail_frame;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Cursor;

pub struct FileReader {
    file: File,
}

impl FileReader {
    pub fn open(path: &str) -> Result<Self> {
        let metadata = fs::metadata(path).with_context(|| format!("open {path}"))?;
        if !metadata.file_type().is_file() {
            bail!("archive path is not a regular file");
        }
        let file = File::open(path).with_context(|| format!("open {path}"))?;
        Ok(Self { file })
    }
}

impl ReadAt for FileReader {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            Ok(self.file.read_at(buf, offset)?)
        }
        #[cfg(not(unix))]
        {
            use std::io::{Read, Seek, SeekFrom};
            let mut cloned = self.file.try_clone().context("clone archive file handle")?;
            cloned
                .seek(SeekFrom::Start(offset))
                .context("seek archive file handle")?;
            Ok(cloned.read(buf).context("read archive file handle")?)
        }
    }
}

impl Len for FileReader {
    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

#[derive(Clone)]
struct SliceReader<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl ReadAt for SliceReader<'_> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let Ok(start) = usize::try_from(offset) else {
            return Ok(0);
        };
        if start >= self.bytes.len() {
            return Ok(0);
        }
        let available = self.bytes.len() - start;
        let n = available.min(buf.len());
        buf[..n].copy_from_slice(&self.bytes[start..start + n]);
        Ok(n)
    }
}

impl Len for SliceReader<'_> {
    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }
}

#[derive(Clone, serde::Serialize)]
pub struct ArchiveSummary {
    pub format_version: String,
    pub global_flags: String,
    pub preservation_profile: String,
    pub total_entries: u64,
    pub structure: ArchiveStructureSummary,
    pub verification: ArchiveVerificationSummary,
    pub strict_extraction_supported: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct ArchiveStructureSummary {
    pub extents: u64,
    pub dictionaries: u64,
    pub has_tail_frame: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct ArchiveVerificationSummary {
    pub extents_valid: bool,
    pub dictionaries_valid: bool,
    pub tail_frame_valid: bool,
}

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryTrustClass {
    Canonical,
    MetadataDegraded,
    RecoveredNamed,
    RecoveredAnonymous,
    Unrecoverable,
}

impl EntryTrustClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::MetadataDegraded => "metadata_degraded",
            Self::RecoveredNamed => "recovered_named",
            Self::RecoveredAnonymous => "recovered_anonymous",
            Self::Unrecoverable => "unrecoverable",
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct EntryLogicalRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct EntryReport {
    pub path: String,
    pub trust_class: EntryTrustClass,
    pub payload_verified: bool,
    pub metadata_complete: bool,
    pub extent_count: u64,
    pub size_bytes: u64,
    pub payload_blake3: String,
    pub logical_range: EntryLogicalRange,
    pub identity_source: String,
    pub reason: Option<String>,
    pub strict_extraction_supported: bool,
    pub extent_segments: Vec<EntryExtentSegment>,
}

#[derive(Clone, serde::Serialize)]
pub struct EntryExtentSegment {
    pub extent_index: u64,
    pub block_id: u64,
    pub logical_start: u64,
    pub logical_end: u64,
    pub size_bytes: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct EntryMatch {
    pub path: String,
    pub trust_class: EntryTrustClass,
}

pub fn inspect_archive(path: &str, product_version: &str) -> Result<ArchiveSummary> {
    let reader = FileReader::open(path)?;
    inspect_archive_reader(&reader, product_version)
}

pub fn inspect_archive_bytes(bytes: &[u8], product_version: &str) -> Result<ArchiveSummary> {
    let reader = SliceReader::new(bytes);
    inspect_archive_reader(&reader, product_version)
}

fn inspect_archive_reader<R: ReadAt + Len>(
    reader: &R,
    product_version: &str,
) -> Result<ArchiveSummary> {
    let opened = open_archive_v1(reader)?;
    let snapshot =
        info_envelope_from_open_archive(&opened, product_version, "1970-01-01T00:00:00Z");
    let index = decode_index(&opened.tail.idx3_bytes).ok();
    let total_entries = index
        .as_ref()
        .map(|idx| idx.entries.len() as u64)
        .unwrap_or(0);
    let extents = index
        .as_ref()
        .map(|idx| {
            idx.entries
                .iter()
                .filter(|entry| entry.kind == EntryKind::Regular)
                .map(|entry| entry.extents.len() as u64)
                .sum::<u64>()
        })
        .unwrap_or(0);
    let extents_valid = verify_block_payloads_v1(reader, opened.tail.footer.blocks_end_offset)
        .map(|bad| bad.is_empty())
        .unwrap_or(false);
    let dictionaries_valid = opened.tail.dct1.is_some() || !snapshot.payload.summary.has_dct1;
    let tail_frame_valid = !snapshot.payload.tail_frames.is_empty();

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

    Ok(ArchiveSummary {
        format_version: format_version.to_string(),
        global_flags: "none".to_string(),
        preservation_profile: index
            .as_ref()
            .map(|idx| idx.preservation_profile.as_str().to_string())
            .unwrap_or_else(|| PreservationProfile::Full.as_str().to_string()),
        total_entries,
        structure: ArchiveStructureSummary {
            extents,
            dictionaries: snapshot.payload.dicts.count as u64,
            has_tail_frame: !snapshot.payload.tail_frames.is_empty(),
        },
        verification: ArchiveVerificationSummary {
            extents_valid,
            dictionaries_valid,
            tail_frame_valid,
        },
        strict_extraction_supported: extents_valid && dictionaries_valid && tail_frame_valid,
    })
}

pub fn inspect_entry(path: &str, entry_path: &str) -> Result<Option<EntryReport>> {
    let reader = FileReader::open(path)?;
    inspect_entry_reader(&reader, entry_path)
}

pub fn inspect_entry_bytes(bytes: &[u8], entry_path: &str) -> Result<Option<EntryReport>> {
    let reader = SliceReader::new(bytes);
    inspect_entry_reader(&reader, entry_path)
}

fn inspect_entry_reader<R: ReadAt + Len>(
    reader: &R,
    entry_path: &str,
) -> Result<Option<EntryReport>> {
    let records = load_entry_records(reader)?;
    Ok(records.into_iter().find(|record| record.path == entry_path))
}

pub fn find_entries(path: &str, query: &str, limit: Option<usize>) -> Result<Vec<EntryMatch>> {
    let reader = FileReader::open(path)?;
    find_entries_reader(&reader, query, limit)
}

pub fn find_entries_bytes(
    bytes: &[u8],
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<EntryMatch>> {
    let reader = SliceReader::new(bytes);
    find_entries_reader(&reader, query, limit)
}

fn find_entries_reader<R: ReadAt + Len>(
    reader: &R,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<EntryMatch>> {
    let mut matches = load_entry_records(reader)?
        .into_iter()
        .filter(|record| record.path.contains(query))
        .map(|record| EntryMatch {
            path: record.path,
            trust_class: record.trust_class,
        })
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| a.path.cmp(&b.path));
    if let Some(limit) = limit {
        matches.truncate(limit);
    }
    Ok(matches)
}

pub fn analyze_propagation(path: &str) -> Result<PropagationReportV1> {
    let reader = FileReader::open(path)?;
    propagation_report_with_structural_fallback(&reader)
}

pub fn analyze_propagation_bytes(bytes: &[u8]) -> Result<PropagationReportV1> {
    let reader = SliceReader::new(bytes);
    propagation_report_with_structural_fallback(&reader)
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

fn load_entry_records<R: ReadAt + Len>(reader: &R) -> Result<Vec<EntryReport>> {
    match open_archive_v1(reader) {
        Ok(opened) => entry_records_from_index_bytes(reader, &opened.tail.idx3_bytes, false),
        Err(_) => {
            let idx3_bytes = read_idx3_bytes_from_footer(reader)?;
            entry_records_from_index_bytes(reader, &idx3_bytes, true)
        }
    }
}

fn entry_records_from_index_bytes<R: ReadAt + Len>(
    reader: &R,
    idx3_bytes: &[u8],
    degraded: bool,
) -> Result<Vec<EntryReport>> {
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
        records.push(EntryReport {
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
            extent_segments: build_extent_segments(&entry.extents),
        });
    }
    records.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(records)
}

fn build_extent_segments(extents: &[Extent]) -> Vec<EntryExtentSegment> {
    extents
        .iter()
        .enumerate()
        .map(|(idx, extent)| EntryExtentSegment {
            extent_index: idx as u64,
            block_id: u64::from(extent.block_id),
            logical_start: extent.logical_offset,
            logical_end: extent.logical_offset.saturating_add(extent.len),
            size_bytes: extent.len,
        })
        .collect()
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
