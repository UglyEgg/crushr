use super::*;

pub(super) fn decode_verified_blocks(
    archive_bytes: &[u8],
    candidates: &[BlockCandidate],
) -> BTreeMap<u32, BlockExportData> {
    let mut out = BTreeMap::new();

    for candidate in candidates {
        let Some(block_id) = candidate.mapped_block_id else {
            continue;
        };
        if candidate.content_verification_status != "content_verified" {
            continue;
        }

        let offset = candidate.scan_offset as usize;
        if offset + 6 > archive_bytes.len() {
            continue;
        }
        let header_len =
            u16::from_le_bytes([archive_bytes[offset + 4], archive_bytes[offset + 5]]) as usize;
        if offset + header_len > archive_bytes.len() {
            continue;
        }

        let Ok(header) = read_blk3_header(Cursor::new(&archive_bytes[offset..offset + header_len]))
        else {
            continue;
        };

        let payload_offset = offset + header.header_len as usize;
        let Some(payload_end) = payload_offset.checked_add(header.comp_len as usize) else {
            continue;
        };
        if payload_end > archive_bytes.len() || header.codec != 1 {
            continue;
        }

        let Ok(raw) = zstd::decode_all(Cursor::new(&archive_bytes[payload_offset..payload_end]))
        else {
            continue;
        };

        if let Some(raw_hash) = header.raw_hash {
            if blake3::hash(&raw).as_bytes() != &raw_hash {
                continue;
            }
        }

        out.insert(
            block_id,
            BlockExportData {
                archive_offset: candidate.scan_offset,
                block_id,
                codec: header.codec,
                dictionary_dependency_status: candidate.dictionary_dependency_status,
                raw_hash: header.raw_hash.map(|v| to_hex(&v)),
                payload: raw,
            },
        );
    }

    out
}

pub(super) fn sanitize_component(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

pub(super) fn sanitize_rel_path(path: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for component in Path::new(path).components() {
        let text = component.as_os_str().to_string_lossy();
        if text == "/" || text == "." || text == ".." || text.is_empty() {
            continue;
        }
        out.push(sanitize_component(&text));
    }
    if out.as_os_str().is_empty() {
        out.push("_");
    }
    out
}

pub(super) fn write_json_output(path: &Path, rendered: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, rendered).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(super) fn export_artifacts(
    export_dir: &Path,
    archive_bytes: &[u8],
    candidates: &[BlockCandidate],
    file_plans: &[FilePlan],
) -> Result<ExportedArtifacts> {
    fs::create_dir_all(export_dir).with_context(|| format!("create {}", export_dir.display()))?;
    fs::write(
        export_dir.join("SALVAGE_RESEARCH_OUTPUT.txt"),
        "Output produced by crushr-salvage.\nArtifacts are UNVERIFIED research outputs and are not canonical extraction.\nArtifacts may be incomplete or partial. Use at your own risk.\n",
    )
    .with_context(|| format!("write {}", export_dir.join("SALVAGE_RESEARCH_OUTPUT.txt").display()))?;

    let mut exported = ExportedArtifacts::default();
    let verified_blocks = decode_verified_blocks(archive_bytes, candidates);

    let blocks_dir = export_dir.join("blocks");
    fs::create_dir_all(&blocks_dir).with_context(|| format!("create {}", blocks_dir.display()))?;

    let mut block_entries = verified_blocks.values().collect::<Vec<_>>();
    block_entries.sort_by_key(|b| b.archive_offset);

    for block in block_entries {
        let base = format!("block_{}_{}", block.archive_offset, block.block_id);
        let bin_rel = format!("blocks/{base}.bin");
        let json_rel = format!("blocks/{base}.json");
        fs::write(export_dir.join(&bin_rel), &block.payload)
            .with_context(|| format!("write {}", export_dir.join(&bin_rel).display()))?;
        let sidecar = serde_json::json!({
            "block_offset": block.archive_offset,
            "block_id": block.block_id,
            "codec": block.codec,
            "verification_status": "content_verified",
            "raw_hash": block.raw_hash,
            "dependency_state": block.dictionary_dependency_status,
            "verification_label": RESEARCH_LABEL,
        });
        fs::write(
            export_dir.join(&json_rel),
            serde_json::to_string_pretty(&sidecar)?,
        )
        .with_context(|| format!("write {}", export_dir.join(&json_rel).display()))?;
        exported.exported_block_artifacts.push(bin_rel);
        exported.exported_block_artifacts.push(json_rel);
    }

    let mut sorted_files = file_plans.iter().collect::<Vec<_>>();
    sorted_files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for file in sorted_files {
        let file_root = export_dir
            .join("files")
            .join(sanitize_rel_path(&file.file_path));
        fs::create_dir_all(&file_root)
            .with_context(|| format!("create {}", file_root.display()))?;

        let mut extents = file.extents.clone();
        extents.sort_by_key(|e| e.offset);

        let mut verified_extent_payloads = Vec::new();
        for (idx, extent) in extents.iter().enumerate() {
            let Some(block) = verified_blocks.get(&extent.block_id) else {
                continue;
            };
            let start = extent.offset as usize;
            let Some(end) = start.checked_add(extent.len as usize) else {
                continue;
            };
            if end > block.payload.len() {
                continue;
            }

            let payload = block.payload[start..end].to_vec();
            verified_extent_payloads.push((idx, extent, payload));
        }

        for (idx, extent, payload) in &verified_extent_payloads {
            let bin_rel = format!(
                "files/{}/extent_{}.bin",
                sanitize_rel_path(&file.file_path).display(),
                idx
            );
            let json_rel = format!(
                "files/{}/extent_{}.json",
                sanitize_rel_path(&file.file_path).display(),
                idx
            );
            fs::write(export_dir.join(&bin_rel), payload)
                .with_context(|| format!("write {}", export_dir.join(&bin_rel).display()))?;
            let sidecar = serde_json::json!({
                "original_file_path": file.file_path,
                "extent_index": idx,
                "offset_within_file": extent.offset,
                "source_block_id": extent.block_id,
                "verification_status": "content_verified",
                "verification_label": RESEARCH_LABEL,
            });
            fs::write(
                export_dir.join(&json_rel),
                serde_json::to_string_pretty(&sidecar)?,
            )
            .with_context(|| format!("write {}", export_dir.join(&json_rel).display()))?;
            exported.exported_fragment_artifacts.push(bin_rel);
            exported.exported_fragment_artifacts.push(json_rel);
        }

        let is_full = file.status == "SALVAGEABLE"
            && !extents.is_empty()
            && verified_extent_payloads.len() == extents.len()
            && extents.first().map(|e| e.offset) == Some(0)
            && extents
                .windows(2)
                .all(|w| w[0].offset.saturating_add(w[0].len) == w[1].offset)
            && extents
                .iter()
                .fold(0u64, |acc, e| acc.saturating_add(e.len))
                == file.file_size;

        if is_full {
            let mut buf = Vec::new();
            for (_, _, payload) in &verified_extent_payloads {
                buf.extend_from_slice(payload);
            }
            let rel = format!(
                "files/{}/file_verified.bin",
                sanitize_rel_path(&file.file_path).display()
            );
            fs::write(export_dir.join(&rel), buf)
                .with_context(|| format!("write {}", export_dir.join(&rel).display()))?;
            exported.exported_complete_file_artifacts.push(rel);
        }
    }

    Ok(exported)
}
