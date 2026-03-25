// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

#[derive(Debug, serde::Deserialize)]
struct RedundantMapLedger {
    schema: String,
    files: Vec<RedundantMapLedgerFile>,
}

#[derive(Debug, serde::Deserialize)]
struct RedundantMapLedgerFile {
    path: String,
    size: u64,
    extents: Vec<RedundantMapLedgerExtent>,
}

#[derive(Debug, serde::Deserialize)]
struct RedundantMapLedgerExtent {
    block_id: u32,
    file_offset: u64,
    len: u64,
}

#[derive(Debug, serde::Deserialize)]
struct ExperimentalSchema {
    schema: String,
}

#[derive(Debug, Clone)]
pub(super) enum ExperimentalMetadataRecord {
    SelfDescribingExtent(SelfDescribingExtentEnvelope),
    CheckpointMapSnapshot(CheckpointMapSnapshot),
    FilePathMap(FilePathMapRecord),
    FilePathMapEntry(FilePathMapEntryRecord),
    PathCheckpoint(PathCheckpointRecord),
    PathDictionaryCopyV1(PathDictionaryCopyV1Record),
    PathDictionaryCopyV2(PathDictionaryCopyV2Record),
    PayloadBlockIdentity(PayloadBlockIdentityMetadataRecord),
    FileIdentityExtent(FileIdentityExtentMetadataRecord),
    FileManifest(FileManifestMetadataRecord),
    BootstrapAnchor(BootstrapAnchorRecord),
    Unknown,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct SelfDescribingExtentEnvelope {
    record: ExtentRecord,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct CheckpointMapSnapshot {
    records: Vec<ExtentRecord>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct ExtentRecord {
    path: String,
    full_file_size: u64,
    block_id: u32,
    logical_offset: u64,
    logical_length: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct FilePathMapRecord {
    records: Vec<FilePathMapEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct FilePathMapEntryRecord {
    file_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct FilePathMapEntry {
    file_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PathCheckpointRecord {
    entries: Vec<PathCheckpointEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PathCheckpointEntry {
    file_id: u32,
    path: String,
    path_digest_blake3: String,
    full_file_size: u64,
    total_block_count: u64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryEntry {
    path_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryCopyV1Record {
    entries: Vec<PathDictionaryEntry>,
    #[serde(default)]
    generation: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct PathDictionaryCopyV2Record {
    #[serde(default)]
    archive_instance_id: Option<String>,
    #[serde(default)]
    dictionary_content_hash: Option<String>,
    #[serde(default)]
    dictionary_length: Option<u64>,
    #[serde(default)]
    generation: u64,
    body: PathDictionaryBody,
    body_raw_json: String,
}

#[derive(Debug, serde::Deserialize)]
struct PathDictionaryCopyV2RawRecord {
    #[serde(default)]
    archive_instance_id: Option<String>,
    #[serde(default)]
    dictionary_content_hash: Option<String>,
    #[serde(default)]
    dictionary_length: Option<u64>,
    #[serde(default)]
    generation: u64,
}

#[derive(Debug, serde::Deserialize)]
struct PathDictionaryCopyV2BodyOnlyRecord {
    body: PathDictionaryBody,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    representation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entry_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    directory_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    basename_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<PathDictionaryEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    directories: Option<Vec<PathDictionaryDirectory>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    basenames: Option<Vec<PathDictionaryBasename>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_bindings: Option<Vec<PathDictionaryFileBinding>>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct PathDictionaryBodyHashView<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    representation: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entry_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    basename_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directories: Option<&'a Vec<PathDictionaryDirectory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    basenames: Option<&'a Vec<PathDictionaryBasename>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_bindings: Option<&'a Vec<PathDictionaryFileBinding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<&'a Vec<PathDictionaryEntry>>,
}

impl<'a> From<&'a PathDictionaryBody> for PathDictionaryBodyHashView<'a> {
    fn from(body: &'a PathDictionaryBody) -> Self {
        Self {
            representation: body.representation.as_ref(),
            entry_count: body.entry_count,
            directory_count: body.directory_count,
            basename_count: body.basename_count,
            directories: body.directories.as_ref(),
            basenames: body.basenames.as_ref(),
            file_bindings: body.file_bindings.as_ref(),
            entries: body.entries.as_ref(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryDirectory {
    dir_id: u32,
    prefix: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryBasename {
    name_id: u32,
    basename: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct PathDictionaryFileBinding {
    path_id: u32,
    dir_id: u32,
    name_id: u32,
    path_digest_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PayloadBlockIdentityContent {
    payload_hash_blake3: String,
    raw_hash_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PayloadBlockIdentityMetadataRecord {
    #[serde(default)]
    archive_identity: Option<String>,
    #[serde(default)]
    file_id: Option<u32>,
    #[serde(default)]
    block_index: Option<u64>,
    #[serde(default)]
    total_block_count: Option<u64>,
    #[serde(default)]
    full_file_size: Option<u64>,
    #[serde(default)]
    logical_offset: Option<u64>,
    #[serde(default)]
    logical_length: Option<u64>,
    #[serde(default)]
    payload_codec: Option<u32>,
    #[serde(default)]
    payload_length: Option<u64>,
    #[serde(default)]
    block_id: Option<u32>,
    block_scan_offset: Option<u64>,
    #[serde(default)]
    content_identity: Option<PayloadBlockIdentityContent>,
    name: Option<String>,
    path: Option<String>,
    path_digest_blake3: Option<String>,
    path_id: Option<u32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct FileIdentityPathLinkage {
    path_digest_blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct FileIdentityExtentMetadataRecord {
    #[serde(default)]
    file_id: Option<u32>,
    #[serde(default)]
    full_file_size: Option<u64>,
    #[serde(default)]
    extent_ordinal: Option<u64>,
    #[serde(default)]
    block_id: Option<u32>,
    #[serde(default)]
    logical_offset: Option<u64>,
    #[serde(default)]
    logical_length: Option<u64>,
    block_scan_offset: Option<u64>,
    #[serde(default)]
    path_linkage: Option<FileIdentityPathLinkage>,
    #[serde(default)]
    content_identity: Option<PayloadBlockIdentityContent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct FileManifestMetadataRecord {
    file_id: u32,
    file_size: Option<u64>,
    expected_block_count: Option<u64>,
    extent_count: Option<u64>,
    file_digest: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct BootstrapAnchorRecord {
    anchor_ordinal: u64,
}

pub(super) fn parse_redundant_map_files(ledger_json: &[u8]) -> Result<Vec<RedundantMapFile>> {
    let ledger: RedundantMapLedger =
        serde_json::from_slice(ledger_json).context("parse LDG1 JSON as redundant map")?;
    if ledger.schema != "crushr-redundant-file-map.v1"
        && ledger.schema != "crushr-redundant-file-map.experimental.v2"
    {
        bail!("unsupported redundant map schema: {}", ledger.schema);
    }

    let mut out = Vec::with_capacity(ledger.files.len());
    for file in ledger.files {
        let path = file.path;
        if path.is_empty() {
            bail!("redundant map file path must be non-empty");
        }
        let mut extents = Vec::with_capacity(file.extents.len());
        for extent in file.extents {
            extents.push(Extent {
                block_id: extent.block_id,
                offset: extent.file_offset,
                len: extent.len,
                logical_offset: extent.file_offset,
            });
        }
        out.push(RedundantMapFile {
            path,
            size: file.size,
            extents,
        });
    }

    Ok(out)
}

pub(super) fn parse_experimental_metadata_records(
    archive_bytes: &[u8],
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> Vec<ExperimentalMetadataRecord> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    while offset + BLK3_MAGIC.len() <= archive_bytes.len() {
        if archive_bytes[offset..offset + 4] != BLK3_MAGIC {
            offset += 1;
            continue;
        }
        let Some(header_prefix) = archive_bytes.get(offset + 4..offset + 6) else {
            break;
        };
        let header_len = u16::from_le_bytes([header_prefix[0], header_prefix[1]]) as usize;
        if offset + header_len > archive_bytes.len() {
            offset += 1;
            continue;
        }
        let Ok(header) = read_blk3_header(Cursor::new(&archive_bytes[offset..offset + header_len]))
        else {
            offset += 1;
            continue;
        };
        let payload_offset = offset + header.header_len as usize;
        let Some(payload_end) = payload_offset.checked_add(header.comp_len as usize) else {
            offset += 1;
            continue;
        };
        if payload_end > archive_bytes.len() || header.codec != 1 {
            offset += 1;
            continue;
        }
        if let Some(raw_hash) = header.raw_hash {
            let Ok(raw) =
                zstd::decode_all(Cursor::new(&archive_bytes[payload_offset..payload_end]))
            else {
                offset += 1;
                continue;
            };
            if raw.len() as u64 != header.raw_len || blake3::hash(&raw).as_bytes() != &raw_hash {
                offset += 1;
                continue;
            }
            if let Some(record) = parse_experimental_metadata_record(&raw) {
                if let ExperimentalMetadataRecord::SelfDescribingExtent(envelope) = &record
                    && let Some(v) = block_verification.get(&envelope.record.block_id)
                    && !v.content_verified
                {
                    offset += 1;
                    continue;
                }
                records.push(record);
            }
        }
        offset += 1;
    }
    records
}

fn parse_experimental_metadata_record(raw: &[u8]) -> Option<ExperimentalMetadataRecord> {
    let schema = serde_json::from_slice::<ExperimentalSchema>(raw)
        .ok()?
        .schema;
    match schema.as_str() {
        "crushr-self-describing-extent.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::SelfDescribingExtent),
        "crushr-checkpoint-map-snapshot.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::CheckpointMapSnapshot),
        "crushr-file-path-map.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::FilePathMap),
        "crushr-file-path-map-entry.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::FilePathMapEntry),
        "crushr-path-checkpoint.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::PathCheckpoint),
        "crushr-path-dictionary-copy.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::PathDictionaryCopyV1),
        "crushr-path-dictionary-copy.v2" => {
            let raw_copy: PathDictionaryCopyV2RawRecord = serde_json::from_slice(raw).ok()?;
            let body = serde_json::from_slice::<PathDictionaryCopyV2BodyOnlyRecord>(raw)
                .ok()?
                .body;
            let body_raw_json = extract_top_level_field_raw_json(raw, "body")?;
            Some(ExperimentalMetadataRecord::PathDictionaryCopyV2(
                PathDictionaryCopyV2Record {
                    archive_instance_id: raw_copy.archive_instance_id,
                    dictionary_content_hash: raw_copy.dictionary_content_hash,
                    dictionary_length: raw_copy.dictionary_length,
                    generation: raw_copy.generation,
                    body,
                    body_raw_json,
                },
            ))
        }
        "crushr-payload-block-identity.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::PayloadBlockIdentity),
        "crushr-file-identity-extent.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::FileIdentityExtent),
        "crushr-file-manifest.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::FileManifest),
        "crushr-bootstrap-anchor.v1" => serde_json::from_slice(raw)
            .ok()
            .map(ExperimentalMetadataRecord::BootstrapAnchor),
        _ => Some(ExperimentalMetadataRecord::Unknown),
    }
}

fn extract_top_level_field_raw_json(raw: &[u8], field: &str) -> Option<String> {
    let mut i = skip_json_ws(raw, 0);
    if *raw.get(i)? != b'{' {
        return None;
    }
    i += 1;
    loop {
        i = skip_json_ws(raw, i);
        if *raw.get(i)? == b'}' {
            return None;
        }
        if *raw.get(i)? != b'"' {
            return None;
        }
        let key_end = json_string_end(raw, i)?;
        let key = serde_json::from_slice::<String>(&raw[i..key_end]).ok()?;
        i = skip_json_ws(raw, key_end);
        if *raw.get(i)? != b':' {
            return None;
        }
        i = skip_json_ws(raw, i + 1);
        let value_start = i;
        let value_end = json_value_end(raw, value_start)?;
        if key == field {
            return std::str::from_utf8(&raw[value_start..value_end])
                .ok()
                .map(str::to_string);
        }
        i = skip_json_ws(raw, value_end);
        match raw.get(i).copied()? {
            b',' => i += 1,
            b'}' => return None,
            _ => return None,
        }
    }
}

fn skip_json_ws(raw: &[u8], mut i: usize) -> usize {
    while i < raw.len() && raw[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn json_string_end(raw: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < raw.len() {
        match raw[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn json_value_end(raw: &[u8], start: usize) -> Option<usize> {
    match *raw.get(start)? {
        b'"' => json_string_end(raw, start),
        b'{' | b'[' => json_compound_end(raw, start),
        _ => {
            let mut i = start;
            while i < raw.len() {
                if matches!(raw[i], b',' | b'}' | b']') || raw[i].is_ascii_whitespace() {
                    break;
                }
                i += 1;
            }
            Some(i)
        }
    }
}

fn json_compound_end(raw: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let mut object_depth = 0usize;
    let mut array_depth = 0usize;
    let mut in_string = false;
    while i < raw.len() {
        let b = raw[i];
        if in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
                i += 1;
            }
            b'{' => {
                object_depth += 1;
                i += 1;
            }
            b'}' => {
                object_depth = object_depth.checked_sub(1)?;
                i += 1;
                if object_depth == 0 && array_depth == 0 {
                    return Some(i);
                }
            }
            b'[' => {
                array_depth += 1;
                i += 1;
            }
            b']' => {
                array_depth = array_depth.checked_sub(1)?;
                i += 1;
                if object_depth == 0 && array_depth == 0 {
                    return Some(i);
                }
            }
            _ => i += 1,
        }
    }
    None
}

pub(super) fn is_bootstrap_anchor_record(record: &ExperimentalMetadataRecord) -> bool {
    match record {
        ExperimentalMetadataRecord::BootstrapAnchor(anchor) => {
            let _ = anchor.anchor_ordinal;
            true
        }
        _ => false,
    }
}

pub(super) fn parse_self_describing_extent_records(
    records: &[ExperimentalMetadataRecord],
) -> Vec<ExperimentalExtentRecord> {
    let mut out = Vec::new();
    for record in records {
        let ExperimentalMetadataRecord::SelfDescribingExtent(envelope) = record else {
            continue;
        };
        out.push(ExperimentalExtentRecord {
            path: envelope.record.path.clone(),
            size: envelope.record.full_file_size,
            extent: Extent {
                block_id: envelope.record.block_id,
                offset: envelope.record.logical_offset,
                len: envelope.record.logical_length,
                logical_offset: envelope.record.logical_offset,
            },
        });
    }
    out
}

pub(super) fn parse_checkpoint_extent_records(
    records: &[ExperimentalMetadataRecord],
) -> Vec<ExperimentalExtentRecord> {
    let mut out = Vec::new();
    for record in records {
        let ExperimentalMetadataRecord::CheckpointMapSnapshot(snapshot) = record else {
            continue;
        };
        for rec in &snapshot.records {
            out.push(ExperimentalExtentRecord {
                path: rec.path.clone(),
                size: rec.full_file_size,
                extent: Extent {
                    block_id: rec.block_id,
                    offset: rec.logical_offset,
                    len: rec.logical_length,
                    logical_offset: rec.logical_offset,
                },
            });
        }
    }
    out
}

pub(super) fn parse_file_identity_path_map(
    records: &[ExperimentalMetadataRecord],
) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    for record in records {
        match record {
            ExperimentalMetadataRecord::FilePathMap(map_record) => {
                for rec in &map_record.records {
                    let computed = to_hex(blake3::hash(rec.path.as_bytes()).as_bytes());
                    if computed == rec.path_digest_blake3 {
                        out.insert(rec.file_id, rec.path.clone());
                    }
                }
            }
            ExperimentalMetadataRecord::FilePathMapEntry(rec) => {
                let computed = to_hex(blake3::hash(rec.path.as_bytes()).as_bytes());
                if computed == rec.path_digest_blake3 {
                    out.insert(rec.file_id, rec.path.clone());
                }
            }
            _ => {}
        }
    }
    out
}

pub(super) fn parse_payload_block_path_checkpoints(
    records: &[ExperimentalMetadataRecord],
) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    for record in records {
        let ExperimentalMetadataRecord::PathCheckpoint(snapshot) = record else {
            continue;
        };
        for entry in &snapshot.entries {
            if entry.total_block_count == 0 {
                continue;
            }
            let computed = to_hex(blake3::hash(entry.path.as_bytes()).as_bytes());
            if computed != entry.path_digest_blake3 {
                continue;
            }
            if entry.full_file_size > 0 {
                out.insert(entry.file_id, entry.path.clone());
            }
        }
    }
    out
}

#[derive(Debug, Default, Clone)]
pub(super) struct ParsedPathDictionary {
    pub(super) map: BTreeMap<u32, String>,
    pub(super) conflict: bool,
    pub(super) valid_dictionary_copy_count: u64,
    pub(super) rejected_wrong_archive_count: u64,
    pub(super) rejected_hash_mismatch_count: u64,
    pub(super) detected_generation_mismatch_count: u64,
}

pub(super) fn parse_payload_block_path_dictionary(
    records: &[ExperimentalMetadataRecord],
) -> ParsedPathDictionary {
    let mut report = ParsedPathDictionary::default();
    let mut canonical: Option<BTreeMap<u32, String>> = None;
    let mut canonical_generation: Option<u64> = None;
    let expected_archive = records.iter().find_map(|record| match record {
        ExperimentalMetadataRecord::PayloadBlockIdentity(value) => value.archive_identity.clone(),
        _ => None,
    });

    for record in records {
        let mut map = BTreeMap::new();
        let (generation, is_v2, v2_meta): (u64, bool, Option<&PathDictionaryCopyV2Record>) =
            match record {
                ExperimentalMetadataRecord::PathDictionaryCopyV1(copy) => {
                    for entry in &copy.entries {
                        let computed = to_hex(blake3::hash(entry.path.as_bytes()).as_bytes());
                        if computed == entry.path_digest_blake3 {
                            map.insert(entry.path_id, entry.path.clone());
                        }
                    }
                    (copy.generation, false, None)
                }
                ExperimentalMetadataRecord::PathDictionaryCopyV2(copy) => {
                    if let Some(entries) = &copy.body.entries {
                        for entry in entries {
                            let computed = to_hex(blake3::hash(entry.path.as_bytes()).as_bytes());
                            if computed == entry.path_digest_blake3 {
                                map.insert(entry.path_id, entry.path.clone());
                            }
                        }
                    } else if let (Some(directories), Some(basenames), Some(file_bindings)) = (
                        &copy.body.directories,
                        &copy.body.basenames,
                        &copy.body.file_bindings,
                    ) {
                        let mut dirs = BTreeMap::new();
                        let mut names = BTreeMap::new();
                        for d in directories {
                            dirs.insert(d.dir_id, d.prefix.clone());
                        }
                        for n in basenames {
                            names.insert(n.name_id, n.basename.clone());
                        }
                        for f in file_bindings {
                            let Some(prefix) = dirs.get(&f.dir_id) else {
                                continue;
                            };
                            let Some(name) = names.get(&f.name_id) else {
                                continue;
                            };
                            let path = if prefix.is_empty() {
                                name.clone()
                            } else {
                                format!("{prefix}/{name}")
                            };
                            let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
                            if computed == f.path_digest_blake3 {
                                map.insert(f.path_id, path);
                            }
                        }
                    }
                    (copy.generation, true, Some(copy))
                }
                _ => continue,
            };

        if map.is_empty() {
            continue;
        }

        if is_v2 {
            let copy = v2_meta.expect("v2 metadata must exist");
            if let (Some(expected), Some(actual)) = (
                expected_archive.as_deref(),
                copy.archive_instance_id.as_deref(),
            ) && expected != actual
            {
                report.rejected_wrong_archive_count += 1;
                continue;
            }
            let hash_view_bytes =
                match serde_json::to_vec(&PathDictionaryBodyHashView::from(&copy.body)) {
                    Ok(b) => b,
                    Err(_) => {
                        report.rejected_hash_mismatch_count += 1;
                        continue;
                    }
                };
            let raw_body_bytes = copy.body_raw_json.as_bytes();
            let expected_hash = copy.dictionary_content_hash.as_deref();
            let expected_len = copy.dictionary_length;
            let hash_view_hash = to_hex(blake3::hash(&hash_view_bytes).as_bytes());
            let raw_hash = to_hex(blake3::hash(raw_body_bytes).as_bytes());
            let hash_view_ok = expected_hash == Some(hash_view_hash.as_str())
                && expected_len == Some(hash_view_bytes.len() as u64);
            let raw_ok = expected_hash == Some(raw_hash.as_str())
                && expected_len == Some(raw_body_bytes.len() as u64);
            if !hash_view_ok && !raw_ok {
                report.rejected_hash_mismatch_count += 1;
                continue;
            }
        }

        report.valid_dictionary_copy_count += 1;
        if let Some(existing_generation) = canonical_generation {
            if generation != existing_generation {
                report.detected_generation_mismatch_count += 1;
                report.conflict = true;
                continue;
            }
        } else {
            canonical_generation = Some(generation);
        }

        if let Some(existing) = &canonical {
            if existing != &map {
                report.conflict = true;
            }
        } else {
            canonical = Some(map);
        }
    }

    report.map = canonical.unwrap_or_default();
    report
}

pub(super) fn parse_payload_block_identity_records(
    records: &[ExperimentalMetadataRecord],
) -> Vec<PayloadBlockIdentityRecord> {
    let mut out = Vec::new();
    for record in records {
        let ExperimentalMetadataRecord::PayloadBlockIdentity(value) = record else {
            continue;
        };
        let (
            Some(archive_identity),
            Some(file_id),
            Some(block_index),
            Some(total_block_count),
            Some(full_file_size),
            Some(logical_offset),
            Some(logical_length),
            Some(payload_codec),
            Some(payload_length),
            Some(block_id),
            Some(content_identity),
        ) = (
            value.archive_identity.clone(),
            value.file_id,
            value.block_index,
            value.total_block_count,
            value.full_file_size,
            value.logical_offset,
            value.logical_length,
            value.payload_codec,
            value.payload_length,
            value.block_id,
            value.content_identity.clone(),
        )
        else {
            continue;
        };
        out.push(PayloadBlockIdentityRecord {
            archive_identity,
            file_id,
            block_index,
            total_block_count,
            full_file_size,
            logical_offset,
            logical_length,
            payload_codec,
            payload_length,
            block_id,
            block_scan_offset: value.block_scan_offset,
            payload_hash_blake3: content_identity.payload_hash_blake3,
            raw_hash_blake3: content_identity.raw_hash_blake3,
            name: value.name.clone(),
            path: value.path.clone(),
            path_digest_blake3: value.path_digest_blake3.clone(),
            path_id: value.path_id,
        });
    }
    out
}

pub(super) fn verify_and_plan_payload_block_identity_records(
    records: Vec<PayloadBlockIdentityRecord>,
    metadata_records: &[ExperimentalMetadataRecord],
    block_verification: &BTreeMap<u32, BlockVerification>,
    verified_candidate_offsets: &BTreeSet<u64>,
) -> Result<Vec<FilePlan>> {
    let path_map = parse_payload_block_path_checkpoints(metadata_records);
    let parsed_dictionary = parse_payload_block_path_dictionary(metadata_records);
    let path_dictionary = parsed_dictionary.map;
    let dictionary_conflict = parsed_dictionary.conflict;
    let mut grouped: BTreeMap<String, PayloadIdentityGroup> = BTreeMap::new();
    for record in records {
        if record.total_block_count == 0 {
            bail!("payload block identity total_block_count must be > 0");
        }
        if record.payload_codec != 1 {
            bail!("payload block identity codec mismatch");
        }
        if !matches!(block_verification.get(&record.block_id), Some(v) if v.content_verified) {
            if let Some(scan_offset) = record.block_scan_offset {
                if !verified_candidate_offsets.contains(&scan_offset) {
                    bail!("payload block identity points to unverified content block");
                }
            } else {
                bail!("payload block identity points to unverified content block");
            }
        }
        if record.payload_hash_blake3.is_empty() || record.raw_hash_blake3.is_empty() {
            bail!("payload block identity missing content hash");
        }
        let inline_verified_path = if record.path_id.is_some() {
            None
        } else {
            match (&record.name, &record.path, &record.path_digest_blake3) {
                (Some(name), Some(path), Some(digest))
                    if to_hex(blake3::hash(path.as_bytes()).as_bytes()) == *digest
                        && Path::new(path)
                            .file_name()
                            .map(|p| p.to_string_lossy().as_ref() == name)
                            .unwrap_or(false) =>
                {
                    Some(path.clone())
                }
                _ => None,
            }
        };
        let path = if let Some(path) = inline_verified_path {
            path
        } else if let Some(path_id) = record.path_id {
            if dictionary_conflict {
                format!("anonymous_verified/file_{:08}.bin", record.file_id)
            } else {
                path_dictionary
                    .get(&path_id)
                    .cloned()
                    .unwrap_or_else(|| format!("anonymous_verified/file_{:08}.bin", record.file_id))
            }
        } else {
            path_map
                .get(&record.file_id)
                .cloned()
                .unwrap_or_else(|| format!("anonymous_verified/file_{:08}.bin", record.file_id))
        };
        let entry = grouped.entry(path).or_insert_with(|| {
            (
                record.archive_identity.clone(),
                record.full_file_size,
                record.total_block_count,
                Vec::new(),
            )
        });
        if entry.0 != record.archive_identity {
            bail!("payload block identity archive mismatch");
        }
        if entry.1 != record.full_file_size {
            bail!("payload block identity file size mismatch");
        }
        if entry.2 != record.total_block_count {
            bail!("payload block identity total block count mismatch");
        }
        if record
            .logical_offset
            .checked_add(record.logical_length)
            .context("payload block logical bounds overflow")?
            > record.full_file_size
        {
            bail!("payload block logical bounds exceed full file size");
        }
        if record.payload_length == 0 {
            bail!("payload block payload length must be non-zero");
        }
        entry.3.push((
            record.block_index,
            Extent {
                block_id: record.block_id,
                offset: record.logical_offset,
                len: record.logical_length,
                logical_offset: record.logical_offset,
            },
        ));
    }

    let mut plans = Vec::new();
    for (path, (_archive_id, size, total_blocks, mut extents)) in grouped {
        extents.sort_by_key(|(idx, _)| *idx);

        let mut seen = BTreeSet::new();
        for (idx, _) in &extents {
            if !seen.insert(*idx) {
                bail!("payload block identity duplicate block index");
            }
        }

        let has_index_gaps = extents
            .iter()
            .enumerate()
            .any(|(expected, (idx, _))| *idx != expected as u64);
        let is_complete = !has_index_gaps && extents.len() as u64 == total_blocks;

        let file_extents = extents.into_iter().map(|(_, e)| e).collect::<Vec<_>>();
        let required_block_ids = file_extents.iter().map(|e| e.block_id).collect::<Vec<_>>();

        let (status, reason, mut failure_reasons) = if is_complete {
            classify_file(&file_extents, &required_block_ids, block_verification)
        } else {
            (
                "UNSALVAGEABLE",
                "payload_block_identity_missing_required_block_coverage",
                vec![ReasonCode::PayloadBlockIdentityMissingRequiredBlockCoverage],
            )
        };

        let mut recovery_classification = if path.starts_with("anonymous_verified/") {
            RecoveryClassification::FullAnonymous
        } else {
            RecoveryClassification::FullVerified
        };

        if !is_complete {
            if has_index_gaps {
                recovery_classification = RecoveryClassification::PartialUnordered;
                failure_reasons.push(ReasonCode::PayloadBlockIdentityIndexGap);
            } else {
                recovery_classification = RecoveryClassification::PartialOrdered;
            }
        }

        plans.push(FilePlan {
            mapping_provenance: if path.starts_with("anonymous_verified/") {
                MappingProvenance::PayloadBlockIdentityPathAnonymous
            } else {
                MappingProvenance::PayloadBlockIdentityPath
            },
            recovery_classification,
            file_path: path,
            status,
            reason,
            failure_reasons,
            required_block_ids,
            extents: file_extents,
            file_size: size,
        });
    }

    plans.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(plans)
}

pub(super) fn parse_file_identity_extent_records(
    records: &[ExperimentalMetadataRecord],
) -> Vec<FileIdentityExtentRecord> {
    let mut out = Vec::new();
    for record in records {
        let ExperimentalMetadataRecord::FileIdentityExtent(value) = record else {
            continue;
        };
        let (
            Some(file_id),
            Some(size),
            Some(extent_ordinal),
            Some(block_id),
            Some(offset),
            Some(len),
            Some(path_linkage),
            Some(content_identity),
        ) = (
            value.file_id,
            value.full_file_size,
            value.extent_ordinal,
            value.block_id,
            value.logical_offset,
            value.logical_length,
            value.path_linkage.clone(),
            value.content_identity.clone(),
        )
        else {
            continue;
        };
        out.push(FileIdentityExtentRecord {
            file_id,
            size,
            extent_ordinal,
            extent: Extent {
                block_id,
                offset,
                len,
                logical_offset: offset,
            },
            block_scan_offset: value.block_scan_offset,
            path_digest_blake3: path_linkage.path_digest_blake3,
            payload_hash_blake3: content_identity.payload_hash_blake3,
            raw_hash_blake3: content_identity.raw_hash_blake3,
        });
    }
    out
}

pub(super) fn verify_and_plan_file_identity_extent_records(
    records: Vec<FileIdentityExtentRecord>,
    metadata_records: &[ExperimentalMetadataRecord],
    block_verification: &BTreeMap<u32, BlockVerification>,
    verified_candidate_offsets: &BTreeSet<u64>,
) -> Result<Vec<FilePlan>> {
    let path_map = parse_file_identity_path_map(metadata_records);
    let mut grouped: BTreeMap<String, (u64, Vec<(u64, Extent)>)> = BTreeMap::new();

    for record in records {
        if !matches!(block_verification.get(&record.extent.block_id), Some(v) if v.content_verified)
        {
            if let Some(scan_offset) = record.block_scan_offset {
                if !verified_candidate_offsets.contains(&scan_offset) {
                    bail!("file identity extent points to unverified content block");
                }
            } else {
                bail!("file identity extent points to unverified content block");
            }
        }
        let resolved_path = if let Some(path) = path_map.get(&record.file_id) {
            let computed_path_digest = to_hex(blake3::hash(path.as_bytes()).as_bytes());
            if computed_path_digest != record.path_digest_blake3 {
                bail!("file identity extent path linkage digest mismatch");
            }
            path.clone()
        } else {
            format!("anonymous_verified/file_{:08}.bin", record.file_id)
        };
        if record.payload_hash_blake3.is_empty() || record.raw_hash_blake3.is_empty() {
            bail!("file identity extent content identity mismatch");
        }

        let entry = grouped
            .entry(resolved_path)
            .or_insert_with(|| (record.size, Vec::new()));
        if entry.0 != record.size {
            bail!("inconsistent file-identity full_file_size");
        }
        entry.1.push((record.extent_ordinal, record.extent));
    }

    let files = grouped
        .into_iter()
        .map(|(path, (size, mut extents))| {
            extents.sort_by_key(|(ordinal, _)| *ordinal);
            for (expected, (ordinal, _)) in extents.iter().enumerate() {
                if *ordinal != expected as u64 {
                    bail!("file identity extent ordinal gap");
                }
            }
            let only_extents = extents.into_iter().map(|(_, e)| e).collect::<Vec<_>>();
            Ok(RedundantMapFile {
                path,
                size,
                extents: only_extents,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut plans = verify_and_plan_redundant_map(files, block_verification)?;
    let has_named_paths = plans
        .iter()
        .all(|p| !p.file_path.starts_with("anonymous_verified/"));
    for plan in &mut plans {
        plan.mapping_provenance = if has_named_paths {
            MappingProvenance::FileIdentityExtentPath
        } else {
            MappingProvenance::FileIdentityExtentPathAnonymous
        };
        plan.recovery_classification = if plan.file_path.starts_with("anonymous_verified/") {
            RecoveryClassification::FullAnonymous
        } else {
            RecoveryClassification::FullVerified
        };
    }
    Ok(plans)
}

#[derive(Debug)]
pub(super) struct FileManifestRecord {
    file_id: u32,
    path: Option<String>,
    file_size: u64,
    expected_block_count: u64,
    extent_count: u64,
    file_digest: String,
}

#[derive(Debug, Default)]
struct VerifiedRelationshipGraph {
    block_to_extent_edges: Vec<(u32, (u32, u64))>,
    extent_to_manifest_edges: Vec<((u32, u64), u32)>,
    manifest_to_path_edges: Vec<(u32, String)>,
    extent_ordinals_by_file: BTreeMap<u32, BTreeSet<u64>>,
    manifest_expected_count: BTreeMap<u32, u64>,
    manifest_has_path: BTreeMap<u32, bool>,
}

fn build_verified_graph(
    payload_records: &[PayloadBlockIdentityRecord],
    manifests: &BTreeMap<u32, FileManifestRecord>,
    path_map: &BTreeMap<u32, String>,
) -> VerifiedRelationshipGraph {
    let mut graph = VerifiedRelationshipGraph::default();
    let mut block_owner = BTreeMap::<u32, u32>::new();
    let mut rejected_blocks = BTreeSet::<u32>::new();

    for record in payload_records {
        if let Some(prev_owner) = block_owner.insert(record.block_id, record.file_id)
            && prev_owner != record.file_id
        {
            rejected_blocks.insert(record.block_id);
        }
    }

    for (&file_id, manifest) in manifests {
        graph
            .manifest_expected_count
            .insert(file_id, manifest.expected_block_count);
        graph.manifest_has_path.insert(
            file_id,
            path_map.contains_key(&file_id) || manifest.path.is_some(),
        );
    }

    for record in payload_records {
        if rejected_blocks.contains(&record.block_id) || !manifests.contains_key(&record.file_id) {
            continue;
        }
        let extent_node = (record.file_id, record.block_index);
        graph
            .block_to_extent_edges
            .push((record.block_id, extent_node));
        graph
            .extent_to_manifest_edges
            .push((extent_node, record.file_id));
        graph
            .extent_ordinals_by_file
            .entry(record.file_id)
            .or_default()
            .insert(record.block_index);
    }

    for (&file_id, path) in path_map {
        if manifests.contains_key(&file_id) {
            graph.manifest_to_path_edges.push((file_id, path.clone()));
        }
    }

    graph
}

fn classify_from_verified_graph(
    plan: &FilePlan,
    graph: &VerifiedRelationshipGraph,
    manifest_file_id: Option<u32>,
) -> RecoveryClassification {
    let block_count = plan.required_block_ids.len() as u64;
    let ordering_from_extents = plan.extents.windows(2).all(|w| w[0].offset <= w[1].offset);

    let Some(file_id) = manifest_file_id else {
        return RecoveryClassification::OrphanBlocks;
    };

    let has_manifest = graph.manifest_expected_count.contains_key(&file_id);
    let expected_count = graph
        .manifest_expected_count
        .get(&file_id)
        .copied()
        .unwrap_or(0);
    let has_path = graph
        .manifest_has_path
        .get(&file_id)
        .copied()
        .unwrap_or(false);
    let ordering_known = ordering_from_extents
        || graph
            .extent_ordinals_by_file
            .get(&file_id)
            .is_some_and(|ordinals| !ordinals.is_empty());

    if has_manifest && block_count == expected_count && has_path {
        RecoveryClassification::FullVerified
    } else if has_manifest && block_count == expected_count {
        RecoveryClassification::FullAnonymous
    } else if block_count > 0 && ordering_known {
        RecoveryClassification::PartialOrdered
    } else if block_count > 0 {
        RecoveryClassification::PartialUnordered
    } else {
        RecoveryClassification::OrphanBlocks
    }
}

pub(super) fn parse_file_manifest_records(
    records: &[ExperimentalMetadataRecord],
) -> Vec<FileManifestRecord> {
    let mut out = Vec::new();
    for record in records {
        let ExperimentalMetadataRecord::FileManifest(value) = record else {
            continue;
        };
        out.push(FileManifestRecord {
            file_id: value.file_id,
            path: value.path.clone(),
            file_size: value.file_size.unwrap_or(0),
            expected_block_count: value.expected_block_count.unwrap_or(0),
            extent_count: value.extent_count.unwrap_or(0),
            file_digest: value.file_digest.clone().unwrap_or_default(),
        });
    }
    out
}

pub(super) fn verify_and_apply_manifest_expectations(
    mut plans: Vec<FilePlan>,
    manifests: Vec<FileManifestRecord>,
    metadata_records: &[ExperimentalMetadataRecord],
    block_verification: &BTreeMap<u32, BlockVerification>,
    mapping_provenance: MappingProvenance,
) -> Result<Vec<FilePlan>> {
    let payload_path_map = parse_payload_block_path_checkpoints(metadata_records);
    let mut manifest_by_path = BTreeMap::new();
    let mut manifest_by_id = BTreeMap::new();
    for m in manifests {
        if m.expected_block_count == 0 || m.extent_count == 0 || m.file_digest.is_empty() {
            continue;
        }
        if let Some(path) = m.path.clone() {
            manifest_by_path.insert(path, m.file_id);
        }
        manifest_by_id.insert(m.file_id, m);
    }

    let payload_records = parse_payload_block_identity_records(metadata_records);
    let verified_graph = build_verified_graph(&payload_records, &manifest_by_id, &payload_path_map);
    let mut block_raw_hash = BTreeMap::new();
    for record in metadata_records {
        let ExperimentalMetadataRecord::PayloadBlockIdentity(payload) = record else {
            continue;
        };
        let (Some(block_id), Some(content_identity)) =
            (payload.block_id, payload.content_identity.clone())
        else {
            continue;
        };
        block_raw_hash.insert(block_id, content_identity.raw_hash_blake3);
    }

    if plans.is_empty() {
        let mut by_file_id: BTreeMap<u32, Vec<PayloadBlockIdentityRecord>> = BTreeMap::new();
        for record in payload_records {
            by_file_id.entry(record.file_id).or_default().push(record);
        }

        for (file_id, manifest) in &manifest_by_id {
            let mut extents = by_file_id
                .remove(file_id)
                .unwrap_or_default()
                .into_iter()
                .map(|record| {
                    (
                        record.block_index,
                        Extent {
                            block_id: record.block_id,
                            offset: record.logical_offset,
                            len: record.logical_length,
                            logical_offset: record.logical_offset,
                        },
                    )
                })
                .collect::<Vec<_>>();
            extents.sort_by_key(|(idx, _)| *idx);
            let extents = extents.into_iter().map(|(_, e)| e).collect::<Vec<_>>();
            let required_block_ids = extents.iter().map(|e| e.block_id).collect::<Vec<_>>();
            let (mut status, mut reason, mut failure_reasons) = if extents.is_empty() {
                (
                    "UNMAPPABLE",
                    "manifest has no recoverable block identity extents",
                    vec![ReasonCode::ManifestWithoutRecoverableExtents],
                )
            } else {
                classify_file(&extents, &required_block_ids, block_verification)
            };
            if manifest.expected_block_count > required_block_ids.len() as u64 {
                status = "UNSALVAGEABLE";
                reason = "manifest expected blocks missing from recovered set";
                failure_reasons.push(ReasonCode::ManifestExpectedBlocksMissing);
            }
            plans.push(FilePlan {
                mapping_provenance,
                recovery_classification: RecoveryClassification::OrphanBlocks,
                file_path: manifest
                    .path
                    .clone()
                    .or_else(|| payload_path_map.get(file_id).cloned())
                    .unwrap_or_else(|| format!("anonymous_verified/file_{:08}.bin", file_id)),
                status,
                reason,
                failure_reasons,
                required_block_ids,
                extents,
                file_size: manifest.file_size,
            });
        }
    }

    for plan in &mut plans {
        let manifest = manifest_by_path
            .get(&plan.file_path)
            .and_then(|id| manifest_by_id.get(id))
            .or_else(|| {
                if let Some(name) = plan.file_path.strip_prefix("anonymous_verified/file_") {
                    let id = name.strip_suffix(".bin")?.parse::<u32>().ok()?;
                    manifest_by_id.get(&id)
                } else {
                    None
                }
            });

        if let Some(manifest) = manifest {
            plan.mapping_provenance = mapping_provenance;
            let digest_match = if plan.required_block_ids.len() == 1 {
                block_raw_hash
                    .get(&plan.required_block_ids[0])
                    .map(|v| v == &manifest.file_digest)
                    .unwrap_or(false)
            } else {
                false
            };
            if plan.file_size == manifest.file_size
                && plan.required_block_ids.len() as u64 == manifest.expected_block_count
                && !digest_match
            {
                plan.failure_reasons
                    .push(ReasonCode::ManifestDigestNotVerified);
            }
            plan.recovery_classification =
                classify_from_verified_graph(plan, &verified_graph, Some(manifest.file_id));
        } else if plan.status == "UNMAPPABLE" {
            plan.recovery_classification = RecoveryClassification::OrphanBlocks;
        } else if plan.status == "UNSALVAGEABLE" {
            plan.recovery_classification =
                classify_from_verified_graph(plan, &verified_graph, None);
        }
    }

    Ok(plans)
}

pub(super) fn verify_and_plan_experimental_records(
    records: Vec<ExperimentalExtentRecord>,
    block_verification: &BTreeMap<u32, BlockVerification>,
    mapping_provenance: MappingProvenance,
) -> Result<Vec<FilePlan>> {
    let mut grouped: BTreeMap<String, (u64, Vec<Extent>)> = BTreeMap::new();
    for record in records {
        let entry = grouped
            .entry(record.path)
            .or_insert_with(|| (record.size, Vec::new()));
        if entry.0 != record.size {
            bail!("inconsistent experimental file size");
        }
        entry.1.push(record.extent);
    }
    let files = grouped
        .into_iter()
        .map(|(path, (size, mut extents))| {
            extents.sort_by_key(|e| e.offset);
            RedundantMapFile {
                path,
                size,
                extents,
            }
        })
        .collect::<Vec<_>>();
    let mut plans = verify_and_plan_redundant_map(files, block_verification)?;
    for plan in &mut plans {
        plan.mapping_provenance = mapping_provenance;
        plan.recovery_classification = RecoveryClassification::FullVerified;
    }
    Ok(plans)
}

pub(super) fn verify_and_plan_redundant_map(
    files: Vec<RedundantMapFile>,
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> Result<Vec<FilePlan>> {
    let mut seen_paths = BTreeSet::new();
    let mut plans = Vec::with_capacity(files.len());

    for file in files {
        if !seen_paths.insert(file.path.clone()) {
            bail!("redundant map contains duplicate file path: {}", file.path);
        }

        if file.size == 0 && !file.extents.is_empty() {
            bail!("redundant map zero-sized file has extents: {}", file.path);
        }

        let mut covered = 0u64;
        let mut prev_end = 0u64;
        for (idx, extent) in file.extents.iter().enumerate() {
            if extent.len == 0 {
                bail!(
                    "redundant map extent has zero length: {} extent {}",
                    file.path,
                    idx
                );
            }
            if extent.offset != prev_end {
                bail!(
                    "redundant map extents are non-contiguous or out of order: {} extent {}",
                    file.path,
                    idx
                );
            }
            let end = extent
                .offset
                .checked_add(extent.len)
                .context("redundant map extent offset overflow")?;
            if end > file.size {
                bail!("redundant map extent exceeds file size: {}", file.path);
            }
            prev_end = end;
            covered = covered
                .checked_add(extent.len)
                .context("redundant map file length overflow")?;

            let state = block_verification.get(&extent.block_id).with_context(|| {
                format!(
                    "redundant map references unmapped block {}",
                    extent.block_id
                )
            })?;
            let raw_len = state.verified_raw_len.with_context(|| {
                format!(
                    "redundant map block {} missing verified raw length",
                    extent.block_id
                )
            })?;
            if end > raw_len {
                bail!(
                    "redundant map extent exceeds verified block raw length for block {}",
                    extent.block_id
                );
            }
        }

        if covered != file.size {
            bail!(
                "redundant map extents do not fully cover file: {}",
                file.path
            );
        }

        let required_block_ids = file.extents.iter().map(|e| e.block_id).collect::<Vec<_>>();
        let (status, reason, failure_reasons) =
            classify_file(&file.extents, &required_block_ids, block_verification);

        plans.push(FilePlan {
            mapping_provenance: MappingProvenance::RedundantVerifiedMapPath,
            recovery_classification: RecoveryClassification::FullVerified,
            file_path: file.path,
            status,
            reason,
            failure_reasons,
            required_block_ids,
            extents: file.extents,
            file_size: file.size,
        });
    }

    plans.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(plans)
}

#[cfg(test)]
fn test_metadata_records(values: Vec<serde_json::Value>) -> Vec<ExperimentalMetadataRecord> {
    values
        .into_iter()
        .map(|value| {
            let raw = serde_json::to_vec(&value).expect("serialize test metadata record");
            parse_experimental_metadata_record(&raw).expect("parse typed test metadata record")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(file_id: u32, path: Option<&str>, expected_block_count: u64) -> FileManifestRecord {
        FileManifestRecord {
            file_id,
            path: path.map(str::to_string),
            file_size: 12,
            expected_block_count,
            extent_count: expected_block_count,
            file_digest: "digest".to_string(),
        }
    }

    fn plan(blocks: Vec<u32>, extents: Vec<Extent>) -> FilePlan {
        FilePlan {
            mapping_provenance: MappingProvenance::FileManifestPath,
            recovery_classification: RecoveryClassification::OrphanBlocks,
            file_path: "named/a.txt".to_string(),
            status: "SALVAGEABLE",
            reason: "ok",
            failure_reasons: Vec::new(),
            required_block_ids: blocks,
            extents,
            file_size: 12,
        }
    }

    #[test]
    fn graph_construction_basic() {
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "a".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 1,
            full_file_size: 12,
            logical_offset: 0,
            logical_length: 12,
            payload_codec: 1,
            payload_length: 12,
            block_id: 42,
            block_scan_offset: Some(4),
            payload_hash_blake3: "p".to_string(),
            raw_hash_blake3: "r".to_string(),
            name: None,
            path: None,
            path_digest_blake3: None,
            path_id: None,
        }];
        let manifests = BTreeMap::from([(7u32, manifest(7, Some("named/a.txt"), 1))]);
        let path_map = BTreeMap::from([(7u32, "named/a.txt".to_string())]);

        let graph = build_verified_graph(&records, &manifests, &path_map);
        assert_eq!(graph.block_to_extent_edges, vec![(42, (7, 0))]);
        assert_eq!(graph.extent_to_manifest_edges, vec![((7, 0), 7)]);
        assert_eq!(
            graph.manifest_to_path_edges,
            vec![(7, "named/a.txt".to_string())]
        );
    }

    #[test]
    fn classification_full_named() {
        let graph = VerifiedRelationshipGraph {
            manifest_expected_count: BTreeMap::from([(1u32, 2u64)]),
            manifest_has_path: BTreeMap::from([(1u32, true)]),
            extent_ordinals_by_file: BTreeMap::from([(1u32, BTreeSet::from([0u64, 1u64]))]),
            ..VerifiedRelationshipGraph::default()
        };
        let p = plan(
            vec![1, 2],
            vec![
                Extent {
                    block_id: 1,
                    offset: 0,
                    len: 6,
                    logical_offset: 0,
                },
                Extent {
                    block_id: 2,
                    offset: 6,
                    len: 6,
                    logical_offset: 6,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            RecoveryClassification::FullVerified
        );
    }

    #[test]
    fn classification_full_anonymous() {
        let graph = VerifiedRelationshipGraph {
            manifest_expected_count: BTreeMap::from([(1u32, 1u64)]),
            manifest_has_path: BTreeMap::from([(1u32, false)]),
            extent_ordinals_by_file: BTreeMap::from([(1u32, BTreeSet::from([0u64]))]),
            ..VerifiedRelationshipGraph::default()
        };
        let p = plan(
            vec![1],
            vec![Extent {
                block_id: 1,
                offset: 0,
                len: 12,
                logical_offset: 0,
            }],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            RecoveryClassification::FullAnonymous
        );
    }

    #[test]
    fn classification_partial_ordered() {
        let graph = VerifiedRelationshipGraph {
            manifest_expected_count: BTreeMap::from([(1u32, 3u64)]),
            manifest_has_path: BTreeMap::from([(1u32, true)]),
            extent_ordinals_by_file: BTreeMap::from([(1u32, BTreeSet::from([0u64, 1u64]))]),
            ..VerifiedRelationshipGraph::default()
        };
        let p = plan(
            vec![1, 2],
            vec![
                Extent {
                    block_id: 1,
                    offset: 0,
                    len: 6,
                    logical_offset: 0,
                },
                Extent {
                    block_id: 2,
                    offset: 6,
                    len: 6,
                    logical_offset: 6,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            RecoveryClassification::PartialOrdered
        );
    }

    #[test]
    fn classification_partial_unordered() {
        let graph = VerifiedRelationshipGraph {
            manifest_expected_count: BTreeMap::from([(1u32, 3u64)]),
            manifest_has_path: BTreeMap::from([(1u32, false)]),
            ..VerifiedRelationshipGraph::default()
        };
        let p = plan(
            vec![1, 2],
            vec![
                Extent {
                    block_id: 1,
                    offset: 6,
                    len: 6,
                    logical_offset: 6,
                },
                Extent {
                    block_id: 2,
                    offset: 0,
                    len: 6,
                    logical_offset: 0,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            RecoveryClassification::PartialUnordered
        );
    }

    #[test]
    fn classification_orphan() {
        let graph = VerifiedRelationshipGraph::default();
        let p = plan(
            vec![9],
            vec![Extent {
                block_id: 9,
                offset: 0,
                len: 4,
                logical_offset: 0,
            }],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, None),
            RecoveryClassification::OrphanBlocks
        );
        let p_no_blocks = plan(Vec::new(), Vec::new());
        assert_eq!(
            classify_from_verified_graph(&p_no_blocks, &graph, None),
            RecoveryClassification::OrphanBlocks
        );
    }

    #[test]
    fn payload_identity_groups_and_orders_by_block_index() {
        let records = vec![
            PayloadBlockIdentityRecord {
                archive_identity: "aid".to_string(),
                file_id: 7,
                block_index: 1,
                total_block_count: 2,
                full_file_size: 12,
                logical_offset: 6,
                logical_length: 6,
                payload_codec: 1,
                payload_length: 6,
                block_id: 2,
                block_scan_offset: Some(20),
                payload_hash_blake3: "p2".to_string(),
                raw_hash_blake3: "r2".to_string(),
                name: None,
                path: None,
                path_digest_blake3: None,
                path_id: None,
            },
            PayloadBlockIdentityRecord {
                archive_identity: "aid".to_string(),
                file_id: 7,
                block_index: 0,
                total_block_count: 2,
                full_file_size: 12,
                logical_offset: 0,
                logical_length: 6,
                payload_codec: 1,
                payload_length: 6,
                block_id: 1,
                block_scan_offset: Some(10),
                payload_hash_blake3: "p1".to_string(),
                raw_hash_blake3: "r1".to_string(),
                name: None,
                path: None,
                path_digest_blake3: None,
                path_id: None,
            },
        ];
        let values = test_metadata_records(vec![
            serde_json::json!({
                "schema": "crushr-payload-block-identity.v1",
                "block_id": 1,
                "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
            }),
            serde_json::json!({
                "schema": "crushr-payload-block-identity.v1",
                "block_id": 2,
                "content_identity": {"payload_hash_blake3":"p2", "raw_hash_blake3":"r2"},
            }),
        ]);
        let block_verification = BTreeMap::from([
            (
                1u32,
                BlockVerification {
                    content_verified: true,
                    verified_raw_len: Some(6),
                },
            ),
            (
                2u32,
                BlockVerification {
                    content_verified: true,
                    verified_raw_len: Some(12),
                },
            ),
        ]);
        let verified_offsets = BTreeSet::from([10u64, 20u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();

        assert_eq!(plans.len(), 1);
        let plan = &plans[0];
        assert_eq!(plan.extents[0].block_id, 1);
        assert_eq!(plan.extents[1].block_id, 2);
    }

    #[test]
    fn payload_identity_missing_extent_yields_partial_ordered() {
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "aid".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 2,
            full_file_size: 12,
            logical_offset: 0,
            logical_length: 6,
            payload_codec: 1,
            payload_length: 6,
            block_id: 1,
            block_scan_offset: Some(10),
            payload_hash_blake3: "p1".to_string(),
            raw_hash_blake3: "r1".to_string(),
            name: None,
            path: None,
            path_digest_blake3: None,
            path_id: None,
        }];
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })]);
        let block_verification = BTreeMap::from([(
            1u32,
            BlockVerification {
                content_verified: true,
                verified_raw_len: Some(6),
            },
        )]);
        let verified_offsets = BTreeSet::from([10u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();

        assert_eq!(plans.len(), 1);
        let plan = &plans[0];
        assert_eq!(plan.status, "UNSALVAGEABLE");
        assert_eq!(
            plan.recovery_classification,
            RecoveryClassification::PartialOrdered
        );
    }

    #[test]
    fn payload_identity_inline_path_recovers_named_path() {
        let path = "nested/file.txt";
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "aid".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 1,
            full_file_size: 6,
            logical_offset: 0,
            logical_length: 6,
            payload_codec: 1,
            payload_length: 6,
            block_id: 1,
            block_scan_offset: Some(10),
            payload_hash_blake3: "p1".to_string(),
            raw_hash_blake3: "r1".to_string(),
            name: Some("file.txt".to_string()),
            path: Some(path.to_string()),
            path_digest_blake3: Some(to_hex(blake3::hash(path.as_bytes()).as_bytes())),
            path_id: None,
        }];
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })]);
        let block_verification = BTreeMap::from([(
            1u32,
            BlockVerification {
                content_verified: true,
                verified_raw_len: Some(6),
            },
        )]);
        let verified_offsets = BTreeSet::from([10u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].file_path, path);
        assert_eq!(
            plans[0].recovery_classification,
            RecoveryClassification::FullVerified
        );
    }

    #[test]
    fn payload_identity_invalid_inline_path_falls_back_to_anonymous() {
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "aid".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 1,
            full_file_size: 6,
            logical_offset: 0,
            logical_length: 6,
            payload_codec: 1,
            payload_length: 6,
            block_id: 1,
            block_scan_offset: Some(10),
            payload_hash_blake3: "p1".to_string(),
            raw_hash_blake3: "r1".to_string(),
            name: Some("wrong.txt".to_string()),
            path: Some("nested/file.txt".to_string()),
            path_digest_blake3: Some("00".repeat(32)),
            path_id: None,
        }];
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })]);
        let block_verification = BTreeMap::from([(
            1u32,
            BlockVerification {
                content_verified: true,
                verified_raw_len: Some(6),
            },
        )]);
        let verified_offsets = BTreeSet::from([10u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();

        assert_eq!(plans.len(), 1);
        assert!(plans[0].file_path.starts_with("anonymous_verified/"));
        assert_eq!(
            plans[0].recovery_classification,
            RecoveryClassification::FullAnonymous
        );
    }
}

#[cfg(test)]
mod format13_dictionary_tests {
    use super::*;

    #[test]
    fn path_dictionary_parser_is_deterministic() {
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v1",
            "entries": [
                {"path_id": 0, "path": "a/b.txt", "path_digest_blake3": to_hex(blake3::hash("a/b.txt".as_bytes()).as_bytes())},
                {"path_id": 1, "path": "c.txt", "path_digest_blake3": to_hex(blake3::hash("c.txt".as_bytes()).as_bytes())}
            ]
        })]);
        let a = parse_payload_block_path_dictionary(&values);
        let b = parse_payload_block_path_dictionary(&values);
        assert_eq!(a.map, b.map);
        assert!(!a.conflict);
        assert!(!b.conflict);
    }

    #[test]
    fn v2_full_path_body_is_parsed() {
        let body = serde_json::json!({
            "representation": "full_path_v1",
            "entries": [{
                "path_id": 0,
                "path": "a/b.txt",
                "path_digest_blake3": to_hex(blake3::hash("a/b.txt".as_bytes()).as_bytes())
            }]
        });
        let body_bytes = serde_json::to_vec(&body).unwrap();
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v2",
            "archive_instance_id": "aid",
            "generation": 1,
            "dictionary_length": body_bytes.len(),
            "dictionary_content_hash": to_hex(blake3::hash(&body_bytes).as_bytes()),
            "body": body
        })]);
        let parsed = parse_payload_block_path_dictionary(&values);
        assert_eq!(parsed.map.get(&0).map(String::as_str), Some("a/b.txt"));
    }

    #[test]
    fn v2_dictionary_rejects_wrong_archive() {
        let body = serde_json::json!({
            "directories": [{"dir_id":0,"prefix":"a"}],
            "basenames": [{"name_id":0,"basename":"b.txt"}],
            "file_bindings": [{"path_id":0,"dir_id":0,"name_id":0,"path_digest_blake3": to_hex(blake3::hash("a/b.txt".as_bytes()).as_bytes())}]
        });
        let body_bytes = serde_json::to_vec(&body).unwrap();
        let values = test_metadata_records(vec![
            serde_json::json!({
                "schema": "crushr-payload-block-identity.v1",
                "archive_identity": "good_archive",
                "file_id": 0,
                "block_index": 0,
                "total_block_count": 1,
                "full_file_size": 1,
                "logical_offset": 0,
                "logical_length": 1,
                "payload_codec": 1,
                "payload_length": 1,
                "block_id": 1,
                "content_identity": {"payload_hash_blake3":"p","raw_hash_blake3":"r"}
            }),
            serde_json::json!({
                "schema": "crushr-path-dictionary-copy.v2",
                "archive_instance_id": "wrong_archive",
                "generation": 1,
                "dictionary_length": body_bytes.len(),
                "dictionary_content_hash": to_hex(blake3::hash(&body_bytes).as_bytes()),
                "body": body
            }),
        ]);
        let parsed = parse_payload_block_path_dictionary(&values);
        assert!(parsed.map.is_empty());
        assert_eq!(parsed.rejected_wrong_archive_count, 1);
    }

    #[test]
    fn v2_dictionary_rejects_hash_mismatch() {
        let body = serde_json::json!({
            "directories": [{"dir_id":0,"prefix":""}],
            "basenames": [{"name_id":0,"basename":"x.txt"}],
            "file_bindings": [{"path_id":0,"dir_id":0,"name_id":0,"path_digest_blake3": to_hex(blake3::hash("x.txt".as_bytes()).as_bytes())}]
        });
        let values = test_metadata_records(vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v2",
            "archive_instance_id": "a",
            "generation": 1,
            "dictionary_length": 999,
            "dictionary_content_hash": "00",
            "body": body
        })]);
        let parsed = parse_payload_block_path_dictionary(&values);
        assert!(parsed.map.is_empty());
        assert_eq!(parsed.rejected_hash_mismatch_count, 1);
    }

    #[test]
    fn v2_generation_mismatch_fails_closed() {
        let mk = |generation: u64| {
            let body = serde_json::json!({
                "directories": [{"dir_id":0,"prefix":""}],
                "basenames": [{"name_id":0,"basename":"x.txt"}],
                "file_bindings": [{"path_id":0,"dir_id":0,"name_id":0,"path_digest_blake3": to_hex(blake3::hash("x.txt".as_bytes()).as_bytes())}]
            });
            let body_bytes = serde_json::to_vec(&body).unwrap();
            serde_json::json!({
                "schema": "crushr-path-dictionary-copy.v2",
                "archive_instance_id": "a",
                "generation": generation,
                "dictionary_length": body_bytes.len(),
                "dictionary_content_hash": to_hex(blake3::hash(&body_bytes).as_bytes()),
                "body": body
            })
        };
        let parsed =
            parse_payload_block_path_dictionary(&test_metadata_records(vec![mk(1), mk(2)]));
        assert!(parsed.conflict);
        assert_eq!(parsed.detected_generation_mismatch_count, 1);
    }

    #[test]
    fn v2_raw_body_hash_parity_is_preserved_without_value_carrier() {
        let digest = to_hex(blake3::hash("a.txt".as_bytes()).as_bytes());
        let body_raw = format!(
            "{{\"representation\":\"full_path_v1\",\"entry_count\":1,\"entries\":[{{\"path_id\":0,\"path\":\"a.txt\",\"path_digest_blake3\":\"{}\"}}]}}",
            digest
        );
        let body_hash = to_hex(blake3::hash(body_raw.as_bytes()).as_bytes());
        let value = serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v2",
            "archive_instance_id": "aid",
            "generation": 1,
            "dictionary_length": body_raw.len(),
            "dictionary_content_hash": body_hash,
            "body": serde_json::from_str::<serde_json::Value>(&body_raw).unwrap()
        });
        let records = test_metadata_records(vec![value]);
        let parsed = parse_payload_block_path_dictionary(&records);
        assert_eq!(parsed.rejected_hash_mismatch_count, 0);
        assert_eq!(parsed.map.get(&0).map(String::as_str), Some("a.txt"));
    }

    #[test]
    fn missing_dictionary_copy_falls_back_to_anonymous_not_checkpoint_name() {
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "aid".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 1,
            full_file_size: 6,
            logical_offset: 0,
            logical_length: 6,
            payload_codec: 1,
            payload_length: 6,
            block_id: 1,
            block_scan_offset: Some(10),
            payload_hash_blake3: "p1".to_string(),
            raw_hash_blake3: "r1".to_string(),
            name: None,
            path: None,
            path_digest_blake3: None,
            path_id: Some(0),
        }];
        let values = test_metadata_records(vec![
            serde_json::json!({
                "schema": "crushr-payload-block-identity.v1",
                "block_id": 1,
                "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
            }),
            serde_json::json!({
                "schema": "crushr-path-checkpoint.v1",
                "entries": [
                    {
                        "file_id": 7,
                        "path": "named/from/checkpoint.txt",
                        "path_digest_blake3": to_hex(blake3::hash("named/from/checkpoint.txt".as_bytes()).as_bytes()),
                        "full_file_size": 6,
                        "total_block_count": 1
                    }
                ]
            }),
        ]);
        let block_verification = BTreeMap::from([(
            1u32,
            BlockVerification {
                content_verified: true,
                verified_raw_len: Some(6),
            },
        )]);
        let verified_offsets = BTreeSet::from([10u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();

        assert!(plans[0].file_path.starts_with("anonymous_verified/"));
        assert_eq!(
            plans[0].recovery_classification,
            RecoveryClassification::FullAnonymous
        );
    }

    #[test]
    fn conflicting_dictionary_copies_fail_closed_to_anonymous() {
        let records = vec![PayloadBlockIdentityRecord {
            archive_identity: "aid".to_string(),
            file_id: 7,
            block_index: 0,
            total_block_count: 1,
            full_file_size: 6,
            logical_offset: 0,
            logical_length: 6,
            payload_codec: 1,
            payload_length: 6,
            block_id: 1,
            block_scan_offset: Some(10),
            payload_hash_blake3: "p1".to_string(),
            raw_hash_blake3: "r1".to_string(),
            name: None,
            path: None,
            path_digest_blake3: None,
            path_id: Some(0),
        }];
        let values = test_metadata_records(vec![
            serde_json::json!({
                "schema": "crushr-payload-block-identity.v1",
                "block_id": 1,
                "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
            }),
            serde_json::json!({
                "schema": "crushr-path-dictionary-copy.v1",
                "entries": [
                    {"path_id": 0, "path": "one.txt", "path_digest_blake3": to_hex(blake3::hash("one.txt".as_bytes()).as_bytes())}
                ]
            }),
            serde_json::json!({
                "schema": "crushr-path-dictionary-copy.v1",
                "entries": [
                    {"path_id": 0, "path": "two.txt", "path_digest_blake3": to_hex(blake3::hash("two.txt".as_bytes()).as_bytes())}
                ]
            }),
        ]);
        let block_verification = BTreeMap::from([(
            1u32,
            BlockVerification {
                content_verified: true,
                verified_raw_len: Some(6),
            },
        )]);
        let verified_offsets = BTreeSet::from([10u64]);

        let plans = verify_and_plan_payload_block_identity_records(
            records,
            &values,
            &block_verification,
            &verified_offsets,
        )
        .unwrap();
        assert_eq!(
            plans[0].recovery_classification,
            RecoveryClassification::FullAnonymous
        );
    }
}
