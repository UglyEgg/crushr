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

mod discovery {
    use super::*;

    pub(super) fn collect_files(inputs: &[PathBuf]) -> Result<Vec<InputFile>> {
        collect_files_impl(inputs)
    }

    pub(super) fn plan_pack_profile(
        candidates: Vec<InputFile>,
        profile: PreservationProfile,
    ) -> PackProfilePlan {
        plan_pack_profile_impl(candidates, profile)
    }

    pub(super) fn emit_profile_warnings(omissions: &[ProfileOmission]) {
        emit_profile_warnings_impl(omissions);
    }

    pub(super) fn reject_duplicate_logical_paths(files: &[InputFile]) -> Result<()> {
        reject_duplicate_logical_paths_impl(files)
    }
}

mod planning {
    use super::*;

    pub(super) fn build_pack_layout_plan(
        profile_plan: PackProfilePlan,
        options: PackExperimentalOptions,
    ) -> Result<PackLayoutPlan> {
        build_pack_layout_plan_impl(profile_plan, options)
    }
}

mod emission {
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
}

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
    let candidates = match discovery::collect_files(inputs) {
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
        let current = capture_mode_mtime_uid_gid(&current_meta);
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

const POSIX_ACL_ACCESS_XATTR: &str = "system.posix_acl_access";
const POSIX_ACL_DEFAULT_XATTR: &str = "system.posix_acl_default";
const SELINUX_LABEL_XATTR: &str = "security.selinux";
const LINUX_CAPABILITY_XATTR: &str = "security.capability";

#[derive(Debug, Default, Clone)]
struct CapturedSecurityMetadata {
    acl_access: Option<Vec<u8>>,
    acl_default: Option<Vec<u8>>,
    selinux_label: Option<Vec<u8>>,
    linux_capability: Option<Vec<u8>>,
}

fn capture_xattrs(path: &Path) -> (Vec<Xattr>, CapturedSecurityMetadata) {
    #[cfg(unix)]
    {
        let mut out = Vec::new();
        let mut security = CapturedSecurityMetadata::default();
        if let Ok(names) = xattr::list(path) {
            for name_os in names {
                let name = name_os.to_string_lossy().to_string();
                if let Ok(Some(value)) = xattr::get(path, &name_os) {
                    match name.as_str() {
                        POSIX_ACL_ACCESS_XATTR => security.acl_access = Some(value),
                        POSIX_ACL_DEFAULT_XATTR => security.acl_default = Some(value),
                        SELINUX_LABEL_XATTR => security.selinux_label = Some(value),
                        LINUX_CAPABILITY_XATTR => security.linux_capability = Some(value),
                        _ => out.push(Xattr { name, value }),
                    }
                }
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        (out, security)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        (Vec::new(), CapturedSecurityMetadata::default())
    }
}

struct CapturedMeta {
    mode: u32,
    mtime: i64,
    uid: u32,
    gid: u32,
    hardlink_key: Option<(u64, u64)>,
    device_major: Option<u32>,
    device_minor: Option<u32>,
}

fn capture_mode_mtime_uid_gid(meta: &std::fs::Metadata) -> CapturedMeta {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let hardlink_key = if meta.is_file() && meta.nlink() > 1 {
            Some((meta.dev(), meta.ino()))
        } else {
            None
        };
        let file_kind = meta.mode() & libc_s_ifmt();
        let (device_major, device_minor) =
            if file_kind == libc_s_ifchr() || file_kind == libc_s_ifblk() {
                let rdev = meta.rdev();
                (Some(libc_major(rdev) as u32), Some(libc_minor(rdev) as u32))
            } else {
                (None, None)
            };
        CapturedMeta {
            mode: meta.mode() & 0o7777,
            mtime: meta.mtime(),
            uid: meta.uid(),
            gid: meta.gid(),
            hardlink_key,
            device_major,
            device_minor,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        CapturedMeta {
            mode: 0,
            mtime: 0,
            uid: 0,
            gid: 0,
            hardlink_key: None,
            device_major: None,
            device_minor: None,
        }
    }
}

#[cfg(unix)]
fn capture_ownership_names(uid: u32, gid: u32) -> (Option<String>, Option<String>) {
    use std::ffi::CStr;
    #[repr(C)]
    struct Passwd {
        pw_name: *const std::os::raw::c_char,
        pw_passwd: *const std::os::raw::c_char,
        pw_uid: u32,
        pw_gid: u32,
        pw_gecos: *const std::os::raw::c_char,
        pw_dir: *const std::os::raw::c_char,
        pw_shell: *const std::os::raw::c_char,
    }
    #[repr(C)]
    struct Group {
        gr_name: *const std::os::raw::c_char,
        gr_passwd: *const std::os::raw::c_char,
        gr_gid: u32,
        gr_mem: *mut *mut std::os::raw::c_char,
    }
    unsafe extern "C" {
        fn getpwuid(uid: u32) -> *mut Passwd;
        fn getgrgid(gid: u32) -> *mut Group;
    }

    let uname = unsafe {
        let ptr = getpwuid(uid);
        if ptr.is_null() || (*ptr).pw_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*ptr).pw_name).to_string_lossy().to_string())
        }
    };
    let gname = unsafe {
        let ptr = getgrgid(gid);
        if ptr.is_null() || (*ptr).gr_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*ptr).gr_name).to_string_lossy().to_string())
        }
    };
    (uname, gname)
}

