use anyhow::{bail, Context, Result};
use crushr::format::{Entry, EntryKind, Extent, Index};
use crushr::index_codec::encode_index;
use crushr_format::blk3::{write_blk3_header, Blk3Flags, Blk3Header};
use crushr_format::ledger::LedgerBlob;
use crushr_format::tailframe::assemble_tail_frame;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

const ZSTD_CODEC: u32 = 1;
const USAGE: &str = "usage: crushr-pack <input>... -o <archive> [--level <n>] [--experimental-self-describing-extents] [--experimental-file-identity-extents] [--experimental-self-identifying-blocks] [--experimental-file-manifest-checkpoints]\n\nFlags:\n  -o, --output <archive>                     output archive path\n  --level <n>                                zstd compression level (default: 3)\n  --experimental-self-describing-extents     emit self-describing extent + checkpoint metadata\n  --experimental-file-identity-extents       emit file-identity extent + verified path-map metadata + distributed bootstrap anchors\n  --experimental-self-identifying-blocks     emit payload block identity + repeated verified path checkpoints\n  --experimental-file-manifest-checkpoints   emit distributed file-manifest checkpoints for recovery verification\n  -h, --help                                 print this help text";

fn compress_deterministic(raw: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = zstd::Encoder::new(Vec::new(), level).context("create zstd encoder")?;
    encoder
        .include_checksum(false)
        .context("set zstd checksum flag")?;
    encoder
        .include_contentsize(true)
        .context("set zstd content-size flag")?;
    encoder
        .include_dictid(false)
        .context("set zstd dict-id flag")?;
    encoder.write_all(raw).context("zstd write")?;
    encoder.finish().context("zstd finish")
}

#[derive(Debug)]
struct InputFile {
    rel_path: String,
    abs_path: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        let message = format!("{err:#}");
        let code = if message.contains("usage:")
            || message.contains("unsupported flag")
            || message.contains("unexpected argument")
        {
            1
        } else {
            2
        };
        std::process::exit(code);
    }
}

fn run() -> Result<()> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut level: i32 = 3;
    let mut experimental_self_describing_extents = false;
    let mut experimental_file_identity_extents = false;
    let mut experimental_self_identifying_blocks = false;
    let mut experimental_file_manifest_checkpoints = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" || arg == "help" {
            println!("{USAGE}");
            return Ok(());
        }
        if arg == "-o" || arg == "--output" {
            let value = args.next().context(USAGE)?;
            output = Some(PathBuf::from(value));
        } else if arg == "--level" {
            let value = args.next().context(USAGE)?;
            level = value
                .parse::<i32>()
                .with_context(|| format!("invalid --level value: {value}"))?;
        } else if arg == "--experimental-self-describing-extents" {
            experimental_self_describing_extents = true;
        } else if arg == "--experimental-file-identity-extents" {
            experimental_file_identity_extents = true;
        } else if arg == "--experimental-self-identifying-blocks" {
            experimental_self_identifying_blocks = true;
        } else if arg == "--experimental-file-manifest-checkpoints" {
            experimental_file_manifest_checkpoints = true;
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else {
            inputs.push(PathBuf::from(arg));
        }
    }

    let output = output.context(USAGE)?;
    if inputs.is_empty() {
        bail!(USAGE);
    }

    pack_minimal_v1(
        &inputs,
        &output,
        level,
        experimental_self_describing_extents,
        experimental_file_identity_extents,
        experimental_self_identifying_blocks,
        experimental_file_manifest_checkpoints,
    )
}

