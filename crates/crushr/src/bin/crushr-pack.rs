// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{bail, Context, Result};
use crushr::format::{Entry, EntryKind, Extent, Index};
use crushr::index_codec::encode_index;
use crushr_format::blk3::{write_blk3_header, Blk3Flags, Blk3Header};
use crushr_format::ledger::LedgerBlob;
use crushr_format::tailframe::assemble_tail_frame;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
#[path = "../cli_presentation.rs"]
mod cli_presentation;
use cli_presentation::{group_u64, CliPresenter, StatusWord};

const ZSTD_CODEC: u32 = 1;
const USAGE: &str = "usage: crushr-pack <input>... -o <archive> [--level <n>] [--experimental-self-describing-extents] [--experimental-file-identity-extents] [--experimental-self-identifying-blocks] [--experimental-file-manifest-checkpoints] [--metadata-profile <payload_only|payload_plus_manifest|payload_plus_path|full_current_experimental|extent_identity_only|extent_identity_inline_path|extent_identity_distributed_names|extent_identity_path_dict_single|extent_identity_path_dict_header_tail|extent_identity_path_dict_quasi_uniform|extent_identity_path_dict_factored_header_tail>] [--placement-strategy <fixed_spread|hash_spread|golden_spread>] [--silent]\n\nFlags:\n  -o, --output <archive>                     output archive path\n  --level <n>                                zstd compression level (default: 3)\n  --experimental-self-describing-extents     emit self-describing extent + checkpoint metadata\n  --experimental-file-identity-extents       emit file-identity extent + verified path-map metadata + distributed bootstrap anchors\n  --experimental-self-identifying-blocks     emit payload block identity + repeated verified path checkpoints\n  --experimental-file-manifest-checkpoints   emit distributed file-manifest checkpoints for recovery verification\n  --metadata-profile <name>                  experimental metadata pruning profile: payload_only | payload_plus_manifest | payload_plus_path | full_current_experimental | extent_identity_only | extent_identity_inline_path | extent_identity_distributed_names | extent_identity_path_dict_single | extent_identity_path_dict_header_tail | extent_identity_path_dict_quasi_uniform | extent_identity_path_dict_factored_header_tail\n  --placement-strategy <name>                metadata checkpoint placement strategy (experimental only): fixed_spread | hash_spread | golden_spread\n  --silent                                   emit deterministic one-line summary output\n  -h, --help                                 print this help text";

#[derive(Clone, Copy, Debug)]
enum PlacementStrategy {
    Fixed,
    Hash,
    Golden,
}

#[derive(Clone, Copy, Debug)]
struct PackExperimentalOptions {
    self_describing_extents: bool,
    file_identity_extents: bool,
    self_identifying_blocks: bool,
    file_manifest_checkpoints: bool,
    metadata_profile: Option<MetadataProfile>,
    placement_strategy: Option<PlacementStrategy>,
}

#[derive(Clone, Copy, Debug)]
enum MetadataProfile {
    PayloadOnly,
    PayloadPlusManifest,
    PayloadPlusPath,
    FullCurrentExperimental,
    ExtentIdentityOnly,
    ExtentIdentityInlinePath,
    ExtentIdentityDistributedNames,
    ExtentIdentityPathDictSingle,
    ExtentIdentityPathDictHeaderTail,
    ExtentIdentityPathDictQuasiUniform,
    ExtentIdentityPathDictFactoredHeaderTail,
}

impl PlacementStrategy {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "fixed_spread" => Ok(Self::Fixed),
            "hash_spread" => Ok(Self::Hash),
            "golden_spread" => Ok(Self::Golden),
            _ => bail!("unsupported placement strategy: {value}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed_spread",
            Self::Hash => "hash_spread",
            Self::Golden => "golden_spread",
        }
    }
}

