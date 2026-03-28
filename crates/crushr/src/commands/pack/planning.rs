// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn build_pack_layout_plan(
    profile_plan: PackProfilePlan,
    options: PackExperimentalOptions,
) -> Result<PackLayoutPlan> {
    build_pack_layout_plan_impl(profile_plan, options)
}

fn build_pack_layout_plan_impl(
    profile_plan: PackProfilePlan,
    options: PackExperimentalOptions,
) -> Result<PackLayoutPlan> {
    let files: Vec<&InputFile> = profile_plan
        .included
        .iter()
        .filter(|entry| entry.kind == EntryKind::Regular)
        .collect();
    let total_files = files.len();
    let placement_seed = compute_file_identity_archive_id(&files);
    let file_identity_archive_id = if options.file_identity_extents {
        Some(placement_seed.clone())
    } else {
        None
    };
    let emit_payload_identity =
        options.self_identifying_blocks || options.metadata_profile.is_some();
    let emit_path_checkpoints = options
        .metadata_profile
        .map(MetadataProfile::emit_path_checkpoints)
        .unwrap_or(options.self_identifying_blocks);
    let emit_manifest_checkpoints = options
        .metadata_profile
        .map(MetadataProfile::emit_manifest_checkpoints)
        .unwrap_or(options.file_manifest_checkpoints);
    let inline_payload_path = matches!(
        options.metadata_profile,
        Some(MetadataProfile::ExtentIdentityInlinePath)
    );
    let use_path_dictionary = options
        .metadata_profile
        .map(MetadataProfile::uses_path_dictionary)
        .unwrap_or(false);

    let mut path_id_by_path = BTreeMap::new();
    for (idx, file) in files.iter().enumerate() {
        path_id_by_path.insert(file.rel_path.clone(), idx as u32);
    }

    let payload_identity_archive_id = emit_payload_identity.then_some(placement_seed.clone());
    let path_checkpoint_ordinals = options
        .placement_strategy
        .filter(|_| emit_path_checkpoints)
        .map(|strategy| {
            scheduled_metadata_ordinals(strategy, "path_checkpoint", total_files, &placement_seed)
        })
        .unwrap_or_default();
    let manifest_checkpoint_ordinals = options
        .placement_strategy
        .filter(|_| emit_manifest_checkpoints)
        .map(|strategy| {
            scheduled_metadata_ordinals(
                strategy,
                "file_manifest_checkpoint",
                total_files,
                &placement_seed,
            )
        })
        .unwrap_or_default();

    let dictionary = build_dictionary_plan(
        &files,
        &path_id_by_path,
        &placement_seed,
        options.metadata_profile,
    )?;
    let mut planned_files = Vec::with_capacity(files.len());
    let mut hardlink_sources = BTreeMap::<(u64, u64), (u32, u64)>::new();
    let mut next_block_id = 0u32;
    let mut next_hardlink_group_id = 1u64;
    for (idx, file) in files.into_iter().enumerate() {
        let raw_len = file.raw_len;
        let (block_id, write_payload, hardlink_group_id) = if let Some(key) = file.hardlink_key {
            if let Some((existing_block_id, group_id)) = hardlink_sources.get(&key).copied() {
                (existing_block_id, false, Some(group_id))
            } else {
                let block_id = next_block_id;
                next_block_id = next_block_id
                    .checked_add(1)
                    .context("block id overflow while planning hard links")?;
                let group_id = next_hardlink_group_id;
                next_hardlink_group_id = next_hardlink_group_id
                    .checked_add(1)
                    .context("hard-link group id overflow")?;
                hardlink_sources.insert(key, (block_id, group_id));
                (block_id, true, Some(group_id))
            }
        } else {
            let block_id = next_block_id;
            next_block_id = next_block_id
                .checked_add(1)
                .context("block id overflow while planning payloads")?;
            (block_id, true, None)
        };
        planned_files.push(PlannedFileModel {
            file_id: idx as u32,
            block_id,
            write_payload,
            hardlink_group_id,
            rel_path: file.rel_path.clone(),
            abs_path: file.abs_path.clone(),
            raw_len,
            mode: file.mode,
            mtime: file.mtime,
            uid: file.uid,
            gid: file.gid,
            uname: file.uname.clone(),
            gname: file.gname.clone(),
            xattrs: file.xattrs.clone(),
            acl_access: file.acl_access.clone(),
            acl_default: file.acl_default.clone(),
            selinux_label: file.selinux_label.clone(),
            linux_capability: file.linux_capability.clone(),
            sparse_chunks: file.sparse_chunks.clone(),
        });
    }

    Ok(PackLayoutPlan {
        profile_plan,
        files: planned_files,
        metadata: MetadataPlan {
            emit_payload_identity,
            emit_path_checkpoints,
            emit_manifest_checkpoints,
            use_path_dictionary,
            inline_payload_path,
            file_identity_archive_id,
            payload_identity_archive_id,
            path_checkpoint_ordinals,
            manifest_checkpoint_ordinals,
            dictionary,
        },
    })
}

