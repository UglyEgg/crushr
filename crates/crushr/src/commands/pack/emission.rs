// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn emit_archive_from_layout(
    layout: PackLayoutPlan,
    output: &Path,
    level: i32,
    options: PackExperimentalOptions,
    phase_timings: Option<&mut PackPhaseTimings>,
    progress: impl FnMut(PackProgressPhase, u64, u64),
) -> Result<()> {
    emit_archive_from_layout_impl(layout, output, level, options, phase_timings, progress)
}

fn emit_archive_from_layout_impl(
    layout: PackLayoutPlan,
    output: &Path,
    level: i32,
    options: PackExperimentalOptions,
    phase_timings: Option<&mut PackPhaseTimings>,
    mut progress: impl FnMut(PackProgressPhase, u64, u64),
) -> Result<()> {
    let mut phase_timings = phase_timings;
    let total_files = layout.files.len();

    let out_file = File::create(output).with_context(|| format!("create {}", output.display()))?;
    let mut out = BufWriter::with_capacity(1024 * 1024, out_file);
    let mut write_offset = 0u64;
    let mut entries = Vec::with_capacity(total_files);
    let mut compression = DeterministicCompressor::new(level)?;

    let mut experimental_records = Vec::new();
    let mut file_identity_extent_records = Vec::new();
    let mut file_identity_path_records = Vec::new();
    let mut payload_block_identity_records = Vec::new();
    let mut path_checkpoint_entries = Vec::new();
    let mut file_manifest_records = Vec::new();
    let emit_payload_identity = layout.metadata.emit_payload_identity;
    let emit_path_checkpoints = layout.metadata.emit_path_checkpoints;
    let emit_manifest_checkpoints = layout.metadata.emit_manifest_checkpoints;
    let use_path_dictionary = layout.metadata.use_path_dictionary;
    let inline_payload_path = layout.metadata.inline_payload_path;
    let file_identity_archive_id = layout.metadata.file_identity_archive_id.clone();
    let payload_identity_archive_id = layout.metadata.payload_identity_archive_id.clone();
    let path_id_by_path = &layout.metadata.dictionary.path_id_by_path;
    let quasi_uniform_ordinals = &layout.metadata.dictionary.quasi_uniform_ordinals;
    let checkpoint_stride = 2usize;
    if let Some(path_dictionary) = &layout.metadata.dictionary.primary_copy {
        write_experimental_metadata_block(
            &mut out,
            path_dictionary,
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }
    let mut payload_materialized_by_block =
        BTreeMap::<u32, (u64, u64, [u8; 32], [u8; 32], u64)>::new();
    for (ordinal, file) in layout.files.into_iter().enumerate() {
        let current_meta = std::fs::metadata(&file.abs_path)
            .with_context(|| format!("stat {}", file.abs_path.display()))?;
        let current = discovery::capture_mode_mtime_uid_gid(&current_meta);
        if current_meta.len() != file.raw_len || (file.mtime >= 0 && current.mtime != file.mtime) {
            bail!(
                "input changed during pack planning: {}",
                file.abs_path.display()
            );
        }
        let (raw_len, compressed_len, payload_hash, raw_hash, block_scan_offset) =
            if file.write_payload {
                let raw = if file.sparse_chunks.is_empty() {
                    std::fs::read(&file.abs_path)
                        .with_context(|| format!("read {}", file.abs_path.display()))?
                } else {
                    use std::os::unix::fs::FileExt;
                    let source = std::fs::File::open(&file.abs_path)
                        .with_context(|| format!("open {}", file.abs_path.display()))?;
                    let mut packed = Vec::new();
                    for chunk in &file.sparse_chunks {
                        let mut left = chunk.len;
                        let mut src_off = chunk.logical_offset;
                        while left > 0 {
                            let step = left.min(1024 * 1024) as usize;
                            let mut buf = vec![0u8; step];
                            let n = source.read_at(&mut buf, src_off).with_context(|| {
                                format!("read {} at {}", file.abs_path.display(), src_off)
                            })?;
                            if n == 0 {
                                bail!("unexpected EOF while reading sparse chunk");
                            }
                            packed.extend_from_slice(&buf[..n]);
                            left -= n as u64;
                            src_off += n as u64;
                        }
                    }
                    packed
                };
                let raw_len = raw.len() as u64;
                let expected_len = if file.sparse_chunks.is_empty() {
                    file.raw_len
                } else {
                    file.sparse_chunks.iter().map(|chunk| chunk.len).sum()
                };
                if raw_len != expected_len {
                    bail!(
                        "input changed during pack planning: {}",
                        file.abs_path.display()
                    );
                }
                let compression_start = Instant::now();
                let compressed = compression
                    .compress(&raw)
                    .with_context(|| format!("compress {}", file.abs_path.display()))?;
                if let Some(timings) = phase_timings.as_mut() {
                    (*timings).add(PackProfilePhase::Compression, compression_start.elapsed());
                }
                let block_scan_offset = write_offset;
                let hashing_start = Instant::now();
                let payload_hash = *blake3::hash(compressed).as_bytes();
                let raw_hash = *blake3::hash(&raw).as_bytes();
                if let Some(timings) = phase_timings.as_mut() {
                    (*timings).add(PackProfilePhase::Hashing, hashing_start.elapsed());
                }
                let flags = Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH);
                let header = Blk3Header {
                    header_len: BLK3_HEADER_WITH_HASHES_LEN as u16,
                    flags,
                    codec: ZSTD_CODEC,
                    level,
                    dict_id: 0,
                    raw_len,
                    comp_len: compressed.len() as u64,
                    payload_hash: Some(payload_hash),
                    raw_hash: Some(raw_hash),
                };

                let emission_start = Instant::now();
                write_blk3_header(&mut out, &header)?;
                out.write_all(compressed)?;
                let compressed_len = compressed.len() as u64;
                write_offset += BLK3_HEADER_WITH_HASHES_LEN + compressed_len;
                if let Some(timings) = phase_timings.as_mut() {
                    (*timings).add(PackProfilePhase::Emission, emission_start.elapsed());
                }

                payload_materialized_by_block.insert(
                    file.block_id,
                    (
                        raw_len,
                        compressed_len,
                        payload_hash,
                        raw_hash,
                        block_scan_offset,
                    ),
                );
                (
                    raw_len,
                    compressed_len,
                    payload_hash,
                    raw_hash,
                    block_scan_offset,
                )
            } else {
                let (raw_len, compressed_len, payload_hash, raw_hash, block_scan_offset) =
                    payload_materialized_by_block
                        .get(&file.block_id)
                        .cloned()
                        .with_context(|| {
                            format!(
                                "missing hard-link payload source for block {}",
                                file.block_id
                            )
                        })?;
                (
                    raw_len,
                    compressed_len,
                    payload_hash,
                    raw_hash,
                    block_scan_offset,
                )
            };
        progress(
            PackProgressPhase::Compression,
            (ordinal + 1) as u64,
            total_files as u64,
        );

        if options.self_describing_extents {
            let record = build_self_describing_extent_record(
                file.file_id,
                &file.rel_path,
                raw_len,
                &payload_hash,
                &raw_hash,
            );
            experimental_records.push(record.clone());
            write_experimental_metadata_block(
                &mut out,
                &wrap_self_describing_extent(record),
                level,
                &mut compression,
                &mut write_offset,
                phase_timings.as_deref_mut(),
            )?;

            if (ordinal + 1) % checkpoint_stride == 0 {
                write_experimental_metadata_block(
                    &mut out,
                    &build_checkpoint_map_snapshot(
                        ((ordinal + 1) / checkpoint_stride) as u64,
                        &experimental_records,
                    ),
                    level,
                    &mut compression,
                    &mut write_offset,
                    phase_timings.as_deref_mut(),
                )?;
            }
        }

        if options.file_identity_extents {
            let path = file.rel_path.clone();
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            file_identity_extent_records.push(build_file_identity_extent_record(
                file.file_id,
                raw_len,
                &payload_hash,
                &raw_hash,
                block_scan_offset,
                &path_digest,
            ));
            file_identity_path_records.push(build_file_identity_path_record(
                file.file_id,
                &path,
                &path_digest,
            ));

            write_experimental_metadata_block(
                &mut out,
                file_identity_extent_records
                    .last()
                    .context("missing file identity record")?,
                level,
                &mut compression,
                &mut write_offset,
                phase_timings.as_deref_mut(),
            )?;
            write_experimental_metadata_block(
                &mut out,
                &build_file_path_map_entry(file.file_id, &path, &path_digest),
                level,
                &mut compression,
                &mut write_offset,
                phase_timings.as_deref_mut(),
            )?;
            if should_emit_anchor(ordinal, total_files) {
                write_experimental_metadata_block(
                    &mut out,
                    &build_bootstrap_anchor(
                        ordinal as u64,
                        file_identity_archive_id.clone(),
                        file_identity_extent_records.len() as u64,
                    ),
                    level,
                    &mut compression,
                    &mut write_offset,
                    phase_timings.as_deref_mut(),
                )?;
            }
        }

        if emit_payload_identity {
            let archive_identity = payload_identity_archive_id.clone();
            let path = file.rel_path.clone();
            let name = Path::new(&path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let hashing_start = Instant::now();
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            if let Some(timings) = phase_timings.as_mut() {
                (*timings).add(PackProfilePhase::Hashing, hashing_start.elapsed());
            }
            let path_id = path_id_by_path.get(&path).copied();
            let payload_record = build_payload_block_identity_record(
                PayloadIdentityInput {
                    file_id: file.file_id,
                    raw_len,
                    compressed_len,
                    payload_hash: &payload_hash,
                    raw_hash: &raw_hash,
                    block_scan_offset,
                },
                archive_identity,
                inline_payload_path.then_some(name),
                inline_payload_path.then_some(path.clone()),
                inline_payload_path.then_some(to_hex(&path_digest)),
                use_path_dictionary.then_some(path_id).flatten(),
            );
            payload_block_identity_records.push(payload_record.clone());
            write_experimental_metadata_block(
                &mut out,
                &payload_record,
                level,
                &mut compression,
                &mut write_offset,
                phase_timings.as_deref_mut(),
            )?;

            if use_path_dictionary && quasi_uniform_ordinals.contains(&ordinal) {
                let mut copy = layout
                    .metadata
                    .dictionary
                    .primary_copy
                    .clone()
                    .context("missing primary dictionary copy for interior mirror")?;
                copy.copy_role = "interior_mirror";
                write_experimental_metadata_block(
                    &mut out,
                    &copy,
                    level,
                    &mut compression,
                    &mut write_offset,
                    phase_timings.as_deref_mut(),
                )?;
            }

            if emit_path_checkpoints {
                path_checkpoint_entries.push(build_path_checkpoint_entry(
                    file.file_id,
                    &path,
                    &path_digest,
                    raw_len,
                ));

                if should_emit_anchor(ordinal, total_files)
                    || layout.metadata.path_checkpoint_ordinals.contains(&ordinal)
                {
                    write_experimental_metadata_block(
                        &mut out,
                        &build_path_checkpoint_snapshot(
                            ordinal as u64,
                            options.placement_strategy,
                            &path_checkpoint_entries,
                        ),
                        level,
                        &mut compression,
                        &mut write_offset,
                        phase_timings.as_deref_mut(),
                    )?;
                }
            }
        }

        if emit_manifest_checkpoints {
            let manifest_record =
                build_file_manifest_record(file.file_id, &file.rel_path, &raw_hash, raw_len);
            file_manifest_records.push(manifest_record.clone());
            write_experimental_metadata_block(
                &mut out,
                &manifest_record,
                level,
                &mut compression,
                &mut write_offset,
                phase_timings.as_deref_mut(),
            )?;

            if should_emit_anchor(ordinal, total_files)
                || layout
                    .metadata
                    .manifest_checkpoint_ordinals
                    .contains(&ordinal)
            {
                write_experimental_metadata_block(
                    &mut out,
                    &build_manifest_checkpoint_snapshot(
                        ordinal as u64,
                        options.placement_strategy,
                        &file_manifest_records,
                    ),
                    level,
                    &mut compression,
                    &mut write_offset,
                    phase_timings.as_deref_mut(),
                )?;
            }
        }

        let extents = if file.sparse_chunks.is_empty() {
            vec![Extent {
                block_id: file.block_id,
                offset: 0,
                len: raw_len,
                logical_offset: 0,
            }]
        } else {
            let mut block_offset = 0u64;
            let mut out = Vec::with_capacity(file.sparse_chunks.len());
            for chunk in &file.sparse_chunks {
                out.push(Extent {
                    block_id: file.block_id,
                    offset: block_offset,
                    len: chunk.len,
                    logical_offset: chunk.logical_offset,
                });
                block_offset += chunk.len;
            }
            out
        };
        entries.push(Entry {
            path: file.rel_path,
            kind: EntryKind::Regular,
            mode: file.mode,
            mtime: file.mtime,
            size: file.raw_len,
            extents,
            link_target: None,
            xattrs: file.xattrs,
            uid: file.uid,
            gid: file.gid,
            uname: file.uname,
            gname: file.gname,
            hardlink_group_id: file.hardlink_group_id,
            sparse: !file.sparse_chunks.is_empty(),
            device_major: None,
            device_minor: None,
            acl_access: file.acl_access,
            acl_default: file.acl_default,
            selinux_label: file.selinux_label,
            linux_capability: file.linux_capability,
        });
        progress(
            PackProgressPhase::Serialization,
            (ordinal + 1) as u64,
            total_files as u64,
        );
    }

    for input in &layout.profile_plan.included {
        if input.kind == EntryKind::Regular {
            continue;
        }
        let link_target = if input.kind == EntryKind::Symlink {
            Some(
                std::fs::read_link(&input.abs_path)
                    .with_context(|| format!("readlink {}", input.abs_path.display()))?
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        };
        entries.push(Entry {
            path: input.rel_path.clone(),
            kind: input.kind,
            mode: input.mode,
            mtime: input.mtime,
            size: 0,
            extents: Vec::new(),
            link_target,
            xattrs: input.xattrs.clone(),
            uid: input.uid,
            gid: input.gid,
            uname: input.uname.clone(),
            gname: input.gname.clone(),
            hardlink_group_id: None,
            sparse: false,
            device_major: input.device_major,
            device_minor: input.device_minor,
            acl_access: input.acl_access.clone(),
            acl_default: input.acl_default.clone(),
            selinux_label: input.selinux_label.clone(),
            linux_capability: input.linux_capability.clone(),
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    if layout.metadata.dictionary.tail_copy_required {
        let mut copy = layout
            .metadata
            .dictionary
            .primary_copy
            .clone()
            .context("missing primary dictionary copy for tail mirror")?;
        copy.copy_role = "tail_mirror";
        write_experimental_metadata_block(
            &mut out,
            &copy,
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    let finalization_start = Instant::now();
    if options.self_describing_extents {
        write_experimental_metadata_block(
            &mut out,
            &build_checkpoint_map_snapshot(u64::MAX, &experimental_records),
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    if options.file_identity_extents {
        write_experimental_metadata_block(
            &mut out,
            &build_bootstrap_anchor(
                u64::MAX,
                file_identity_archive_id.clone(),
                file_identity_extent_records.len() as u64,
            ),
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
        write_experimental_metadata_block(
            &mut out,
            &FilePathMapRecord {
                schema: "crushr-file-path-map.v1",
                records: file_identity_path_records,
            },
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    if emit_path_checkpoints {
        write_experimental_metadata_block(
            &mut out,
            &build_path_checkpoint_snapshot(
                u64::MAX,
                options.placement_strategy,
                &path_checkpoint_entries,
            ),
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    if emit_payload_identity {
        write_experimental_metadata_block(
            &mut out,
            &PayloadBlockIdentitySummary {
                schema: "crushr-payload-block-identity-summary.v1",
                records_emitted: payload_block_identity_records.len() as u64,
            },
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    if emit_manifest_checkpoints {
        write_experimental_metadata_block(
            &mut out,
            &build_manifest_checkpoint_snapshot(
                u64::MAX,
                options.placement_strategy,
                &file_manifest_records,
            ),
            level,
            &mut compression,
            &mut write_offset,
            phase_timings.as_deref_mut(),
        )?;
    }

    let blocks_end_offset = write_offset;
    write_tail_with_redundant_map(
        &mut out,
        blocks_end_offset,
        &entries,
        options,
        emit_payload_identity,
        emit_path_checkpoints,
        emit_manifest_checkpoints,
    )?;
    out.flush()?;
    if let Some(timings) = phase_timings.as_mut() {
        (*timings).add(PackProfilePhase::Finalization, finalization_start.elapsed());
    }

    Ok(())
}

fn build_self_describing_extent_record(
    file_id: u32,
    rel_path: &str,
    raw_len: u64,
    payload_hash: &[u8; 32],
    raw_hash: &[u8; 32],
) -> SelfDescribingExtentRecord {
    SelfDescribingExtentRecord {
        file_id,
        path: rel_path.to_string(),
        logical_offset: 0,
        logical_length: raw_len,
        full_file_size: raw_len,
        extent_ordinal: 0,
        block_id: file_id,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(payload_hash),
            raw_hash_blake3: to_hex(raw_hash),
        },
    }
}

fn wrap_self_describing_extent(record: SelfDescribingExtentRecord) -> SelfDescribingExtentEnvelope {
    SelfDescribingExtentEnvelope {
        schema: "crushr-self-describing-extent.v1",
        record,
    }
}

fn build_checkpoint_map_snapshot(
    checkpoint_ordinal: u64,
    records: &[SelfDescribingExtentRecord],
) -> CheckpointMapSnapshot {
    CheckpointMapSnapshot {
        schema: "crushr-checkpoint-map-snapshot.v1",
        checkpoint_ordinal,
        records: records.to_vec(),
    }
}

fn build_file_identity_extent_record(
    file_id: u32,
    raw_len: u64,
    payload_hash: &[u8; 32],
    raw_hash: &[u8; 32],
    block_scan_offset: u64,
    path_digest: &[u8; 32],
) -> FileIdentityExtentRecord {
    FileIdentityExtentRecord {
        schema: "crushr-file-identity-extent.v1",
        file_id,
        logical_offset: 0,
        logical_length: raw_len,
        full_file_size: raw_len,
        extent_ordinal: 0,
        block_id: file_id,
        block_scan_offset,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(payload_hash),
            raw_hash_blake3: to_hex(raw_hash),
        },
        path_linkage: PathLinkage {
            path_digest_blake3: to_hex(path_digest),
        },
    }
}

fn build_file_identity_path_record(
    file_id: u32,
    path: &str,
    path_digest: &[u8; 32],
) -> FileIdentityPathRecord {
    FileIdentityPathRecord {
        file_id,
        path: path.to_string(),
        path_digest_blake3: to_hex(path_digest),
    }
}

fn build_file_path_map_entry(
    file_id: u32,
    path: &str,
    path_digest: &[u8; 32],
) -> FilePathMapEntryRecord {
    FilePathMapEntryRecord {
        schema: "crushr-file-path-map-entry.v1",
        file_id,
        path: path.to_string(),
        path_digest_blake3: to_hex(path_digest),
    }
}

fn build_bootstrap_anchor(
    anchor_ordinal: u64,
    archive_identity: Option<String>,
    records_emitted: u64,
) -> BootstrapAnchorRecord {
    BootstrapAnchorRecord {
        schema: "crushr-bootstrap-anchor.v1",
        anchor_ordinal,
        archive_identity,
        records_emitted,
    }
}

fn build_payload_block_identity_record(
    input: PayloadIdentityInput<'_>,
    archive_identity: Option<String>,
    inline_name: Option<String>,
    inline_path: Option<String>,
    inline_path_digest: Option<String>,
    path_id: Option<u32>,
) -> PayloadBlockIdentityRecord {
    PayloadBlockIdentityRecord {
        schema: "crushr-payload-block-identity.v1",
        archive_identity,
        file_id: input.file_id,
        block_id: input.file_id,
        block_index: 0,
        extent_index: 0,
        total_block_count: 1,
        total_extent_count: 1,
        full_file_size: input.raw_len,
        logical_offset: 0,
        payload_codec: ZSTD_CODEC,
        payload_length: input.compressed_len,
        logical_length: input.raw_len,
        extent_length: input.raw_len,
        block_scan_offset: input.block_scan_offset,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(input.payload_hash),
            raw_hash_blake3: to_hex(input.raw_hash),
        },
        name: inline_name,
        path: inline_path,
        path_digest_blake3: inline_path_digest,
        path_id,
    }
}

fn build_path_checkpoint_entry(
    file_id: u32,
    path: &str,
    path_digest: &[u8; 32],
    full_file_size: u64,
) -> PathCheckpointEntry {
    PathCheckpointEntry {
        file_id,
        path: path.to_string(),
        path_digest_blake3: to_hex(path_digest),
        full_file_size,
        total_block_count: 1,
    }
}

fn build_path_checkpoint_snapshot(
    checkpoint_ordinal: u64,
    placement_strategy: Option<PlacementStrategy>,
    entries: &[PathCheckpointEntry],
) -> PathCheckpointSnapshot {
    PathCheckpointSnapshot {
        schema: "crushr-path-checkpoint.v1",
        checkpoint_ordinal,
        placement_strategy: placement_strategy.map(|s| s.as_str()),
        entries: entries.to_vec(),
    }
}

fn build_file_manifest_record(
    file_id: u32,
    rel_path: &str,
    raw_hash: &[u8; 32],
    raw_len: u64,
) -> FileManifestRecord {
    FileManifestRecord {
        schema: "crushr-file-manifest.v1",
        file_id,
        path: rel_path.to_string(),
        file_size: raw_len,
        expected_block_count: 1,
        extent_count: 1,
        file_digest: to_hex(raw_hash),
    }
}

fn build_manifest_checkpoint_snapshot(
    checkpoint_ordinal: u64,
    placement_strategy: Option<PlacementStrategy>,
    records: &[FileManifestRecord],
) -> FileManifestCheckpointSnapshot {
    FileManifestCheckpointSnapshot {
        schema: "crushr-file-manifest-checkpoint.v1",
        checkpoint_ordinal,
        placement_strategy: placement_strategy.map(|s| s.as_str()),
        records: records.to_vec(),
    }
}

fn should_emit_anchor(ordinal: usize, total: usize) -> bool {
    total == 1 || ordinal == 0 || ordinal + 1 == total
}

fn write_tail_with_redundant_map<W: Write>(
    out: &mut W,
    blocks_end_offset: u64,
    entries: &[Entry],
    options: PackExperimentalOptions,
    emit_payload_identity: bool,
    emit_path_checkpoints: bool,
    emit_manifest_checkpoints: bool,
) -> Result<()> {
    let idx3 = encode_index(&Index {
        preservation_profile: options.preservation_profile,
        entries: entries.to_vec(),
    });
    let redundant_file_map = build_redundant_file_map(
        entries,
        options,
        emit_payload_identity,
        emit_path_checkpoints,
        emit_manifest_checkpoints,
    );
    let ledger = LedgerBlob::from_value(&serde_json::to_value(&redundant_file_map)?)?;
    let tail = assemble_tail_frame(blocks_end_offset, None, &idx3, Some(&ledger))?;
    out.write_all(&tail)?;
    Ok(())
}

fn build_redundant_file_map(
    entries: &[Entry],
    options: PackExperimentalOptions,
    emit_payload_identity: bool,
    emit_path_checkpoints: bool,
    emit_manifest_checkpoints: bool,
) -> RedundantFileMap {
    RedundantFileMap {
        schema: if options.self_describing_extents
            || options.file_identity_extents
            || emit_payload_identity
            || emit_path_checkpoints
            || emit_manifest_checkpoints
        {
            "crushr-redundant-file-map.experimental.v2"
        } else {
            "crushr-redundant-file-map.v1"
        },
        experimental_self_describing_extents: options.self_describing_extents,
        experimental_file_identity_extents: options.file_identity_extents,
        experimental_self_identifying_blocks: emit_payload_identity,
        experimental_path_checkpoints: emit_path_checkpoints,
        experimental_file_manifest_checkpoints: emit_manifest_checkpoints,
        experimental_metadata_profile: options.metadata_profile.map(|profile| profile.as_str()),
        metadata_placement_strategy: options.placement_strategy.map(|s| s.as_str()),
        files: entries
            .iter()
            .map(|entry| RedundantFileMapFile {
                path: entry.path.clone(),
                size: entry.size,
                extents: entry
                    .extents
                    .iter()
                    .map(|extent| RedundantFileMapExtent {
                        block_id: extent.block_id,
                        file_offset: extent.offset,
                        len: extent.len,
                    })
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>(),
    }
}

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn write_experimental_metadata_block<T: Serialize>(
    out: &mut BufWriter<File>,
    value: &T,
    level: i32,
    compression: &mut DeterministicCompressor,
    write_offset: &mut u64,
    phase_timings: Option<&mut PackPhaseTimings>,
) -> Result<()> {
    let mut phase_timings = phase_timings;
    let raw = serde_json::to_vec(value)?;
    let compression_start = Instant::now();
    let compressed = compression.compress(&raw)?;
    if let Some(timings) = phase_timings.as_mut() {
        (*timings).add(PackProfilePhase::Compression, compression_start.elapsed());
    }
    let hashing_start = Instant::now();
    let payload_hash = *blake3::hash(compressed).as_bytes();
    let raw_hash = *blake3::hash(&raw).as_bytes();
    if let Some(timings) = phase_timings.as_mut() {
        (*timings).add(PackProfilePhase::Hashing, hashing_start.elapsed());
    }
    let header = Blk3Header {
        header_len: BLK3_HEADER_WITH_HASHES_LEN as u16,
        flags: Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH),
        codec: ZSTD_CODEC,
        level,
        dict_id: 0,
        raw_len: raw.len() as u64,
        comp_len: compressed.len() as u64,
        payload_hash: Some(payload_hash),
        raw_hash: Some(raw_hash),
    };
    let emission_start = Instant::now();
    write_blk3_header(&mut *out, &header)?;
    out.write_all(compressed)?;
    *write_offset += BLK3_HEADER_WITH_HASHES_LEN + compressed.len() as u64;
    if let Some(timings) = phase_timings.as_mut() {
        (*timings).add(PackProfilePhase::Emission, emission_start.elapsed());
    }
    Ok(())
}