impl MetadataProfile {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "payload_only" => Ok(Self::PayloadOnly),
            "payload_plus_manifest" => Ok(Self::PayloadPlusManifest),
            "payload_plus_path" => Ok(Self::PayloadPlusPath),
            "full_current_experimental" => Ok(Self::FullCurrentExperimental),
            "extent_identity_only" => Ok(Self::ExtentIdentityOnly),
            "extent_identity_inline_path" => Ok(Self::ExtentIdentityInlinePath),
            "extent_identity_distributed_names" => Ok(Self::ExtentIdentityDistributedNames),
            "extent_identity_path_dict_single" => Ok(Self::ExtentIdentityPathDictSingle),
            "extent_identity_path_dict_header_tail" => Ok(Self::ExtentIdentityPathDictHeaderTail),
            "extent_identity_path_dict_quasi_uniform" => {
                Ok(Self::ExtentIdentityPathDictQuasiUniform)
            }
            "extent_identity_path_dict_factored_header_tail" => {
                Ok(Self::ExtentIdentityPathDictFactoredHeaderTail)
            }
            _ => bail!("unsupported metadata profile: {value}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::PayloadPlusPath => "payload_plus_path",
            Self::FullCurrentExperimental => "full_current_experimental",
            Self::ExtentIdentityOnly => "extent_identity_only",
            Self::ExtentIdentityInlinePath => "extent_identity_inline_path",
            Self::ExtentIdentityDistributedNames => "extent_identity_distributed_names",
            Self::ExtentIdentityPathDictSingle => "extent_identity_path_dict_single",
            Self::ExtentIdentityPathDictHeaderTail => "extent_identity_path_dict_header_tail",
            Self::ExtentIdentityPathDictQuasiUniform => "extent_identity_path_dict_quasi_uniform",
            Self::ExtentIdentityPathDictFactoredHeaderTail => {
                "extent_identity_path_dict_factored_header_tail"
            }
        }
    }

    fn emit_path_checkpoints(self) -> bool {
        matches!(
            self,
            Self::PayloadPlusPath
                | Self::FullCurrentExperimental
                | Self::ExtentIdentityDistributedNames
        )
    }

    fn emit_manifest_checkpoints(self) -> bool {
        matches!(
            self,
            Self::PayloadPlusManifest | Self::FullCurrentExperimental
        )
    }

    fn uses_path_dictionary(self) -> bool {
        matches!(
            self,
            Self::ExtentIdentityPathDictSingle
                | Self::ExtentIdentityPathDictHeaderTail
                | Self::ExtentIdentityPathDictQuasiUniform
                | Self::ExtentIdentityPathDictFactoredHeaderTail
        )
    }
}

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

#[derive(Debug)]
struct CanonicalFileModel {
    file_id: u32,
    rel_path: String,
    raw: Vec<u8>,
    compressed: Vec<u8>,
    payload_hash: [u8; 32],
    raw_hash: [u8; 32],
}

#[derive(Debug)]
struct DictionaryPlan {
    path_id_by_path: BTreeMap<String, u32>,
    primary_copy: Option<PathDictionaryCopyRecordV2>,
    tail_copy_required: bool,
    quasi_uniform_ordinals: BTreeSet<usize>,
}

#[derive(Debug)]
struct MetadataPlan {
    emit_payload_identity: bool,
    emit_path_checkpoints: bool,
    emit_manifest_checkpoints: bool,
    use_path_dictionary: bool,
    inline_payload_path: bool,
    file_identity_archive_id: Option<String>,
    payload_identity_archive_id: Option<String>,
    path_checkpoint_ordinals: BTreeSet<usize>,
    manifest_checkpoint_ordinals: BTreeSet<usize>,
    dictionary: DictionaryPlan,
}

#[derive(Debug)]
struct PackLayoutPlan {
    files: Vec<CanonicalFileModel>,
    metadata: MetadataPlan,
}

#[derive(Debug, Serialize)]
struct RedundantFileMap {
    schema: &'static str,
    experimental_self_describing_extents: bool,
    experimental_file_identity_extents: bool,
    experimental_self_identifying_blocks: bool,
    experimental_path_checkpoints: bool,
    experimental_file_manifest_checkpoints: bool,
    experimental_metadata_profile: Option<&'static str>,
    metadata_placement_strategy: Option<&'static str>,
    files: Vec<RedundantFileMapFile>,
}

#[derive(Debug, Serialize)]
struct RedundantFileMapFile {
    path: String,
    size: u64,
    extents: Vec<RedundantFileMapExtent>,
}

#[derive(Debug, Serialize)]
struct RedundantFileMapExtent {
    block_id: u32,
    file_offset: u64,
    len: u64,
}