#[cfg(not(unix))]
fn capture_ownership_names(_uid: u32, _gid: u32) -> (Option<String>, Option<String>) {
    (None, None)
}

#[cfg(unix)]
fn capture_sparse_chunks(path: &Path, size: u64) -> Vec<SparseChunk> {
    use std::os::unix::fs::FileExt;
    use std::os::unix::io::AsRawFd;
    if size == 0 {
        return Vec::new();
    }
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let mut chunks = Vec::new();
    let mut cursor = 0u64;
    while cursor < size {
        let Some(data_off) = sparse_lseek(file.as_raw_fd(), cursor, libc_seek_data()) else {
            break;
        };
        if data_off >= size {
            break;
        }
        let hole_off = sparse_lseek(file.as_raw_fd(), data_off, libc_seek_hole()).unwrap_or(size);
        let end = hole_off.min(size);
        if end <= data_off {
            break;
        }
        chunks.push(SparseChunk {
            logical_offset: data_off,
            len: end - data_off,
        });
        cursor = end;
    }
    if chunks.is_empty() {
        let mut probe = [0u8; 1];
        let has_data = file.read_at(&mut probe, 0).ok().unwrap_or(0) > 0;
        if has_data {
            vec![SparseChunk {
                logical_offset: 0,
                len: size,
            }]
        } else {
            Vec::new()
        }
    } else {
        chunks
    }
}

#[cfg(not(unix))]
fn capture_sparse_chunks(_path: &Path, _size: u64) -> Vec<SparseChunk> {
    Vec::new()
}

