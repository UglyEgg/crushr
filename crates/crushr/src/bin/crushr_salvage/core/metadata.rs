use super::*;

pub(super) fn parse_redundant_map_files(ledger_json: &[u8]) -> Result<Vec<RedundantMapFile>> {
    let value: Value = serde_json::from_slice(ledger_json).context("parse LDG1 JSON")?;
    let obj = value
        .as_object()
        .context("redundant map ledger must be a JSON object")?;
    let schema = obj
        .get("schema")
        .and_then(Value::as_str)
        .context("redundant map ledger missing schema")?;
    if schema != "crushr-redundant-file-map.v1"
        && schema != "crushr-redundant-file-map.experimental.v2"
    {
        bail!("unsupported redundant map schema: {schema}");
    }

    let files = obj
        .get("files")
        .and_then(Value::as_array)
        .context("redundant map ledger missing files array")?;

    let mut out = Vec::with_capacity(files.len());
    for file in files {
        let f = file
            .as_object()
            .context("redundant map file entry must be an object")?;
        let path = f
            .get("path")
            .and_then(Value::as_str)
            .context("redundant map file missing path")?
            .to_string();
        if path.is_empty() {
            bail!("redundant map file path must be non-empty");
        }
        let size = f
            .get("size")
            .and_then(Value::as_u64)
            .context("redundant map file missing size")?;
        let extents_value = f
            .get("extents")
            .and_then(Value::as_array)
            .context("redundant map file missing extents array")?;
        let mut extents = Vec::with_capacity(extents_value.len());
        for ex in extents_value {
            let e = ex
                .as_object()
                .context("redundant map extent entry must be an object")?;
            let block_id = e
                .get("block_id")
                .and_then(Value::as_u64)
                .context("redundant map extent missing block_id")?;
            let offset = e
                .get("file_offset")
                .and_then(Value::as_u64)
                .context("redundant map extent missing file_offset")?;
            let len = e
                .get("len")
                .and_then(Value::as_u64)
                .context("redundant map extent missing len")?;
            let block_id =
                u32::try_from(block_id).context("redundant map block_id out of range")?;
            extents.push(Extent {
                block_id,
                offset,
                len,
            });
        }
        out.push(RedundantMapFile {
            path,
            size,
            extents,
        });
    }

    Ok(out)
}

pub(super) fn parse_experimental_metadata_records(
    archive_bytes: &[u8],
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> Vec<Value> {
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
            if let Ok(value) = serde_json::from_slice::<Value>(&raw) {
                if let Some(block_id_u64) = value
                    .get("record")
                    .and_then(|r| r.get("block_id"))
                    .and_then(|v| v.as_u64())
                {
                    if let Ok(block_id) = u32::try_from(block_id_u64) {
                        if let Some(v) = block_verification.get(&block_id) {
                            if !v.content_verified {
                                offset += 1;
                                continue;
                            }
                        }
                    }
                }
                records.push(value);
            }
        }
        offset += 1;
    }
    records
}

pub(super) fn parse_self_describing_extent_records(
    values: &[Value],
) -> Vec<ExperimentalExtentRecord> {
    let mut out = Vec::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-self-describing-extent.v1")
        {
            continue;
        }
        let Some(record) = value.get("record") else {
            continue;
        };
        let Some(path) = record.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(size) = record.get("full_file_size").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(block_id_u64) = record.get("block_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(offset) = record.get("logical_offset").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(len) = record.get("logical_length").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(block_id) = u32::try_from(block_id_u64) else {
            continue;
        };
        out.push(ExperimentalExtentRecord {
            path: path.to_string(),
            size,
            extent: Extent {
                block_id,
                offset,
                len,
            },
        });
    }
    out
}

pub(super) fn parse_checkpoint_extent_records(values: &[Value]) -> Vec<ExperimentalExtentRecord> {
    let mut out = Vec::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-checkpoint-map-snapshot.v1")
        {
            continue;
        }
        let Some(records) = value.get("records").and_then(|v| v.as_array()) else {
            continue;
        };
        for rec in records {
            let Some(path) = rec.get("path").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size) = rec.get("full_file_size").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Some(block_id_u64) = rec.get("block_id").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Some(offset) = rec.get("logical_offset").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Some(len) = rec.get("logical_length").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Ok(block_id) = u32::try_from(block_id_u64) else {
                continue;
            };
            out.push(ExperimentalExtentRecord {
                path: path.to_string(),
                size,
                extent: Extent {
                    block_id,
                    offset,
                    len,
                },
            });
        }
    }
    out
}