#[derive(Debug, Serialize, Clone)]
struct ContentIdentity {
    payload_hash_blake3: String,
    raw_hash_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct SelfDescribingExtentRecord {
    file_id: u32,
    path: String,
    logical_offset: u64,
    logical_length: u64,
    full_file_size: u64,
    extent_ordinal: u64,
    block_id: u32,
    content_identity: ContentIdentity,
}

#[derive(Debug, Serialize, Clone)]
struct SelfDescribingExtentEnvelope {
    schema: &'static str,
    record: SelfDescribingExtentRecord,
}

#[derive(Debug, Serialize, Clone)]
struct CheckpointMapSnapshot {
    schema: &'static str,
    checkpoint_ordinal: u64,
    records: Vec<SelfDescribingExtentRecord>,
}

#[derive(Debug, Serialize, Clone)]
struct PathLinkage {
    path_digest_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct FileIdentityExtentRecord {
    schema: &'static str,
    file_id: u32,
    logical_offset: u64,
    logical_length: u64,
    full_file_size: u64,
    extent_ordinal: u64,
    block_id: u32,
    block_scan_offset: u64,
    content_identity: ContentIdentity,
    path_linkage: PathLinkage,
}

#[derive(Debug, Serialize, Clone)]
struct FileIdentityPathRecord {
    file_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct FilePathMapEntryRecord {
    schema: &'static str,
    file_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct BootstrapAnchorRecord {
    schema: &'static str,
    anchor_ordinal: u64,
    archive_identity: Option<String>,
    records_emitted: u64,
}

#[derive(Debug, Serialize, Clone)]
struct PayloadBlockIdentityRecord {
    schema: &'static str,
    archive_identity: Option<String>,
    file_id: u32,
    block_id: u32,
    block_index: u64,
    extent_index: u64,
    total_block_count: u64,
    total_extent_count: u64,
    full_file_size: u64,
    logical_offset: u64,
    payload_codec: u32,
    payload_length: u64,
    logical_length: u64,
    extent_length: u64,
    block_scan_offset: u64,
    content_identity: ContentIdentity,
    name: Option<String>,
    path: Option<String>,
    path_digest_blake3: Option<String>,
    path_id: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
struct PathCheckpointEntry {
    file_id: u32,
    path: String,
    path_digest_blake3: String,
    full_file_size: u64,
    total_block_count: u64,
}

#[derive(Debug, Serialize, Clone)]
struct PathCheckpointSnapshot {
    schema: &'static str,
    checkpoint_ordinal: u64,
    placement_strategy: Option<&'static str>,
    entries: Vec<PathCheckpointEntry>,
}

#[derive(Debug, Serialize, Clone)]
struct FileManifestRecord {
    schema: &'static str,
    file_id: u32,
    path: String,
    file_size: u64,
    expected_block_count: u64,
    extent_count: u64,
    file_digest: String,
}

#[derive(Debug, Serialize, Clone)]
struct FileManifestCheckpointSnapshot {
    schema: &'static str,
    checkpoint_ordinal: u64,
    placement_strategy: Option<&'static str>,
    records: Vec<FileManifestRecord>,
}

#[derive(Debug, Serialize, Clone)]
struct FilePathMapRecord {
    schema: &'static str,
    records: Vec<FileIdentityPathRecord>,
}

#[derive(Debug, Serialize, Clone)]
struct PayloadBlockIdentitySummary {
    schema: &'static str,
    records_emitted: u64,
}

#[derive(Debug, Serialize, Clone)]
struct PathDictionaryEntry {
    path_id: u32,
    path: String,
    path_digest_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
struct FactoredDirectory {
    dir_id: u32,
    prefix: String,
}

#[derive(Debug, Serialize, Clone)]
struct FactoredBasename {
    name_id: u32,
    basename: String,
}

#[derive(Debug, Serialize, Clone)]
struct FactoredFileBinding {
    path_id: u32,
    dir_id: u32,
    name_id: u32,
    path_digest_blake3: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "representation")]
enum PathDictionaryBody {
    #[serde(rename = "full_path_v1")]
    FullPath {
        entry_count: u64,
        entries: Vec<PathDictionaryEntry>,
    },
    #[serde(rename = "factored_namespace_v1")]
    FactoredNamespace {
        entry_count: u64,
        directory_count: u64,
        basename_count: u64,
        directories: Vec<FactoredDirectory>,
        basenames: Vec<FactoredBasename>,
        file_bindings: Vec<FactoredFileBinding>,
    },
}

#[derive(Debug, Serialize, Clone)]
struct PathDictionaryCopyRecordV2 {
    schema: &'static str,
    copy_role: &'static str,
    archive_instance_id: String,
    dict_uuid: String,
    generation: u64,
    dictionary_length: u64,
    dictionary_content_hash: String,
    body: PathDictionaryBody,
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
    let mut metadata_profile = None;
    let mut placement_strategy = None;
    let mut silent = false;

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
        } else if arg == "--metadata-profile" {
            metadata_profile = Some(MetadataProfile::parse(&args.next().context(USAGE)?)?);
        } else if arg == "--placement-strategy" {
            placement_strategy = Some(PlacementStrategy::parse(&args.next().context(USAGE)?)?);
        } else if arg == "--silent" {
            silent = true;
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
    if metadata_profile.is_some()
        && (experimental_self_identifying_blocks || experimental_file_manifest_checkpoints)
    {
        bail!(
            "--metadata-profile cannot be combined with --experimental-self-identifying-blocks or --experimental-file-manifest-checkpoints"
        );
    }

    let emit_path_checkpoints = metadata_profile
        .map(MetadataProfile::emit_path_checkpoints)
        .unwrap_or(experimental_self_identifying_blocks);
    let emit_manifest_checkpoints = metadata_profile
        .map(MetadataProfile::emit_manifest_checkpoints)
        .unwrap_or(experimental_file_manifest_checkpoints);

    if placement_strategy.is_some() && !emit_path_checkpoints && !emit_manifest_checkpoints {
        bail!(
            "--placement-strategy requires --experimental-self-identifying-blocks and/or --experimental-file-manifest-checkpoints"
        );
    }

    let presenter = CliPresenter::new("crushr-pack", "pack", silent);
    presenter.header();
    presenter.section("Target");
    presenter.kv("archive", output.display());
    presenter.kv_number("inputs", inputs.len() as u64);
    presenter.kv("compression level", level);

    pack_minimal_v1(
        &inputs,
        &output,
        level,
        PackExperimentalOptions {
            self_describing_extents: experimental_self_describing_extents,
            file_identity_extents: experimental_file_identity_extents,
            self_identifying_blocks: experimental_self_identifying_blocks,
            file_manifest_checkpoints: experimental_file_manifest_checkpoints,
            metadata_profile,
            placement_strategy,
        },
        &presenter,
    )
}

fn pack_minimal_v1(
    inputs: &[PathBuf],
    output: &Path,
    level: i32,
    options: PackExperimentalOptions,
    presenter: &CliPresenter,
) -> Result<()> {
    presenter.section("Progress");
    presenter.stage("input discovery", StatusWord::Scanning);
    let files = collect_files(inputs)?;
    if files.is_empty() {
        bail!("no input files to pack");
    }
    reject_duplicate_logical_paths(&files)?;
    presenter.stage("planning", StatusWord::Running);
    let layout = build_pack_layout_plan(files, level, options)?;
    let file_count = layout.files.len();
    presenter.stage("serialization", StatusWord::Writing);
    emit_archive_from_layout(layout, output, level, options)?;
    presenter.stage("finalization", StatusWord::Finalizing);
    presenter.section("Result");
    presenter.kv("files packed", group_u64(file_count as u64));
    presenter.outcome(StatusWord::Complete, "archive emitted");
    presenter.silent_summary(
        StatusWord::Complete,
        &[
            ("archive", output.display().to_string()),
            ("files", file_count.to_string()),
        ],
    );
    Ok(())
}

fn build_pack_layout_plan(
    files: Vec<InputFile>,
    level: i32,
    options: PackExperimentalOptions,
) -> Result<PackLayoutPlan> {
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
    let mut canonical_files = Vec::with_capacity(files.len());
    for (idx, file) in files.into_iter().enumerate() {
        let raw = std::fs::read(&file.abs_path)
            .with_context(|| format!("read {}", file.abs_path.display()))?;
        let compressed = compress_deterministic(&raw, level)
            .with_context(|| format!("compress {}", file.abs_path.display()))?;
        canonical_files.push(CanonicalFileModel {
            file_id: idx as u32,
            rel_path: file.rel_path,
            payload_hash: *blake3::hash(&compressed).as_bytes(),
            raw_hash: *blake3::hash(&raw).as_bytes(),
            raw,
            compressed,
        });
    }

    Ok(PackLayoutPlan {
        files: canonical_files,
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

fn emit_archive_from_layout(
    layout: PackLayoutPlan,
    output: &Path,
    level: i32,
    options: PackExperimentalOptions,
) -> Result<()> {
    let total_files = layout.files.len();

    let mut out = File::create(output).with_context(|| format!("create {}", output.display()))?;
    let mut entries = Vec::with_capacity(total_files);

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
        write_experimental_metadata_block(&mut out, path_dictionary, level)?;
    }
    for (ordinal, file) in layout.files.into_iter().enumerate() {
        let block_scan_offset = out.stream_position()?;
        let payload_hash = file.payload_hash;
        let raw_hash = file.raw_hash;
        let flags = Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH);
        let header = Blk3Header {
            header_len: (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u16,
            flags,
            codec: ZSTD_CODEC,
            level,
            dict_id: 0,
            raw_len: file.raw.len() as u64,
            comp_len: file.compressed.len() as u64,
            payload_hash: Some(payload_hash),
            raw_hash: Some(raw_hash),
        };

        write_blk3_header(&mut out, &header)?;
        out.write_all(&file.compressed)?;

        if options.self_describing_extents {
            let record = build_self_describing_extent_record(&file);
            experimental_records.push(record.clone());
            write_experimental_metadata_block(
                &mut out,
                &wrap_self_describing_extent(record),
                level,
            )?;

            if (ordinal + 1) % checkpoint_stride == 0 {
                write_experimental_metadata_block(
                    &mut out,
                    &build_checkpoint_map_snapshot(
                        ((ordinal + 1) / checkpoint_stride) as u64,
                        &experimental_records,
                    ),
                    level,
                )?;
            }
        }

        if options.file_identity_extents {
            let path = file.rel_path.clone();
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            file_identity_extent_records.push(build_file_identity_extent_record(
                &file,
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
            )?;
            write_experimental_metadata_block(
                &mut out,
                &build_file_path_map_entry(file.file_id, &path, &path_digest),
                level,
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
            let path_digest = *blake3::hash(path.as_bytes()).as_bytes();
            let path_id = path_id_by_path.get(&path).copied();
            let payload_record = build_payload_block_identity_record(
                &file,
                archive_identity,
                block_scan_offset,
                inline_payload_path.then_some(name),
                inline_payload_path.then_some(path.clone()),
                inline_payload_path.then_some(to_hex(&path_digest)),
                use_path_dictionary.then_some(path_id).flatten(),
            );
            payload_block_identity_records.push(payload_record.clone());
            write_experimental_metadata_block(&mut out, &payload_record, level)?;

            if use_path_dictionary && quasi_uniform_ordinals.contains(&ordinal) {
                let mut copy = layout
                    .metadata
                    .dictionary
                    .primary_copy
                    .clone()
                    .context("missing primary dictionary copy for interior mirror")?;
                copy.copy_role = "interior_mirror";
                write_experimental_metadata_block(&mut out, &copy, level)?;
            }

            if emit_path_checkpoints {
                path_checkpoint_entries.push(build_path_checkpoint_entry(
                    file.file_id,
                    &path,
                    &path_digest,
                    file.raw.len() as u64,
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
                    )?;
                }
            }
        }

        if emit_manifest_checkpoints {
            let manifest_record = build_file_manifest_record(&file);
            file_manifest_records.push(manifest_record.clone());
            write_experimental_metadata_block(&mut out, &manifest_record, level)?;

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
                )?;
            }
        }

        entries.push(Entry {
            path: file.rel_path,
            kind: EntryKind::Regular,
            mode: 0,
            mtime: 0,
            size: file.raw.len() as u64,
            extents: vec![Extent {
                block_id: file.file_id,
                offset: 0,
                len: file.raw.len() as u64,
            }],
            link_target: None,
            xattrs: Vec::new(),
        });
    }

    if layout.metadata.dictionary.tail_copy_required {
        let mut copy = layout
            .metadata
            .dictionary
            .primary_copy
            .clone()
            .context("missing primary dictionary copy for tail mirror")?;
        copy.copy_role = "tail_mirror";
        write_experimental_metadata_block(&mut out, &copy, level)?;
    }

    if options.self_describing_extents {
        write_experimental_metadata_block(
            &mut out,
            &build_checkpoint_map_snapshot(u64::MAX, &experimental_records),
            level,
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
        )?;
        write_experimental_metadata_block(
            &mut out,
            &FilePathMapRecord {
                schema: "crushr-file-path-map.v1",
                records: file_identity_path_records,
            },
            level,
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
        )?;
    }

    let blocks_end_offset = out.stream_position()?;
    write_tail_with_redundant_map(
        &mut out,
        blocks_end_offset,
        &entries,
        options,
        emit_payload_identity,
        emit_path_checkpoints,
        emit_manifest_checkpoints,
    )?;

    Ok(())
}

fn build_self_describing_extent_record(file: &CanonicalFileModel) -> SelfDescribingExtentRecord {
    SelfDescribingExtentRecord {
        file_id: file.file_id,
        path: file.rel_path.clone(),
        logical_offset: 0,
        logical_length: file.raw.len() as u64,
        full_file_size: file.raw.len() as u64,
        extent_ordinal: 0,
        block_id: file.file_id,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(&file.payload_hash),
            raw_hash_blake3: to_hex(&file.raw_hash),
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
    file: &CanonicalFileModel,
    block_scan_offset: u64,
    path_digest: &[u8; 32],
) -> FileIdentityExtentRecord {
    FileIdentityExtentRecord {
        schema: "crushr-file-identity-extent.v1",
        file_id: file.file_id,
        logical_offset: 0,
        logical_length: file.raw.len() as u64,
        full_file_size: file.raw.len() as u64,
        extent_ordinal: 0,
        block_id: file.file_id,
        block_scan_offset,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(&file.payload_hash),
            raw_hash_blake3: to_hex(&file.raw_hash),
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
    file: &CanonicalFileModel,
    archive_identity: Option<String>,
    block_scan_offset: u64,
    inline_name: Option<String>,
    inline_path: Option<String>,
    inline_path_digest: Option<String>,
    path_id: Option<u32>,
) -> PayloadBlockIdentityRecord {
    PayloadBlockIdentityRecord {
        schema: "crushr-payload-block-identity.v1",
        archive_identity,
        file_id: file.file_id,
        block_id: file.file_id,
        block_index: 0,
        extent_index: 0,
        total_block_count: 1,
        total_extent_count: 1,
        full_file_size: file.raw.len() as u64,
        logical_offset: 0,
        payload_codec: ZSTD_CODEC,
        payload_length: file.compressed.len() as u64,
        logical_length: file.raw.len() as u64,
        extent_length: file.raw.len() as u64,
        block_scan_offset,
        content_identity: ContentIdentity {
            payload_hash_blake3: to_hex(&file.payload_hash),
            raw_hash_blake3: to_hex(&file.raw_hash),
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

fn build_file_manifest_record(file: &CanonicalFileModel) -> FileManifestRecord {
    FileManifestRecord {
        schema: "crushr-file-manifest.v1",
        file_id: file.file_id,
        path: file.rel_path.clone(),
        file_size: file.raw.len() as u64,
        expected_block_count: 1,
        extent_count: 1,
        file_digest: to_hex(blake3::hash(&file.raw).as_bytes()),
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

fn write_tail_with_redundant_map(
    out: &mut File,
    blocks_end_offset: u64,
    entries: &[Entry],
    options: PackExperimentalOptions,
    emit_payload_identity: bool,
    emit_path_checkpoints: bool,
    emit_manifest_checkpoints: bool,
) -> Result<()> {
    let idx3 = encode_index(&Index {
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

fn build_dictionary_plan(
    files: &[InputFile],
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

fn write_experimental_metadata_block<T: Serialize>(
    out: &mut File,
    value: &T,
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
