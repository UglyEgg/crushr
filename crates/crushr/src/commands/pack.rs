// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::cli_presentation::{CliPresenter, StatusWord, group_u64};
use crate::format::{Entry, EntryKind, Extent, Index, PreservationProfile, Xattr};
use crate::index_codec::encode_index;
use anyhow::{Context, Result, bail};
use crushr_format::blk3::{Blk3Flags, Blk3Header, write_blk3_header};
use crushr_format::ledger::LedgerBlob;
use crushr_format::tailframe::assemble_tail_frame;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const ZSTD_CODEC: u32 = 1;
const PRODUCTION_USAGE: &str = "usage: crushr-pack <input>... -o <archive> [--level <n>] [--preservation <full|basic|payload-only>] [--profile-pack] [--silent]\n\nFlags:\n  -o, --output <archive>                     output archive path\n  --level <n>                                zstd compression level (default: 3)\n  --preservation <name>                      preservation profile: full | basic | payload-only (default: full)\n  --profile-pack                             emit deterministic pack phase timing breakdown\n  --silent                                   emit deterministic one-line summary output\n  -h, --help                                 print this help text";

const LAB_EXPERIMENTAL_USAGE: &str = "usage: crushr lab pack-experimental <input>... -o <archive> [--level <n>] [--experimental-self-describing-extents] [--experimental-file-identity-extents] [--experimental-self-identifying-blocks] [--experimental-file-manifest-checkpoints] [--metadata-profile <payload_only|payload_plus_manifest|payload_plus_path|full_current_experimental|extent_identity_only|extent_identity_inline_path|extent_identity_distributed_names|extent_identity_path_dict_single|extent_identity_path_dict_header_tail|extent_identity_path_dict_quasi_uniform|extent_identity_path_dict_factored_header_tail>] [--placement-strategy <fixed_spread|hash_spread|golden_spread>] [--silent]\n\nFlags:\n  -o, --output <archive>                     output archive path\n  --level <n>                                zstd compression level (default: 3)\n  --experimental-self-describing-extents     emit self-describing extent + checkpoint metadata\n  --experimental-file-identity-extents       emit file-identity extent + verified path-map metadata + distributed bootstrap anchors\n  --experimental-self-identifying-blocks     emit payload block identity + repeated verified path checkpoints\n  --experimental-file-manifest-checkpoints   emit distributed file-manifest checkpoints for recovery verification\n  --metadata-profile <name>                  experimental metadata pruning profile: payload_only | payload_plus_manifest | payload_plus_path | full_current_experimental | extent_identity_only | extent_identity_inline_path | extent_identity_distributed_names | extent_identity_path_dict_single | extent_identity_path_dict_header_tail | extent_identity_path_dict_quasi_uniform | extent_identity_path_dict_factored_header_tail\n  --placement-strategy <name>                metadata checkpoint placement strategy (experimental only): fixed_spread | hash_spread | golden_spread\n  --silent                                   emit deterministic one-line summary output\n  -h, --help                                 print this help text";

#[derive(Clone, Copy, Debug)]
enum PlacementStrategy {
    Fixed,
    Hash,
    Golden,
}

#[derive(Clone, Copy, Debug)]
struct PackExperimentalOptions {
    preservation_profile: PreservationProfile,
    profile_pack: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackCliSurface {
    Production,
    LabExperimental,
}

impl PackCliSurface {
    fn usage(self) -> &'static str {
        match self {
            Self::Production => PRODUCTION_USAGE,
            Self::LabExperimental => LAB_EXPERIMENTAL_USAGE,
        }
    }

    fn allows_experimental_flags(self) -> bool {
        matches!(self, Self::LabExperimental)
    }
}