pub(super) fn parse_file_identity_path_map(values: &[Value]) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    for value in values {
        let schema = value.get("schema").and_then(|v| v.as_str());
        if schema == Some("crushr-file-path-map.v1") {
            let Some(records) = value.get("records").and_then(|v| v.as_array()) else {
                continue;
            };
            for rec in records {
                let Some(file_id_u64) = rec.get("file_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Ok(file_id) = u32::try_from(file_id_u64) else {
                    continue;
                };
                let Some(path) = rec.get("path").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(path_digest) = rec.get("path_digest_blake3").and_then(|v| v.as_str())
                else {
                    continue;
                };
                let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
                if computed != path_digest {
                    continue;
                }
                out.insert(file_id, path.to_string());
            }
            continue;
        }
        if schema != Some("crushr-file-path-map-entry.v1") {
            continue;
        }
        let Some(file_id_u64) = value.get("file_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(file_id) = u32::try_from(file_id_u64) else {
            continue;
        };
        let Some(path) = value.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(path_digest) = value.get("path_digest_blake3").and_then(|v| v.as_str()) else {
            continue;
        };
        let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
        if computed != path_digest {
            continue;
        }
        out.insert(file_id, path.to_string());
    }
    out
}

pub(super) fn parse_payload_block_path_checkpoints(values: &[Value]) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-path-checkpoint.v1") {
            continue;
        }
        let Some(entries) = value.get("entries").and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in entries {
            let Some(file_id_u64) = entry.get("file_id").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Ok(file_id) = u32::try_from(file_id_u64) else {
                continue;
            };
            let Some(path) = entry.get("path").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(path_digest) = entry.get("path_digest_blake3").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(full_file_size) = entry.get("full_file_size").and_then(|v| v.as_u64()) else {
                continue;
            };
            let Some(total_block_count) = entry.get("total_block_count").and_then(|v| v.as_u64())
            else {
                continue;
            };
            if total_block_count == 0 {
                continue;
            }
            let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
            if computed != path_digest {
                continue;
            }
            if entry
                .get("path")
                .and_then(|_| entry.get("full_file_size"))
                .is_some()
                && full_file_size > 0
            {
                out.insert(file_id, path.to_string());
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

pub(super) fn parse_payload_block_path_dictionary(values: &[Value]) -> ParsedPathDictionary {
    let mut report = ParsedPathDictionary::default();
    let mut canonical: Option<BTreeMap<u32, String>> = None;
    let mut canonical_generation: Option<u64> = None;
    let expected_archive = values.iter().find_map(|v| {
        (v.get("schema").and_then(|s| s.as_str()) == Some("crushr-payload-block-identity.v1"))
            .then(|| {
                v.get("archive_identity")
                    .and_then(|a| a.as_str())
                    .map(str::to_string)
            })
            .flatten()
    });

    for value in values {
        let schema = value.get("schema").and_then(|v| v.as_str());
        if schema != Some("crushr-path-dictionary-copy.v1")
            && schema != Some("crushr-path-dictionary-copy.v2")
        {
            continue;
        }

        let mut map = BTreeMap::new();
        if let Some(entries) = value.get("entries").and_then(|v| v.as_array()) {
            for entry in entries {
                let Some(path_id_u64) = entry.get("path_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Ok(path_id) = u32::try_from(path_id_u64) else {
                    continue;
                };
                let Some(path) = entry.get("path").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(path_digest) = entry.get("path_digest_blake3").and_then(|v| v.as_str())
                else {
                    continue;
                };
                let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
                if computed != path_digest {
                    continue;
                }
                map.insert(path_id, path.to_string());
            }
        } else if let Some(body) = value.get("body") {
            if let Some(entries) = body.get("entries").and_then(|v| v.as_array()) {
                for entry in entries {
                    let Some(path_id_u64) = entry.get("path_id").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let Ok(path_id) = u32::try_from(path_id_u64) else {
                        continue;
                    };
                    let Some(path) = entry.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(path_digest) =
                        entry.get("path_digest_blake3").and_then(|v| v.as_str())
                    else {
                        continue;
                    };
                    let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
                    if computed != path_digest {
                        continue;
                    }
                    map.insert(path_id, path.to_string());
                }
            } else {
                let mut dirs = BTreeMap::new();
                let mut names = BTreeMap::new();
                if let Some(arr) = body.get("directories").and_then(|v| v.as_array()) {
                    for d in arr {
                        if let (Some(id), Some(prefix)) = (
                            d.get("dir_id").and_then(|v| v.as_u64()),
                            d.get("prefix").and_then(|v| v.as_str()),
                        ) {
                            dirs.insert(id as u32, prefix.to_string());
                        }
                    }
                }
                if let Some(arr) = body.get("basenames").and_then(|v| v.as_array()) {
                    for n in arr {
                        if let (Some(id), Some(name)) = (
                            n.get("name_id").and_then(|v| v.as_u64()),
                            n.get("basename").and_then(|v| v.as_str()),
                        ) {
                            names.insert(id as u32, name.to_string());
                        }
                    }
                }
                if let Some(arr) = body.get("file_bindings").and_then(|v| v.as_array()) {
                    for f in arr {
                        let (Some(path_id), Some(dir_id), Some(name_id), Some(path_digest)) = (
                            f.get("path_id").and_then(|v| v.as_u64()),
                            f.get("dir_id").and_then(|v| v.as_u64()),
                            f.get("name_id").and_then(|v| v.as_u64()),
                            f.get("path_digest_blake3").and_then(|v| v.as_str()),
                        ) else {
                            continue;
                        };
                        let Some(prefix) = dirs.get(&(dir_id as u32)) else {
                            continue;
                        };
                        let Some(name) = names.get(&(name_id as u32)) else {
                            continue;
                        };
                        let path = if prefix.is_empty() {
                            name.clone()
                        } else {
                            format!("{prefix}/{name}")
                        };
                        let computed = to_hex(blake3::hash(path.as_bytes()).as_bytes());
                        if computed != path_digest {
                            continue;
                        }
                        map.insert(path_id as u32, path);
                    }
                }
            }
        }

        if map.is_empty() {
            continue;
        }

        if schema == Some("crushr-path-dictionary-copy.v2") {
            if let (Some(expected), Some(actual)) = (
                expected_archive.as_deref(),
                value.get("archive_instance_id").and_then(|v| v.as_str()),
            ) {
                if expected != actual {
                    report.rejected_wrong_archive_count += 1;
                    continue;
                }
            }
            if let Some(body) = value.get("body") {
                let body_bytes = match serde_json::to_vec(body) {
                    Ok(b) => b,
                    Err(_) => {
                        report.rejected_hash_mismatch_count += 1;
                        continue;
                    }
                };
                let computed = to_hex(blake3::hash(&body_bytes).as_bytes());
                if value
                    .get("dictionary_content_hash")
                    .and_then(|v| v.as_str())
                    != Some(computed.as_str())
                {
                    report.rejected_hash_mismatch_count += 1;
                    continue;
                }
                if value.get("dictionary_length").and_then(|v| v.as_u64())
                    != Some(body_bytes.len() as u64)
                {
                    report.rejected_hash_mismatch_count += 1;
                    continue;
                }
            }
        }

        let generation = value
            .get("generation")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
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
    values: &[Value],
) -> Vec<PayloadBlockIdentityRecord> {
    let mut out = Vec::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-payload-block-identity.v1")
        {
            continue;
        }
        let Some(archive_identity) = value.get("archive_identity").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(file_id_u64) = value.get("file_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(file_id) = u32::try_from(file_id_u64) else {
            continue;
        };
        let Some(block_index) = value.get("block_index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(total_block_count) = value.get("total_block_count").and_then(|v| v.as_u64())
        else {
            continue;
        };
        let Some(full_file_size) = value.get("full_file_size").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(logical_offset) = value.get("logical_offset").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(logical_length) = value.get("logical_length").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(payload_codec_u64) = value.get("payload_codec").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(payload_codec) = u32::try_from(payload_codec_u64) else {
            continue;
        };
        let Some(payload_length) = value.get("payload_length").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(block_id_u64) = value.get("block_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(block_id) = u32::try_from(block_id_u64) else {
            continue;
        };
        let block_scan_offset = value.get("block_scan_offset").and_then(|v| v.as_u64());
        let Some(payload_hash_blake3) = value
            .get("content_identity")
            .and_then(|v| v.get("payload_hash_blake3"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let Some(raw_hash_blake3) = value
            .get("content_identity")
            .and_then(|v| v.get("raw_hash_blake3"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let path_digest_blake3 = value
            .get("path_digest_blake3")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let path_id = value
            .get("path_id")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok());
        out.push(PayloadBlockIdentityRecord {
            archive_identity: archive_identity.to_string(),
            file_id,
            block_index,
            total_block_count,
            full_file_size,
            logical_offset,
            logical_length,
            payload_codec,
            payload_length,
            block_id,
            block_scan_offset,
            payload_hash_blake3: payload_hash_blake3.to_string(),
            raw_hash_blake3: raw_hash_blake3.to_string(),
            name,
            path,
            path_digest_blake3,
            path_id,
        });
    }
    out
}

pub(super) fn verify_and_plan_payload_block_identity_records(
    records: Vec<PayloadBlockIdentityRecord>,
    values: &[Value],
    block_verification: &BTreeMap<u32, BlockVerification>,
    verified_candidate_offsets: &BTreeSet<u64>,
) -> Result<Vec<FilePlan>> {
    let path_map = parse_payload_block_path_checkpoints(values);
    let parsed_dictionary = parse_payload_block_path_dictionary(values);
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
        if let Some(value) = values.iter().find(|value| {
            value.get("schema").and_then(|v| v.as_str()) == Some("crushr-payload-block-identity.v1")
                && value.get("block_id").and_then(|v| v.as_u64()) == Some(record.block_id as u64)
        }) {
            let payload = value
                .get("content_identity")
                .and_then(|v| v.get("payload_hash_blake3"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let raw = value
                .get("content_identity")
                .and_then(|v| v.get("raw_hash_blake3"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if payload != record.payload_hash_blake3 || raw != record.raw_hash_blake3 {
                bail!("payload block identity content hash mismatch");
            }
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
                vec!["payload_block_identity_missing_required_block_coverage"],
            )
        };

        let mut recovery_classification = if path.starts_with("anonymous_verified/") {
            "FULL_ANONYMOUS_VERIFIED"
        } else {
            "FULL_NAMED_VERIFIED"
        };

        if !is_complete {
            if has_index_gaps {
                recovery_classification = "PARTIAL_UNORDERED_VERIFIED";
                failure_reasons.push("payload_block_identity_index_gap");
            } else {
                recovery_classification = "PARTIAL_ORDERED_VERIFIED";
            }
        }

        plans.push(FilePlan {
            mapping_provenance: if path.starts_with("anonymous_verified/") {
                PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS
            } else {
                PAYLOAD_BLOCK_IDENTITY_PATH
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
    values: &[Value],
) -> Vec<FileIdentityExtentRecord> {
    let mut out = Vec::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-file-identity-extent.v1") {
            continue;
        }
        let Some(file_id_u64) = value.get("file_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(file_id) = u32::try_from(file_id_u64) else {
            continue;
        };
        let Some(size) = value.get("full_file_size").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(extent_ordinal) = value.get("extent_ordinal").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(block_id_u64) = value.get("block_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(block_id) = u32::try_from(block_id_u64) else {
            continue;
        };
        let Some(offset) = value.get("logical_offset").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(len) = value.get("logical_length").and_then(|v| v.as_u64()) else {
            continue;
        };
        let block_scan_offset = value.get("block_scan_offset").and_then(|v| v.as_u64());
        let Some(path_digest) = value
            .get("path_linkage")
            .and_then(|v| v.get("path_digest_blake3"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let Some(payload_hash) = value
            .get("content_identity")
            .and_then(|v| v.get("payload_hash_blake3"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let Some(raw_hash) = value
            .get("content_identity")
            .and_then(|v| v.get("raw_hash_blake3"))
            .and_then(|v| v.as_str())
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
            },
            block_scan_offset,
            path_digest_blake3: path_digest.to_string(),
            payload_hash_blake3: payload_hash.to_string(),
            raw_hash_blake3: raw_hash.to_string(),
        });
    }
    out
}

pub(super) fn verify_and_plan_file_identity_extent_records(
    records: Vec<FileIdentityExtentRecord>,
    values: &[Value],
    block_verification: &BTreeMap<u32, BlockVerification>,
    verified_candidate_offsets: &BTreeSet<u64>,
) -> Result<Vec<FilePlan>> {
    let path_map = parse_file_identity_path_map(values);
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
        if let Some(value) = values.iter().find(|value| {
            value.get("schema").and_then(|v| v.as_str()) == Some("crushr-file-identity-extent.v1")
                && value.get("block_id").and_then(|v| v.as_u64())
                    == Some(record.extent.block_id as u64)
        }) {
            let payload = value
                .get("content_identity")
                .and_then(|v| v.get("payload_hash_blake3"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let raw = value
                .get("content_identity")
                .and_then(|v| v.get("raw_hash_blake3"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if payload != record.payload_hash_blake3 || raw != record.raw_hash_blake3 {
                bail!("file identity extent content identity mismatch");
            }
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
            FILE_IDENTITY_EXTENT_PATH
        } else {
            FILE_IDENTITY_EXTENT_PATH_ANONYMOUS
        };
        plan.recovery_classification = if plan.file_path.starts_with("anonymous_verified/") {
            "FULL_ANONYMOUS"
        } else {
            "FULL_VERIFIED"
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
        if let Some(prev_owner) = block_owner.insert(record.block_id, record.file_id) {
            if prev_owner != record.file_id {
                rejected_blocks.insert(record.block_id);
            }
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
) -> &'static str {
    let block_count = plan.required_block_ids.len() as u64;
    let ordering_from_extents = plan.extents.windows(2).all(|w| w[0].offset <= w[1].offset);

    let Some(file_id) = manifest_file_id else {
        return if block_count > 0 {
            "ORPHAN_EVIDENCE_ONLY"
        } else {
            "NO_VERIFIED_EVIDENCE"
        };
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
        "FULL_NAMED_VERIFIED"
    } else if has_manifest && block_count == expected_count {
        "FULL_ANONYMOUS_VERIFIED"
    } else if block_count > 0 && ordering_known {
        "PARTIAL_ORDERED_VERIFIED"
    } else if block_count > 0 {
        "PARTIAL_UNORDERED_VERIFIED"
    } else {
        "ORPHAN_EVIDENCE_ONLY"
    }
}

pub(super) fn parse_file_manifest_records(values: &[Value]) -> Vec<FileManifestRecord> {
    let mut out = Vec::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-file-manifest.v1") {
            continue;
        }
        let Some(file_id_u64) = value.get("file_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(file_id) = u32::try_from(file_id_u64) else {
            continue;
        };
        let file_size = value.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let expected_block_count = value
            .get("expected_block_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let extent_count = value
            .get("extent_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let file_digest = value
            .get("file_digest")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let path = value
            .get("path")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        out.push(FileManifestRecord {
            file_id,
            path,
            file_size,
            expected_block_count,
            extent_count,
            file_digest,
        });
    }
    out
}

pub(super) fn verify_and_apply_manifest_expectations(
    mut plans: Vec<FilePlan>,
    manifests: Vec<FileManifestRecord>,
    values: &[Value],
    block_verification: &BTreeMap<u32, BlockVerification>,
    mapping_provenance: &'static str,
) -> Result<Vec<FilePlan>> {
    let payload_path_map = parse_payload_block_path_checkpoints(values);
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

    let payload_records = parse_payload_block_identity_records(values);
    let verified_graph = build_verified_graph(&payload_records, &manifest_by_id, &payload_path_map);
    let mut block_raw_hash = BTreeMap::new();
    for value in values {
        if value.get("schema").and_then(|v| v.as_str()) != Some("crushr-payload-block-identity.v1")
        {
            continue;
        }
        let Some(block_id_u64) = value.get("block_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Ok(block_id) = u32::try_from(block_id_u64) else {
            continue;
        };
        let Some(raw_hash) = value
            .get("content_identity")
            .and_then(|v| v.get("raw_hash_blake3"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        block_raw_hash.insert(block_id, raw_hash.to_string());
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
                    vec!["manifest_without_recoverable_extents"],
                )
            } else {
                classify_file(&extents, &required_block_ids, block_verification)
            };
            if manifest.expected_block_count > required_block_ids.len() as u64 {
                status = "UNSALVAGEABLE";
                reason = "manifest expected blocks missing from recovered set";
                failure_reasons.push("manifest_expected_blocks_missing");
            }
            plans.push(FilePlan {
                mapping_provenance,
                recovery_classification: "ORPHAN_EVIDENCE_ONLY",
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
                plan.failure_reasons.push("manifest_digest_not_verified");
            }
            plan.recovery_classification =
                classify_from_verified_graph(plan, &verified_graph, Some(manifest.file_id));
        } else if plan.status == "UNMAPPABLE" {
            plan.recovery_classification = "ORPHAN_EVIDENCE_ONLY";
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
    mapping_provenance: &'static str,
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
        plan.recovery_classification = "FULL_NAMED_VERIFIED";
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
            mapping_provenance: REDUNDANT_VERIFIED_MAP_PATH,
            recovery_classification: "FULL_NAMED_VERIFIED",
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
            mapping_provenance: FILE_MANIFEST_PATH,
            recovery_classification: "ORPHAN_EVIDENCE_ONLY",
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
                },
                Extent {
                    block_id: 2,
                    offset: 6,
                    len: 6,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            "FULL_NAMED_VERIFIED"
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
            }],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            "FULL_ANONYMOUS_VERIFIED"
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
                },
                Extent {
                    block_id: 2,
                    offset: 6,
                    len: 6,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            "PARTIAL_ORDERED_VERIFIED"
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
                },
                Extent {
                    block_id: 2,
                    offset: 0,
                    len: 6,
                },
            ],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, Some(1)),
            "PARTIAL_UNORDERED_VERIFIED"
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
            }],
        );
        assert_eq!(
            classify_from_verified_graph(&p, &graph, None),
            "ORPHAN_EVIDENCE_ONLY"
        );
        let p_no_blocks = plan(Vec::new(), Vec::new());
        assert_eq!(
            classify_from_verified_graph(&p_no_blocks, &graph, None),
            "NO_VERIFIED_EVIDENCE"
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
        let values = vec![
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
        ];
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
        let values = vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })];
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
        assert_eq!(plan.recovery_classification, "PARTIAL_ORDERED_VERIFIED");
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
        let values = vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })];
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
        assert_eq!(plans[0].recovery_classification, "FULL_NAMED_VERIFIED");
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
        let values = vec![serde_json::json!({
            "schema": "crushr-payload-block-identity.v1",
            "block_id": 1,
            "content_identity": {"payload_hash_blake3":"p1", "raw_hash_blake3":"r1"},
        })];
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
        assert_eq!(plans[0].recovery_classification, "FULL_ANONYMOUS_VERIFIED");
    }
}

#[cfg(test)]
mod format13_dictionary_tests {
    use super::*;

    #[test]
    fn path_dictionary_parser_is_deterministic() {
        let values = vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v1",
            "entries": [
                {"path_id": 0, "path": "a/b.txt", "path_digest_blake3": to_hex(blake3::hash("a/b.txt".as_bytes()).as_bytes())},
                {"path_id": 1, "path": "c.txt", "path_digest_blake3": to_hex(blake3::hash("c.txt".as_bytes()).as_bytes())}
            ]
        })];
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
        let values = vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v2",
            "archive_instance_id": "aid",
            "generation": 1,
            "dictionary_length": body_bytes.len(),
            "dictionary_content_hash": to_hex(blake3::hash(&body_bytes).as_bytes()),
            "body": body
        })];
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
        let values = vec![
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
        ];
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
        let values = vec![serde_json::json!({
            "schema": "crushr-path-dictionary-copy.v2",
            "archive_instance_id": "a",
            "generation": 1,
            "dictionary_length": 999,
            "dictionary_content_hash": "00",
            "body": body
        })];
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
        let parsed = parse_payload_block_path_dictionary(&[mk(1), mk(2)]);
        assert!(parsed.conflict);
        assert_eq!(parsed.detected_generation_mismatch_count, 1);
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
        let values = vec![
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
        ];
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
        assert_eq!(plans[0].recovery_classification, "FULL_ANONYMOUS_VERIFIED");
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
        let values = vec![
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
        ];
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
        assert_eq!(plans[0].recovery_classification, "FULL_ANONYMOUS_VERIFIED");
    }
}