fn pack_minimal_v1(
    inputs: &[PathBuf],
    output: &Path,
    level: i32,
    experimental_self_describing_extents: bool,
    experimental_file_identity_extents: bool,
    experimental_self_identifying_blocks: bool,
    experimental_file_manifest_checkpoints: bool,
) -> Result<()> {
    let files = collect_files(inputs)?;
    if files.is_empty() {
        bail!("no input files to pack");
    }
    reject_duplicate_logical_paths(&files)?;

    let mut out = File::create(output).with_context(|| format!("create {}", output.display()))?;
    let mut entries = Vec::with_capacity(files.len());
    let mut block_id: u32 = 0;

    let mut experimental_records = Vec::new();
    let mut file_identity_extent_records = Vec::new();
    let mut file_identity_path_records = Vec::new();
    let mut payload_block_identity_records = Vec::new();
    let mut path_checkpoint_entries = Vec::new();
    let mut file_manifest_records = Vec::new();
    let file_identity_archive_id = if experimental_file_identity_extents {
        Some(compute_file_identity_archive_id(&files))
    } else {
        None
    };
    let payload_identity_archive_id = if experimental_self_identifying_blocks {
        Some(compute_file_identity_archive_id(&files))
    } else {
        None
    };
    let total_files = files.len();
    let checkpoint_stride = 2usize;
    for (ordinal, file) in files.into_iter().enumerate() {
        let raw = std::fs::read(&file.abs_path)
            .with_context(|| format!("read {}", file.abs_path.display()))?;
        let compressed = compress_deterministic(&raw, level)
            .with_context(|| format!("compress {}", file.abs_path.display()))?;

        let block_scan_offset = out.stream_position()?;

        let payload_hash = *blake3::hash(&compressed).as_bytes();
        let raw_hash = *blake3::hash(&raw).as_bytes();
        let flags = Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH);
        let header = Blk3Header {
            header_len: (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u16,
            flags,
            codec: ZSTD_CODEC,
            level,
            dict_id: 0,
            raw_len: raw.len() as u64,
            comp_len: compressed.len() as u64,
            payload_hash: Some(payload_hash),
            raw_hash: Some(raw_hash),
        };

        write_blk3_header(&mut out, &header)?;
        out.write_all(&compressed)?;

        if experimental_self_describing_extents {
            let record = json!({
                "file_id": block_id,
                "path": file.rel_path,
                "logical_offset": 0,
                "logical_length": raw.len() as u64,
                "full_file_size": raw.len() as u64,
                "extent_ordinal": 0,
                "block_id": block_id,
                "content_identity": {
                    "payload_hash_blake3": to_hex(&payload_hash),
                    "raw_hash_blake3": to_hex(&raw_hash),
                }
            });
            experimental_records.push(record.clone());
            write_experimental_metadata_block(
                &mut out,
                &json!({
                    "schema": "crushr-self-describing-extent.v1",
                    "record": record,
                }),
                level,
            )?;

            if (ordinal + 1) % checkpoint_stride == 0 {
                write_experimental_metadata_block(
                    &mut out,
                    &json!({
                        "schema": "crushr-checkpoint-map-snapshot.v1",
                        "checkpoint_ordinal": ((ordinal + 1) / checkpoint_stride) as u64,
                        "records": experimental_records,
                    }),
                    level,
                )?;
            }
        }

        if experimental_file_identity_extents {
            let path = file.rel_path.clone();
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            file_identity_extent_records.push(json!({
                "schema": "crushr-file-identity-extent.v1",
                "file_id": block_id,
                "logical_offset": 0,
                "logical_length": raw.len() as u64,
                "full_file_size": raw.len() as u64,
                "extent_ordinal": 0,
                "block_id": block_id,
                "block_scan_offset": block_scan_offset,
                "content_identity": {
                    "payload_hash_blake3": to_hex(&payload_hash),
                    "raw_hash_blake3": to_hex(&raw_hash),
                },
                "path_linkage": {
                    "path_digest_blake3": to_hex(&path_digest),
                }
            }));
            file_identity_path_records.push(json!({
                "file_id": block_id,
                "path": path.clone(),
                "path_digest_blake3": to_hex(&path_digest),
            }));

            write_experimental_metadata_block(
                &mut out,
                file_identity_extent_records
                    .last()
                    .context("missing file identity record")?,
                level,
            )?;
            write_experimental_metadata_block(
                &mut out,
                &json!({
                    "schema": "crushr-file-path-map-entry.v1",
                    "file_id": block_id,
                    "path": path,
                    "path_digest_blake3": to_hex(&path_digest),
                }),
                level,
            )?;
            if should_emit_anchor(ordinal, total_files) {
                write_experimental_metadata_block(
                    &mut out,
                    &json!({
                        "schema": "crushr-bootstrap-anchor.v1",
                        "anchor_ordinal": ordinal as u64,
                        "archive_identity": file_identity_archive_id,
                        "records_emitted": file_identity_extent_records.len() as u64,
                    }),
                    level,
                )?;
            }
        }

        if experimental_self_identifying_blocks {
            let archive_identity = payload_identity_archive_id.clone();
            let path = file.rel_path.clone();
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            let payload_record = json!({
                "schema": "crushr-payload-block-identity.v1",
                "archive_identity": archive_identity,
                "file_id": block_id,
                "block_id": block_id,
                "block_index": 0,
                "total_block_count": 1,
                "full_file_size": raw.len() as u64,
                "logical_offset": 0,
                "payload_codec": ZSTD_CODEC,
                "payload_length": compressed.len() as u64,
                "logical_length": raw.len() as u64,
                "block_scan_offset": block_scan_offset,
                "content_identity": {
                    "payload_hash_blake3": to_hex(&payload_hash),
                    "raw_hash_blake3": to_hex(&raw_hash),
                },
            });
            payload_block_identity_records.push(payload_record.clone());
            write_experimental_metadata_block(&mut out, &payload_record, level)?;

            path_checkpoint_entries.push(json!({
                "file_id": block_id,
                "path": path,
                "path_digest_blake3": to_hex(&path_digest),
                "full_file_size": raw.len() as u64,
                "total_block_count": 1,
            }));

            if should_emit_anchor(ordinal, total_files) {
                write_experimental_metadata_block(
                    &mut out,
                    &json!({
                        "schema": "crushr-path-checkpoint.v1",
                        "checkpoint_ordinal": ordinal as u64,
                        "entries": path_checkpoint_entries,
                    }),
                    level,
                )?;
            }
        }

        if experimental_file_manifest_checkpoints {
            let file_digest = to_hex(blake3::hash(&raw).as_bytes());
            let manifest_record = json!({
                "schema": "crushr-file-manifest.v1",
                "file_id": block_id,
                "path": file.rel_path,
                "file_size": raw.len() as u64,
                "expected_block_count": 1,
                "extent_count": 1,
                "file_digest": file_digest,
            });
            file_manifest_records.push(manifest_record.clone());
            write_experimental_metadata_block(&mut out, &manifest_record, level)?;

            if should_emit_anchor(ordinal, total_files) {
                write_experimental_metadata_block(
                    &mut out,
                    &json!({
                        "schema": "crushr-file-manifest-checkpoint.v1",
                        "checkpoint_ordinal": ordinal as u64,
                        "records": file_manifest_records,
                    }),
                    level,
                )?;
            }
        }

        entries.push(Entry {
            path: file.rel_path,
            kind: EntryKind::Regular,
            mode: 0,
            mtime: 0,
            size: raw.len() as u64,
            extents: vec![Extent {
                block_id,
                offset: 0,
                len: raw.len() as u64,
            }],
            link_target: None,
            xattrs: Vec::new(),
        });

        block_id = block_id
            .checked_add(1)
            .context("too many files for minimal packer")?;
    }

    if experimental_self_describing_extents {
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-checkpoint-map-snapshot.v1",
                "checkpoint_ordinal": u64::MAX,
                "records": experimental_records,
            }),
            level,
        )?;
    }

    if experimental_file_identity_extents {
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-bootstrap-anchor.v1",
                "anchor_ordinal": u64::MAX,
                "archive_identity": file_identity_archive_id,
                "records_emitted": file_identity_extent_records.len() as u64,
            }),
            level,
        )?;
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-file-path-map.v1",
                "records": file_identity_path_records,
            }),
            level,
        )?;
    }

    if experimental_self_identifying_blocks {
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-path-checkpoint.v1",
                "checkpoint_ordinal": u64::MAX,
                "entries": path_checkpoint_entries,
            }),
            level,
        )?;
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-payload-block-identity-summary.v1",
                "records_emitted": payload_block_identity_records.len() as u64,
            }),
            level,
        )?;
    }

    if experimental_file_manifest_checkpoints {
        write_experimental_metadata_block(
            &mut out,
            &json!({
                "schema": "crushr-file-manifest-checkpoint.v1",
                "checkpoint_ordinal": u64::MAX,
                "records": file_manifest_records,
            }),
            level,
        )?;
    }

    let blocks_end_offset = out.stream_position()?;
    let idx3 = encode_index(&Index {
        entries: entries.clone(),
    });
    let redundant_file_map = json!({
        "schema": if experimental_self_describing_extents || experimental_file_identity_extents || experimental_self_identifying_blocks || experimental_file_manifest_checkpoints { "crushr-redundant-file-map.experimental.v2" } else { "crushr-redundant-file-map.v1" },
        "experimental_self_describing_extents": experimental_self_describing_extents,
        "experimental_file_identity_extents": experimental_file_identity_extents,
        "experimental_self_identifying_blocks": experimental_self_identifying_blocks,
        "experimental_file_manifest_checkpoints": experimental_file_manifest_checkpoints,
        "files": entries
            .iter()
            .map(|entry| {
                json!({
                    "path": entry.path,
                    "size": entry.size,
                    "extents": entry
                        .extents
                        .iter()
                        .map(|extent| {
                            json!({
                                "block_id": extent.block_id,
                                "file_offset": extent.offset,
                                "len": extent.len,
                            })
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    });
    let ledger = LedgerBlob::from_value(&redundant_file_map)?;
    let tail = assemble_tail_frame(blocks_end_offset, None, &idx3, Some(&ledger))?;
    out.write_all(&tail)?;

    Ok(())
}

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn compute_file_identity_archive_id(files: &[InputFile]) -> String {
    let mut hasher = blake3::Hasher::new();
    for file in files {
        hasher.update(file.rel_path.as_bytes());
        hasher.update(&[0u8]);
    }
    hasher.finalize().to_hex().to_string()
}

fn should_emit_anchor(ordinal: usize, total: usize) -> bool {
    if total <= 3 {
        return true;
    }
    ordinal == 0 || ordinal + 1 == total || ordinal + 1 == total / 2
}

fn write_experimental_metadata_block(
    out: &mut File,
    value: &serde_json::Value,
    level: i32,
) -> Result<()> {
    let raw = serde_json::to_vec(value)?;
    let compressed = compress_deterministic(&raw, level)?;
    let payload_hash = *blake3::hash(&compressed).as_bytes();
    let raw_hash = *blake3::hash(&raw).as_bytes();
    let header = Blk3Header {
        header_len: (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u16,
        flags: Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH),
        codec: ZSTD_CODEC,
        level,
        dict_id: 0,
        raw_len: raw.len() as u64,
        comp_len: compressed.len() as u64,
        payload_hash: Some(payload_hash),
        raw_hash: Some(raw_hash),
    };
    write_blk3_header(&mut *out, &header)?;
    out.write_all(&compressed)?;
    Ok(())
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<InputFile>> {
    let mut files = Vec::new();

    for input in inputs {
        let abs = std::fs::canonicalize(input)
            .with_context(|| format!("canonicalize {}", input.display()))?;
        let meta =
            std::fs::symlink_metadata(&abs).with_context(|| format!("stat {}", input.display()))?;

        if meta.is_file() {
            let name = abs
                .file_name()
                .context("input file has no file name")?
                .to_string_lossy()
                .to_string();
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: abs,
            });
            continue;
        }

        if !meta.is_dir() {
            bail!("unsupported input type: {}", input.display());
        }

        for entry in walkdir::WalkDir::new(&abs).follow_links(false) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let rel = entry
                .path()
                .strip_prefix(&abs)
                .context("strip input prefix")?
                .to_string_lossy()
                .to_string();

            files.push(InputFile {
                rel_path: normalize_logical_path(&rel),
                abs_path: entry.path().to_path_buf(),
            });
        }
    }

    files.sort_by(|a, b| {
        a.rel_path
            .cmp(&b.rel_path)
            .then_with(|| a.abs_path.cmp(&b.abs_path))
    });
    Ok(files)
}

fn normalize_logical_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn reject_duplicate_logical_paths(files: &[InputFile]) -> Result<()> {
    let mut path_sources: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    for file in files {
        path_sources
            .entry(file.rel_path.as_str())
            .or_default()
            .push(file.abs_path.display().to_string());
    }

    for (logical_path, mut sources) in path_sources {
        if sources.len() > 1 {
            sources.sort();
            bail!(
                "duplicate logical archive path '{logical_path}' from inputs: {}",
                sources.join(", ")
            );
        }
    }

    Ok(())
}