fn build_dictionary_plan(
    files: &[&InputFile],
    path_id_by_path: &BTreeMap<String, u32>,
    placement_seed: &str,
    metadata_profile: Option<MetadataProfile>,
) -> Result<DictionaryPlan> {
    let use_path_dictionary = metadata_profile
        .map(MetadataProfile::uses_path_dictionary)
        .unwrap_or(false);
    let tail_copy_required = matches!(
        metadata_profile,
        Some(MetadataProfile::ExtentIdentityPathDictHeaderTail)
            | Some(MetadataProfile::ExtentIdentityPathDictQuasiUniform)
            | Some(MetadataProfile::ExtentIdentityPathDictFactoredHeaderTail)
    );
    let quasi_uniform_ordinals = if matches!(
        metadata_profile,
        Some(MetadataProfile::ExtentIdentityPathDictQuasiUniform)
    ) {
        scheduled_metadata_ordinals(
            PlacementStrategy::Golden,
            "path_dictionary",
            files.len(),
            placement_seed,
        )
    } else {
        BTreeSet::new()
    };
    if !use_path_dictionary {
        return Ok(DictionaryPlan {
            path_id_by_path: path_id_by_path.clone(),
            primary_copy: None,
            tail_copy_required: false,
            quasi_uniform_ordinals,
        });
    }
    let dictionary_archive_instance_id = placement_seed.to_string();
    let dictionary_generation = 1u64;
    let factored_dictionary = matches!(
        metadata_profile,
        Some(MetadataProfile::ExtentIdentityPathDictFactoredHeaderTail)
    );
    let path_dictionary_body = if factored_dictionary {
        let mut dir_id_by_path = BTreeMap::<String, u32>::new();
        let mut name_id_by_name = BTreeMap::<String, u32>::new();
        let mut next_dir_id = 0u32;
        let mut next_name_id = 0u32;
        for path in path_id_by_path.keys() {
            let path_obj = Path::new(path);
            let dir = path_obj
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if !dir_id_by_path.contains_key(&dir) {
                dir_id_by_path.insert(dir.clone(), next_dir_id);
                next_dir_id += 1;
            }
            let name = path_obj
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            if let std::collections::btree_map::Entry::Vacant(entry) = name_id_by_name.entry(name) {
                entry.insert(next_name_id);
                next_name_id += 1;
            }
        }
        let directories: Vec<FactoredDirectory> = dir_id_by_path
            .iter()
            .map(|(dir, dir_id)| FactoredDirectory {
                dir_id: *dir_id,
                prefix: dir.clone(),
            })
            .collect();
        let basenames: Vec<FactoredBasename> = name_id_by_name
            .iter()
            .map(|(name, name_id)| FactoredBasename {
                name_id: *name_id,
                basename: name.clone(),
            })
            .collect();
        let file_bindings: Vec<FactoredFileBinding> = path_id_by_path
            .iter()
            .map(|(path, path_id)| {
                let path_obj = Path::new(path);
                let dir = path_obj
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let name = path_obj
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                let dir_id = *dir_id_by_path.get(&dir).expect("dir id");
                let name_id = *name_id_by_name.get(&name).expect("name id");
                FactoredFileBinding {
                    path_id: *path_id,
                    dir_id,
                    name_id,
                    path_digest_blake3: to_hex(blake3::hash(path.as_bytes()).as_bytes()),
                }
            })
            .collect();
        PathDictionaryBody::FactoredNamespace {
            entry_count: path_id_by_path.len() as u64,
            directory_count: directories.len() as u64,
            basename_count: basenames.len() as u64,
            directories,
            basenames,
            file_bindings,
        }
    } else {
        let entries: Vec<PathDictionaryEntry> = path_id_by_path
            .iter()
            .map(|(path, path_id)| PathDictionaryEntry {
                path_id: *path_id,
                path: path.clone(),
                path_digest_blake3: to_hex(blake3::hash(path.as_bytes()).as_bytes()),
            })
            .collect();
        PathDictionaryBody::FullPath {
            entry_count: entries.len() as u64,
            entries,
        }
    };
    let path_dictionary_body_bytes = serde_json::to_vec(&path_dictionary_body)?;
    let dictionary_content_hash = to_hex(blake3::hash(&path_dictionary_body_bytes).as_bytes());
    let dictionary_uuid = to_hex(
        blake3::hash(
            format!(
                "{}:{}",
                dictionary_archive_instance_id, dictionary_content_hash
            )
            .as_bytes(),
        )
        .as_bytes(),
    );
    Ok(DictionaryPlan {
        path_id_by_path: path_id_by_path.clone(),
        primary_copy: Some(PathDictionaryCopyRecordV2 {
            schema: "crushr-path-dictionary-copy.v2",
            copy_role: "primary",
            archive_instance_id: dictionary_archive_instance_id,
            dict_uuid: dictionary_uuid,
            generation: dictionary_generation,
            dictionary_length: path_dictionary_body_bytes.len() as u64,
            dictionary_content_hash,
            body: path_dictionary_body,
        }),
        tail_copy_required,
        quasi_uniform_ordinals,
    })
}

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn compute_file_identity_archive_id(files: &[&InputFile]) -> String {
    let mut hasher = blake3::Hasher::new();
    for file in files {
        hasher.update(file.rel_path.as_bytes());
        hasher.update(&[0u8]);
    }
    hasher.finalize().to_hex().to_string()
}