fn print_help(surface: PackCliSurface) {
    let presenter = CliPresenter::new("crushr-pack", "help", false);
    presenter.header();
    presenter.section("Usage");
    match surface {
        PackCliSurface::Production => presenter.kv(
            "command",
            "usage: crushr-pack <input>... -o <archive> [--level <n>] [--preservation <full|basic|payload-only>] [--profile-pack] [--silent]",
        ),
        PackCliSurface::LabExperimental => presenter.kv(
            "command",
            "usage: crushr lab pack-experimental <input>... -o <archive> [--level <n>] [experimental flags] [--silent]",
        ),
    }
    presenter.section("Flags");
    presenter.kv("-o, --output <archive>", "output archive path");
    presenter.kv("--level <n>", "zstd compression level (default: 3)");
    if matches!(surface, PackCliSurface::Production) {
        presenter.kv(
            "--preservation <name>",
            "preservation profile: full | basic | payload-only (default: full)",
        );
        presenter.kv(
            "--profile-pack",
            "emit deterministic pack phase timing breakdown",
        );
    }
    if matches!(surface, PackCliSurface::LabExperimental) {
        presenter.kv(
            "--experimental-self-describing-extents",
            "emit self-describing extent + checkpoint metadata",
        );
        presenter.kv(
            "--experimental-file-identity-extents",
            "emit file-identity extent + verified path-map metadata + distributed bootstrap anchors",
        );
        presenter.kv(
            "--experimental-self-identifying-blocks",
            "emit payload block identity + repeated verified path checkpoints",
        );
        presenter.kv(
            "--experimental-file-manifest-checkpoints",
            "emit distributed file-manifest checkpoints for recovery verification",
        );
        presenter.kv(
            "--metadata-profile <name>",
            "experimental metadata pruning profile",
        );
        presenter.kv(
            "--placement-strategy <name>",
            "metadata checkpoint placement strategy (experimental only)",
        );
    }
    presenter.kv("--silent", "emit deterministic one-line summary output");
    presenter.kv("-h, --help", "print this help text");
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

const BLK3_HEADER_WITH_HASHES_LEN: u64 = (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u64;

struct DeterministicCompressor {
    compressor: zstd::bulk::Compressor<'static>,
    output: Vec<u8>,
}

impl DeterministicCompressor {
    fn new(level: i32) -> Result<Self> {
        let mut compressor =
            zstd::bulk::Compressor::new(level).context("create zstd compressor")?;
        compressor
            .include_checksum(false)
            .context("set zstd checksum flag")?;
        compressor
            .include_contentsize(true)
            .context("set zstd content-size flag")?;
        compressor
            .include_dictid(false)
            .context("set zstd dict-id flag")?;
        Ok(Self {
            compressor,
            output: Vec::new(),
        })
    }

    fn compress(&mut self, raw: &[u8]) -> Result<&[u8]> {
        self.output.clear();
        self.output
            .reserve(zstd::zstd_safe::compress_bound(raw.len()));
        self.compressor
            .compress_to_buffer(raw, &mut self.output)
            .context("zstd compress")?;
        Ok(&self.output)
    }
}

#[derive(Debug)]
struct InputFile {
    rel_path: String,
    abs_path: PathBuf,
    kind: EntryKind,
    raw_len: u64,
    mode: u32,
    mtime: i64,
    uid: u32,
    gid: u32,
    uname: Option<String>,
    gname: Option<String>,
    hardlink_key: Option<(u64, u64)>,
    xattrs: Vec<Xattr>,
    acl_access: Option<Vec<u8>>,
    acl_default: Option<Vec<u8>>,
    selinux_label: Option<Vec<u8>>,
    linux_capability: Option<Vec<u8>>,
    sparse_chunks: Vec<SparseChunk>,
    device_major: Option<u32>,
    device_minor: Option<u32>,
}

#[derive(Debug)]
struct PlannedFileModel {
    file_id: u32,
    block_id: u32,
    write_payload: bool,
    hardlink_group_id: Option<u64>,
    rel_path: String,
    abs_path: PathBuf,
    raw_len: u64,
    mode: u32,
    mtime: i64,
    uid: u32,
    gid: u32,
    uname: Option<String>,
    gname: Option<String>,
    xattrs: Vec<Xattr>,
    acl_access: Option<Vec<u8>>,
    acl_default: Option<Vec<u8>>,
    selinux_label: Option<Vec<u8>>,
    linux_capability: Option<Vec<u8>>,
    sparse_chunks: Vec<SparseChunk>,
}

#[derive(Debug, Clone)]
struct SparseChunk {
    logical_offset: u64,
    len: u64,
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
    profile_plan: PackProfilePlan,
    files: Vec<PlannedFileModel>,
    metadata: MetadataPlan,
}

#[derive(Debug)]
struct PackProfilePlan {
    included: Vec<InputFile>,
    omitted: Vec<ProfileOmission>,
}

#[derive(Debug, Clone, Copy)]
struct MetadataCaptureRequirements {
    capture_ownership_names: bool,
    capture_xattrs: bool,
    capture_sparse_layout: bool,
}

#[derive(Debug)]
struct ProfileOmission {
    rel_path: String,
    kind: EntryKind,
    reason: ProfileOmissionReason,
}

#[derive(Debug, Clone, Copy)]
enum ProfileOmissionReason {
    BasicOmitsSpecialEntries,
    PayloadOnlyOmitsSymlinks,
    PayloadOnlyOmitsSpecialEntries,
}

#[derive(Debug, Clone, Copy)]
enum PackProgressPhase {
    Compression,
    Serialization,
}

#[derive(Clone, Copy, Debug, Default)]
struct PackPhaseTimings {
    discovery: Duration,
    metadata: Duration,
    hashing: Duration,
    compression: Duration,
    emission: Duration,
    finalization: Duration,
}

impl PackPhaseTimings {
    fn add(&mut self, phase: PackProfilePhase, elapsed: Duration) {
        match phase {
            PackProfilePhase::Hashing => self.hashing += elapsed,
            PackProfilePhase::Compression => self.compression += elapsed,
            PackProfilePhase::Emission => self.emission += elapsed,
            PackProfilePhase::Finalization => self.finalization += elapsed,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PackProfilePhase {
    Hashing,
    Compression,
    Emission,
    Finalization,
}

#[derive(Debug)]
struct PackRunMetrics {
    files_packed: u64,
    total_size_bytes: u64,
    compressed_size_bytes: u64,
    elapsed: Duration,
}

fn format_phase_duration_ms(duration: Duration) -> String {
    format!("{:>6} ms", duration.as_millis())
}

fn print_pack_phase_breakdown(timings: PackPhaseTimings) {
    println!();
    println!("Pack phases");
    println!(
        "  discovery      {}",
        format_phase_duration_ms(timings.discovery)
    );
    println!(
        "  metadata       {}",
        format_phase_duration_ms(timings.metadata)
    );
    println!(
        "  hashing        {}",
        format_phase_duration_ms(timings.hashing)
    );
    println!(
        "  compression    {}",
        format_phase_duration_ms(timings.compression)
    );
    println!(
        "  emission       {}",
        format_phase_duration_ms(timings.emission)
    );
    println!(
        "  finalization   {}",
        format_phase_duration_ms(timings.finalization)
    );
}

struct PayloadIdentityInput<'a> {
    file_id: u32,
    raw_len: u64,
    compressed_len: u64,
    payload_hash: &'a [u8; 32],
    raw_hash: &'a [u8; 32],
    block_scan_offset: u64,
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

mod discovery;
mod emission;
mod planning;

pub fn dispatch(args: Vec<String>) -> i32 {
    if let Err(err) = run(args, PackCliSurface::Production) {
        eprintln!("{err:#}");
        let message = format!("{err:#}");
        if message.contains("usage:")
            || message.contains("unsupported flag")
            || message.contains("unexpected argument")
        {
            return 1;
        } else {
            return 2;
        }
    }
    0
}

pub fn dispatch_from_env() -> i32 {
    dispatch(std::env::args().skip(1).collect())
}

pub fn dispatch_lab_experimental(args: Vec<String>) -> i32 {
    if let Err(err) = run(args, PackCliSurface::LabExperimental) {
        eprintln!("{err:#}");
        let message = format!("{err:#}");
        if message.contains("usage:")
            || message.contains("unsupported flag")
            || message.contains("unexpected argument")
        {
            return 1;
        } else {
            return 2;
        }
    }
    0
}

fn run(raw_args: Vec<String>, surface: PackCliSurface) -> Result<()> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut level: i32 = 3;
    let mut preservation_profile = PreservationProfile::Full;
    let mut profile_pack = false;
    let mut experimental_self_describing_extents = false;
    let mut experimental_file_identity_extents = false;
    let mut experimental_self_identifying_blocks = false;
    let mut experimental_file_manifest_checkpoints = false;
    let mut metadata_profile = None;
    let mut placement_strategy = None;
    let mut silent = false;

    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if inputs.is_empty() && output.is_none() && (arg == "--help" || arg == "-h") {
            print_help(surface);
            return Ok(());
        }
        if arg == "-o" || arg == "--output" {
            let value = args.next().context(surface.usage())?;
            output = Some(PathBuf::from(value));
        } else if arg == "--level" {
            let value = args.next().context(surface.usage())?;
            level = value
                .parse::<i32>()
                .with_context(|| format!("invalid --level value: {value}"))?;
        } else if arg == "--preservation" {
            if !matches!(surface, PackCliSurface::Production) {
                bail!("unsupported flag: {arg}");
            }
            let value = args.next().context(surface.usage())?;
            preservation_profile = PreservationProfile::parse_name(&value)
                .with_context(|| format!("unsupported preservation profile: {value}"))?;
        } else if arg == "--profile-pack" {
            if !matches!(surface, PackCliSurface::Production) {
                bail!("unsupported flag: {arg}");
            }
            profile_pack = true;
        } else if arg == "--experimental-self-describing-extents" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            experimental_self_describing_extents = true;
        } else if arg == "--experimental-file-identity-extents" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            experimental_file_identity_extents = true;
        } else if arg == "--experimental-self-identifying-blocks" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            experimental_self_identifying_blocks = true;
        } else if arg == "--experimental-file-manifest-checkpoints" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            experimental_file_manifest_checkpoints = true;
        } else if arg == "--metadata-profile" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            metadata_profile = Some(MetadataProfile::parse(
                &args.next().context(surface.usage())?,
            )?);
        } else if arg == "--placement-strategy" {
            if !surface.allows_experimental_flags() {
                bail!("unsupported flag: {arg}");
            }
            placement_strategy = Some(PlacementStrategy::parse(
                &args.next().context(surface.usage())?,
            )?);
        } else if arg == "--silent" {
            silent = true;
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else {
            inputs.push(PathBuf::from(arg));
        }
    }

    let output = normalize_archive_output_path(output.context(surface.usage())?);
    if inputs.is_empty() {
        bail!(surface.usage());
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
    presenter.kv("preservation profile", preservation_profile.as_str());

    pack_minimal_v1(
        &inputs,
        &output,
        level,
        PackExperimentalOptions {
            preservation_profile,
            profile_pack,
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
    let mut phase_timings = options.profile_pack.then_some(PackPhaseTimings::default());
    presenter.section("Progress");
    let input_discovery = presenter.begin_active_phase("input discovery", None);
    let discovery_start = Instant::now();
    let metadata_capture_requirements =
        discovery::metadata_capture_requirements(options.preservation_profile);
    let candidates = match discovery::collect_files(inputs, metadata_capture_requirements) {
        Ok(files) => {
            if let Some(timings) = phase_timings.as_mut() {
                timings.discovery = discovery_start.elapsed();
            }
            input_discovery.settle(
                StatusWord::Complete,
                Some(&format!("files={}", group_u64(files.len() as u64))),
            );
            files
        }
        Err(err) => {
            if let Some(timings) = phase_timings.as_mut() {
                timings.discovery = discovery_start.elapsed();
            }
            input_discovery.settle(StatusWord::Failed, None);
            return Err(err);
        }
    };
    if candidates.is_empty() {
        bail!("no input files to pack");
    }
    let metadata_start = Instant::now();
    let profile_plan = discovery::plan_pack_profile(candidates, options.preservation_profile);
    discovery::emit_profile_warnings(&profile_plan.omitted);
    if profile_plan.included.is_empty() {
        bail!("no input files to pack");
    }
    let planning = presenter.begin_active_phase("planning", None);
    if let Err(err) = discovery::reject_duplicate_logical_paths(&profile_plan.included) {
        if let Some(timings) = phase_timings.as_mut() {
            timings.metadata = metadata_start.elapsed();
        }
        planning.settle(StatusWord::Failed, Some("duplicate logical paths"));
        return Err(err);
    }
    let layout = match planning::build_pack_layout_plan(profile_plan, options) {
        Ok(layout) => {
            if let Some(timings) = phase_timings.as_mut() {
                timings.metadata = metadata_start.elapsed();
            }
            planning.settle(StatusWord::Complete, None);
            layout
        }
        Err(err) => {
            if let Some(timings) = phase_timings.as_mut() {
                timings.metadata = metadata_start.elapsed();
            }
            planning.settle(StatusWord::Failed, None);
            return Err(err);
        }
    };
    let start = Instant::now();
    let file_count = layout.files.len();
    let total_size_bytes = layout.files.iter().map(|file| file.raw_len).sum::<u64>();
    let compression = presenter.begin_active_phase("compression", None);
    let serialization = presenter.begin_active_phase("serialization", None);
    if let Err(err) = emission::emit_archive_from_layout(
        layout,
        output,
        level,
        options,
        phase_timings.as_mut(),
        |phase, done, total| {
            let detail = format!("files={}/{}", group_u64(done), group_u64(total));
            match phase {
                PackProgressPhase::Compression => compression.set_detail(detail),
                PackProgressPhase::Serialization => serialization.set_detail(detail),
            }
        },
    ) {
        compression.settle(StatusWord::Failed, None);
        serialization.settle(StatusWord::Failed, None);
        return Err(err);
    }
    compression.settle(
        StatusWord::Complete,
        Some(&format!(
            "files={}/{}",
            group_u64(file_count as u64),
            group_u64(file_count as u64)
        )),
    );
    serialization.settle(
        StatusWord::Complete,
        Some(&format!(
            "files={}/{}",
            group_u64(file_count as u64),
            group_u64(file_count as u64)
        )),
    );
    let finalization = presenter.begin_active_phase("finalizing", None);
    let compressed_size_bytes = std::fs::metadata(output)
        .with_context(|| format!("stat {}", output.display()))?
        .len();
    let elapsed = start.elapsed();
    finalization.settle(StatusWord::Complete, None);
    let metrics = PackRunMetrics {
        files_packed: file_count as u64,
        total_size_bytes,
        compressed_size_bytes,
        elapsed,
    };
    let (ratio, reduction) =
        compression_metrics(metrics.total_size_bytes, metrics.compressed_size_bytes);
    presenter.result_summary(
        StatusWord::Complete,
        "archive emitted",
        &[
            ("archive", output.display().to_string()),
            ("files packed", group_u64(metrics.files_packed)),
            ("total size", human_size(metrics.total_size_bytes)),
            ("compressed size", human_size(metrics.compressed_size_bytes)),
            ("compression ratio", format!("{ratio:.2}x")),
            ("reduction", format!("{reduction:.1}%")),
            ("processing time", format_elapsed(metrics.elapsed)),
        ],
    );
    presenter.silent_summary(
        StatusWord::Complete,
        &[
            ("archive", output.display().to_string()),
            ("files", file_count.to_string()),
            ("time", format_elapsed(metrics.elapsed)),
        ],
    );
    if let Some(timings) = phase_timings {
        print_pack_phase_breakdown(timings);
    }
    Ok(())
}

fn normalize_archive_output_path(mut output: PathBuf) -> PathBuf {
    if output.extension().is_none_or(|ext| ext.is_empty()) {
        output.set_extension("crs");
    }
    output
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn compression_metrics(total_size: u64, compressed_size: u64) -> (f64, f64) {
    if total_size == 0 || compressed_size == 0 {
        return (0.0, 0.0);
    }
    let ratio = total_size as f64 / compressed_size as f64;
    let reduction = (1.0 - (compressed_size as f64 / total_size as f64)) * 100.0;
    (ratio, reduction)
}

fn format_elapsed(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs >= 3600 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        return format!("{hours}h {minutes}m");
    }
    if secs >= 60 {
        let minutes = secs / 60;
        let seconds = secs % 60;
        return format!("{minutes}m {seconds}s");
    }
    if secs >= 1 {
        return format!("{secs}s");
    }
    "<1s".to_string()
}

const POSIX_ACL_ACCESS_XATTR: &str = "system.posix_acl_access";
const POSIX_ACL_DEFAULT_XATTR: &str = "system.posix_acl_default";
const SELINUX_LABEL_XATTR: &str = "security.selinux";
const LINUX_CAPABILITY_XATTR: &str = "security.capability";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn baseline_options() -> PackExperimentalOptions {
        PackExperimentalOptions {
            preservation_profile: PreservationProfile::Full,
            profile_pack: false,
            self_describing_extents: false,
            file_identity_extents: false,
            self_identifying_blocks: false,
            file_manifest_checkpoints: false,
            metadata_profile: None,
            placement_strategy: None,
        }
    }

    #[cfg(unix)]
    #[test]
    fn planning_does_not_require_readable_file_contents() {
        use std::os::unix::fs::PermissionsExt;

        let td = TempDir::new().expect("tempdir");
        let unreadable = td.path().join("payload.bin");
        std::fs::write(&unreadable, b"payload").expect("write file");
        let mut perms = std::fs::metadata(&unreadable)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&unreadable, perms).expect("set permissions");

        let files = discovery::collect_files(
            &[unreadable],
            discovery::metadata_capture_requirements(PreservationProfile::Full),
        )
        .expect("collect files");
        let profile_plan = discovery::plan_pack_profile(files, PreservationProfile::Full);
        let layout = planning::build_pack_layout_plan(profile_plan, baseline_options())
            .expect("build layout");
        assert_eq!(layout.files.len(), 1);
        assert!(layout.files[0].raw_len > 0);
    }

    #[test]
    fn pack_fails_if_file_changes_between_planning_and_emit() {
        let td = TempDir::new().expect("tempdir");
        let input = td.path().join("payload.bin");
        std::fs::write(&input, b"before").expect("write file");
        let files = discovery::collect_files(
            std::slice::from_ref(&input),
            discovery::metadata_capture_requirements(PreservationProfile::Full),
        )
        .expect("collect files");
        let profile_plan = discovery::plan_pack_profile(files, PreservationProfile::Full);
        let layout = planning::build_pack_layout_plan(profile_plan, baseline_options())
            .expect("build layout");
        std::fs::write(&input, b"changed-content").expect("mutate file");

        let output = td.path().join("out.crs");
        let err = emission::emit_archive_from_layout(
            layout,
            &output,
            3,
            baseline_options(),
            None,
            |_, _, _| {},
        )
        .expect_err("emit should fail");
        assert!(
            err.to_string()
                .contains("input changed during pack planning"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn metadata_capture_requirements_follow_preservation_profile() {
        let full = discovery::metadata_capture_requirements(PreservationProfile::Full);
        assert!(full.capture_ownership_names);
        assert!(full.capture_xattrs);
        assert!(full.capture_sparse_layout);

        let basic = discovery::metadata_capture_requirements(PreservationProfile::Basic);
        assert!(!basic.capture_ownership_names);
        assert!(!basic.capture_xattrs);
        assert!(basic.capture_sparse_layout);

        let payload = discovery::metadata_capture_requirements(PreservationProfile::PayloadOnly);
        assert!(!payload.capture_ownership_names);
        assert!(!payload.capture_xattrs);
        assert!(!payload.capture_sparse_layout);
    }

    #[test]
    fn deterministic_compressor_handles_growing_inputs() {
        let mut compressor = DeterministicCompressor::new(3).expect("create compressor");
        let small = vec![b'a'; 128];
        let large = vec![b'b'; 2 * 1024 * 1024];

        let small_out = compressor.compress(&small).expect("compress small");
        assert!(!small_out.is_empty(), "small output should not be empty");

        let large_out = compressor.compress(&large).expect("compress large");
        assert!(!large_out.is_empty(), "large output should not be empty");
    }
}