fn collect_files_impl(inputs: &[PathBuf]) -> Result<Vec<InputFile>> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().context("read current working directory")?;
    let mut uname_by_uid = BTreeMap::<u32, Option<String>>::new();
    let mut gname_by_gid = BTreeMap::<u32, Option<String>>::new();

    for input in inputs {
        let abs = if input.is_absolute() {
            input.clone()
        } else {
            cwd.join(input)
        };
        let meta =
            std::fs::symlink_metadata(&abs).with_context(|| format!("stat {}", input.display()))?;

        if meta.is_file() {
            let name = abs
                .file_name()
                .context("input file has no file name")?
                .to_string_lossy()
                .to_string();
            let size = meta.len();
            let captured = capture_mode_mtime_uid_gid(&meta);
            let mode = captured.mode;
            let mtime = captured.mtime;
            let uname = uname_by_uid.entry(captured.uid).or_insert_with(|| {
                let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                uname
            });
            let gname = gname_by_gid.entry(captured.gid).or_insert_with(|| {
                let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                gname
            });
            let (uid, gid, uname, gname) =
                (captured.uid, captured.gid, uname.clone(), gname.clone());
            let (xattrs, security) = capture_xattrs(input);
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: abs,
                raw_len: size,
                kind: EntryKind::Regular,
                mode,
                mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: captured.hardlink_key,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: capture_sparse_chunks(input, size),
                device_major: None,
                device_minor: None,
            });
            continue;
        }

        if meta.file_type().is_symlink() {
            let name = abs
                .file_name()
                .context("input symlink has no file name")?
                .to_string_lossy()
                .to_string();
            let captured = capture_mode_mtime_uid_gid(&meta);
            let (uname, gname) = capture_ownership_names(captured.uid, captured.gid);
            let (uid, gid, uname, gname) = (captured.uid, captured.gid, uname, gname);
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: input.clone(),
                raw_len: 0,
                kind: EntryKind::Symlink,
                mode: captured.mode,
                mtime: captured.mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: None,
                xattrs: Vec::new(),
                acl_access: None,
                acl_default: None,
                selinux_label: None,
                linux_capability: None,
                sparse_chunks: Vec::new(),
                device_major: None,
                device_minor: None,
            });
            continue;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let captured = capture_mode_mtime_uid_gid(&meta);
            let ft = meta.file_type();
            let kind = if ft.is_fifo() {
                Some(EntryKind::Fifo)
            } else if ft.is_char_device() {
                Some(EntryKind::CharDevice)
            } else if ft.is_block_device() {
                Some(EntryKind::BlockDevice)
            } else {
                None
            };
            if let Some(kind) = kind {
                let name = abs
                    .file_name()
                    .context("input special file has no file name")?
                    .to_string_lossy()
                    .to_string();
                let (uname, gname) = capture_ownership_names(captured.uid, captured.gid);
                files.push(InputFile {
                    rel_path: normalize_logical_path(&name),
                    abs_path: abs,
                    raw_len: 0,
                    kind,
                    mode: captured.mode,
                    mtime: captured.mtime,
                    uid: captured.uid,
                    gid: captured.gid,
                    uname,
                    gname,
                    hardlink_key: captured.hardlink_key,
                    xattrs: Vec::new(),
                    acl_access: None,
                    acl_default: None,
                    selinux_label: None,
                    linux_capability: None,
                    sparse_chunks: Vec::new(),
                    device_major: captured.device_major,
                    device_minor: captured.device_minor,
                });
                continue;
            }
        }

        if !meta.is_dir() {
            bail!("unsupported input type: {}", input.display());
        }

        let mut saw_child = false;
        for entry in walkdir::WalkDir::new(&abs).follow_links(false) {
            let entry = entry?;
            let rel = entry
                .path()
                .strip_prefix(&abs)
                .context("strip input prefix")?
                .to_string_lossy()
                .to_string();
            if rel.is_empty() {
                continue;
            }
            saw_child = true;
            let rel_path = normalize_logical_path(&rel);
            let entry_meta = std::fs::symlink_metadata(entry.path())
                .with_context(|| format!("stat {}", entry.path().display()))?;
            let captured = capture_mode_mtime_uid_gid(&entry_meta);
            #[cfg(unix)]
            use std::os::unix::fs::FileTypeExt;
            let kind = if entry.file_type().is_file() {
                EntryKind::Regular
            } else if entry.file_type().is_symlink() {
                EntryKind::Symlink
            } else if entry.file_type().is_dir() {
                EntryKind::Directory
            } else if entry_meta.file_type().is_fifo() {
                EntryKind::Fifo
            } else if entry_meta.file_type().is_char_device() {
                EntryKind::CharDevice
            } else if entry_meta.file_type().is_block_device() {
                EntryKind::BlockDevice
            } else {
                continue;
            };
            let (xattrs, security) = if kind == EntryKind::Regular || kind == EntryKind::Directory {
                capture_xattrs(entry.path())
            } else {
                (Vec::new(), CapturedSecurityMetadata::default())
            };
            let (mode, mtime) = (captured.mode, captured.mtime);
            let uname = uname_by_uid.entry(captured.uid).or_insert_with(|| {
                let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                uname
            });
            let gname = gname_by_gid.entry(captured.gid).or_insert_with(|| {
                let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                gname
            });
            let (uid, gid, uname, gname) =
                (captured.uid, captured.gid, uname.clone(), gname.clone());

            files.push(InputFile {
                rel_path,
                abs_path: entry.path().to_path_buf(),
                raw_len: if kind == EntryKind::Regular {
                    entry_meta.len()
                } else {
                    0
                },
                kind,
                mode,
                mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: captured.hardlink_key,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: if kind == EntryKind::Regular {
                    capture_sparse_chunks(entry.path(), entry_meta.len())
                } else {
                    Vec::new()
                },
                device_major: captured.device_major,
                device_minor: captured.device_minor,
            });
        }
        if !saw_child {
            let captured = capture_mode_mtime_uid_gid(&meta);
            let (xattrs, security) = capture_xattrs(input);
            let name = abs
                .file_name()
                .context("input directory has no file name")?
                .to_string_lossy()
                .to_string();
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: abs,
                raw_len: 0,
                kind: EntryKind::Directory,
                mode: captured.mode,
                mtime: captured.mtime,
                uid: captured.uid,
                gid: captured.gid,
                uname: {
                    let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                    uname
                },
                gname: {
                    let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                    gname
                },
                hardlink_key: None,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: Vec::new(),
                device_major: None,
                device_minor: None,
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

fn plan_pack_profile_impl(
    candidates: Vec<InputFile>,
    profile: PreservationProfile,
) -> PackProfilePlan {
    let mut included = Vec::with_capacity(candidates.len());
    let mut omitted = Vec::new();

    for mut entry in candidates {
        let omission_reason = match (profile, entry.kind) {
            (PreservationProfile::Basic, EntryKind::Fifo)
            | (PreservationProfile::Basic, EntryKind::CharDevice)
            | (PreservationProfile::Basic, EntryKind::BlockDevice) => {
                Some(ProfileOmissionReason::BasicOmitsSpecialEntries)
            }
            (PreservationProfile::PayloadOnly, EntryKind::Symlink) => {
                Some(ProfileOmissionReason::PayloadOnlyOmitsSymlinks)
            }
            (PreservationProfile::PayloadOnly, EntryKind::Fifo)
            | (PreservationProfile::PayloadOnly, EntryKind::CharDevice)
            | (PreservationProfile::PayloadOnly, EntryKind::BlockDevice) => {
                Some(ProfileOmissionReason::PayloadOnlyOmitsSpecialEntries)
            }
            _ => None,
        };

        if let Some(reason) = omission_reason {
            omitted.push(ProfileOmission {
                rel_path: entry.rel_path,
                kind: entry.kind,
                reason,
            });
            continue;
        }

        match profile {
            PreservationProfile::Full => {}
            PreservationProfile::Basic => {
                entry.xattrs.clear();
                entry.uid = 0;
                entry.gid = 0;
                entry.uname = None;
                entry.gname = None;
                entry.acl_access = None;
                entry.acl_default = None;
                entry.selinux_label = None;
                entry.linux_capability = None;
            }
            PreservationProfile::PayloadOnly => {
                match entry.kind {
                    EntryKind::Regular => {
                        entry.mode = 0;
                        entry.mtime = -1;
                        entry.hardlink_key = None;
                        entry.sparse_chunks.clear();
                    }
                    EntryKind::Directory => {
                        entry.mode = 0;
                        entry.mtime = -1;
                    }
                    _ => {}
                }
                entry.xattrs.clear();
                entry.uid = 0;
                entry.gid = 0;
                entry.uname = None;
                entry.gname = None;
                entry.acl_access = None;
                entry.acl_default = None;
                entry.selinux_label = None;
                entry.linux_capability = None;
                entry.device_major = None;
                entry.device_minor = None;
            }
        }
        included.push(entry);
    }

    PackProfilePlan { included, omitted }
}

fn emit_profile_warnings_impl(omissions: &[ProfileOmission]) {
    for omission in omissions {
        eprintln!(
            "WARNING[preservation-omit]: omitted '{}' ({:?}) due to preservation profile {}",
            omission.rel_path,
            omission.kind,
            omission.reason.profile_name()
        );
    }
}

impl ProfileOmissionReason {
    fn profile_name(self) -> &'static str {
        match self {
            ProfileOmissionReason::BasicOmitsSpecialEntries => "basic",
            ProfileOmissionReason::PayloadOnlyOmitsSymlinks
            | ProfileOmissionReason::PayloadOnlyOmitsSpecialEntries => "payload-only",
        }
    }
}