fn scheduled_metadata_ordinals(
    strategy: PlacementStrategy,
    label: &str,
    total_files: usize,
    seed_material: &str,
) -> BTreeSet<usize> {
    if total_files == 0 {
        return BTreeSet::new();
    }
    let target = total_files.min(3);
    match strategy {
        PlacementStrategy::Fixed => {
            let mut set = BTreeSet::new();
            set.insert(0);
            set.insert(total_files / 2);
            set.insert(total_files - 1);
            set
        }
        PlacementStrategy::Hash => hashed_ordinals(label, total_files, target, seed_material),
        PlacementStrategy::Golden => {
            golden_ratio_ordinals(label, total_files, target, seed_material)
        }
    }
}

fn hashed_ordinals(
    label: &str,
    total_files: usize,
    target: usize,
    seed_material: &str,
) -> BTreeSet<usize> {
    let mut set = BTreeSet::new();
    let mut counter = 0u64;
    while set.len() < target {
        let mut hasher = blake3::Hasher::new();
        hasher.update(seed_material.as_bytes());
        hasher.update(label.as_bytes());
        hasher.update(&counter.to_le_bytes());
        let digest = hasher.finalize();
        let mut candidate =
            u64::from_le_bytes(digest.as_bytes()[0..8].try_into().unwrap()) as usize % total_files;
        while set.contains(&candidate) {
            candidate = (candidate + 1) % total_files;
        }
        set.insert(candidate);
        counter += 1;
    }
    set
}

fn golden_ratio_ordinals(
    label: &str,
    total_files: usize,
    target: usize,
    seed_material: &str,
) -> BTreeSet<usize> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed_material.as_bytes());
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let seed = u64::from_le_bytes(digest.as_bytes()[0..8].try_into().unwrap()) as f64;
    let seed_fraction = seed / u64::MAX as f64;
    let step = 0.6180339887498949_f64;
    let mut set = BTreeSet::new();
    let mut i = 0usize;
    while set.len() < target {
        let value = (seed_fraction + (i as f64) * step).fract();
        let mut candidate = (value * total_files as f64).floor() as usize;
        if candidate >= total_files {
            candidate = total_files - 1;
        }
        while set.contains(&candidate) {
            candidate = (candidate + 1) % total_files;
        }
        set.insert(candidate);
        i += 1;
    }
    set
}