fn normalize_logical_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(unix)]
fn libc_s_ifmt() -> u32 {
    0o170000
}
#[cfg(not(unix))]
fn libc_s_ifmt() -> u32 {
    0
}
#[cfg(unix)]
fn libc_s_ifchr() -> u32 {
    0o020000
}
#[cfg(not(unix))]
fn libc_s_ifchr() -> u32 {
    0
}
#[cfg(unix)]
fn libc_s_ifblk() -> u32 {
    0o060000
}
#[cfg(not(unix))]
fn libc_s_ifblk() -> u32 {
    0
}
#[cfg(unix)]
fn libc_seek_data() -> i32 {
    3
}
#[cfg(unix)]
fn libc_seek_hole() -> i32 {
    4
}
#[cfg(unix)]
fn libc_major(dev: u64) -> u64 {
    ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)
}
#[cfg(unix)]
fn libc_minor(dev: u64) -> u64 {
    (dev & 0xff) | ((dev >> 12) & !0xff)
}
#[cfg(unix)]
fn sparse_lseek(fd: std::os::unix::io::RawFd, off: u64, whence: i32) -> Option<u64> {
    unsafe extern "C" {
        fn lseek(fd: std::os::unix::io::RawFd, offset: i64, whence: i32) -> i64;
    }
    let value = unsafe { lseek(fd, off as i64, whence) };
    if value < 0 { None } else { Some(value as u64) }
}

fn reject_duplicate_logical_paths_impl(files: &[InputFile]) -> Result<()> {
    let mut path_sources: BTreeMap<&str, Vec<(EntryKind, String)>> = BTreeMap::new();

    for file in files {
        path_sources
            .entry(file.rel_path.as_str())
            .or_default()
            .push((file.kind, file.abs_path.display().to_string()));
    }

    for (logical_path, mut sources) in path_sources {
        let all_dirs = sources
            .iter()
            .all(|(kind, _)| *kind == EntryKind::Directory);
        if sources.len() > 1 && !all_dirs {
            let mut source_paths = sources
                .drain(..)
                .map(|(_, source)| source)
                .collect::<Vec<_>>();
            source_paths.sort();
            bail!(
                "duplicate logical archive path '{logical_path}' from inputs: {}",
                source_paths.join(", ")
            );
        }
    }

    Ok(())
}

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

        let files = discovery::collect_files(&[unreadable]).expect("collect files");
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
        let files = discovery::collect_files(std::slice::from_ref(&input)).expect("collect files");
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
