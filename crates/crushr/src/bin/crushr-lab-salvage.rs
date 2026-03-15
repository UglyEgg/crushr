use anyhow::{bail, Context, Result};
use crushr_format::blk3::BLK3_MAGIC;
use crushr_format::ftr4::{Ftr4, FTR4_LEN};
use crushr_format::tailframe::{assemble_tail_frame, parse_tail_frame};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: crushr-lab-salvage <input_dir> --output <experiment_dir> [--export-fragments] [--limit <N>] [--verbose]\n       crushr-lab-salvage --resummarize <experiment_dir>\n       crushr-lab-salvage run-redundant-map-comparison --output <comparison_dir> [--verbose]\n       crushr-lab-salvage run-experimental-resilience-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-file-identity-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format04-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format05-comparison --output <comparison_dir> [--verbose]";
const VERIFICATION_LABEL: &str = "UNVERIFIED_RESEARCH_OUTPUT";
const EXPERIMENT_SCHEMA_VERSION: &str = "crushr-lab-salvage-experiment.v1";
const SUMMARY_SCHEMA_VERSION: &str = "crushr-lab-salvage-summary.v1";
const ANALYSIS_SCHEMA_VERSION: &str = "crushr-lab-salvage-analysis.v1";
const OUTCOME_ORDER: [&str; 4] = [
    "FULL_FILE_SALVAGE_AVAILABLE",
    "PARTIAL_FILE_SALVAGE",
    "ORPHAN_EVIDENCE_ONLY",
    "NO_VERIFIED_EVIDENCE",
];

#[derive(Debug)]
struct CliOptions {
    mode: Mode,
    export_fragments: bool,
    limit: Option<usize>,
    verbose: bool,
}

#[derive(Debug)]
enum Mode {
    Help,
    RunExperiment {
        input_dir: PathBuf,
        experiment_dir: PathBuf,
    },
    Resummarize {
        experiment_dir: PathBuf,
    },
    RunRedundantMapComparison {
        comparison_dir: PathBuf,
    },
    RunExperimentalResilienceComparison {
        comparison_dir: PathBuf,
    },
    RunFileIdentityComparison {
        comparison_dir: PathBuf,
    },
    RunFormat04Comparison {
        comparison_dir: PathBuf,
    },
    RunFormat05Comparison {
        comparison_dir: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Format05ScenarioRow {
    scenario_id: String,
    dataset: String,
    corruption_model: String,
    corruption_target: String,
    magnitude: String,
    seed: u64,
    old_outcome: String,
    redundant_outcome: String,
    experimental_outcome: String,
    file_identity_outcome: String,
    format05_outcome: String,
    old_verified_block_count: u64,
    redundant_verified_block_count: u64,
    experimental_verified_block_count: u64,
    file_identity_verified_block_count: u64,
    format05_verified_block_count: u64,
    old_salvageable_file_count: u64,
    redundant_salvageable_file_count: u64,
    experimental_salvageable_file_count: u64,
    file_identity_salvageable_file_count: u64,
    format05_salvageable_file_count: u64,
    old_exported_full_file_count: u64,
    redundant_exported_full_file_count: u64,
    experimental_exported_full_file_count: u64,
    file_identity_exported_full_file_count: u64,
    format05_exported_full_file_count: u64,
}

#[derive(Debug, Serialize)]
struct Format05ComparisonSummary {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    verification_label: &'static str,
    scenario_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    redundant_outcome_counts: BTreeMap<String, u64>,
    experimental_outcome_counts: BTreeMap<String, u64>,
    file_identity_outcome_counts: BTreeMap<String, u64>,
    format05_outcome_counts: BTreeMap<String, u64>,
    orphan_to_partial_improvements_vs_old: u64,
    orphan_to_full_improvements_vs_old: u64,
    no_evidence_to_partial_improvements_vs_old: u64,
    no_evidence_to_full_improvements_vs_old: u64,
    total_verified_block_delta_vs_old: i64,
    total_salvageable_file_delta_vs_old: i64,
    total_exported_full_file_delta_vs_old: i64,
    by_dataset: Vec<Format05ComparisonGroup>,
    by_corruption_target: Vec<Format05ComparisonGroup>,
    per_scenario_rows: Vec<Format05ScenarioRow>,
}

#[derive(Debug, Serialize)]
struct Format05ComparisonGroup {
    key: String,
    scenario_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    redundant_outcome_counts: BTreeMap<String, u64>,
    experimental_outcome_counts: BTreeMap<String, u64>,
    file_identity_outcome_counts: BTreeMap<String, u64>,
    format05_outcome_counts: BTreeMap<String, u64>,
    verified_block_delta_vs_old: i64,
    salvageable_file_delta_vs_old: i64,
    exported_full_file_delta_vs_old: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ExperimentalScenarioRow {
    scenario_id: String,
    dataset: String,
    corruption_model: String,
    corruption_target: String,
    magnitude: String,
    seed: u64,
    old_outcome: String,
    redundant_outcome: String,
    experimental_outcome: String,
    old_verified_block_count: u64,
    redundant_verified_block_count: u64,
    experimental_verified_block_count: u64,
    old_salvageable_file_count: u64,
    redundant_salvageable_file_count: u64,
    experimental_salvageable_file_count: u64,
    old_exported_full_file_count: u64,
    redundant_exported_full_file_count: u64,
    experimental_exported_full_file_count: u64,
    file_identity_outcome: String,
    file_identity_verified_block_count: u64,
    file_identity_salvageable_file_count: u64,
    file_identity_exported_full_file_count: u64,
}

#[derive(Debug, Serialize)]
struct ExperimentalComparisonSummary {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    verification_label: &'static str,
    scenario_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    redundant_outcome_counts: BTreeMap<String, u64>,
    experimental_outcome_counts: BTreeMap<String, u64>,
    file_identity_outcome_counts: BTreeMap<String, u64>,
    orphan_to_salvage_improvements_vs_old: u64,
    orphan_to_partial_improvements_vs_old: u64,
    orphan_to_full_improvements_vs_old: u64,
    orphan_to_salvage_improvements_vs_redundant: u64,
    no_evidence_to_partial_improvements_vs_old: u64,
    no_evidence_to_full_improvements_vs_old: u64,
    total_verified_block_delta_vs_old: i64,
    total_salvageable_file_delta_vs_old: i64,
    total_exported_full_file_delta_vs_old: i64,
    by_dataset: Vec<ExperimentalComparisonGroup>,
    by_corruption_target: Vec<ExperimentalComparisonGroup>,
    per_scenario_rows: Vec<ExperimentalScenarioRow>,
}

#[derive(Debug, Serialize)]
struct ExperimentalComparisonGroup {
    key: String,
    scenario_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    redundant_outcome_counts: BTreeMap<String, u64>,
    experimental_outcome_counts: BTreeMap<String, u64>,
    file_identity_outcome_counts: BTreeMap<String, u64>,
    verified_block_delta_vs_old: i64,
    salvageable_file_delta_vs_old: i64,
    exported_full_file_delta_vs_old: i64,
}

#[derive(Debug, Clone)]
struct ArchiveRun {
    source_path: PathBuf,
    archive_path: String,
    archive_fingerprint: String,
    archive_id: String,
}

#[derive(Debug, Serialize)]
struct ExperimentManifest {
    experiment_id: String,
    tool_version: &'static str,
    schema_version: &'static str,
    run_count: usize,
    run_timestamp: String,
    verification_label: &'static str,
    export_fragments_enabled: bool,
    archive_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExperimentManifestInput {
    experiment_id: String,
    verification_label: String,
    archive_list: Vec<String>,
    #[serde(default)]
    export_fragments_enabled: bool,
}

#[derive(Debug, Serialize)]
struct RunMetadata {
    archive_id: String,
    archive_path: String,
    archive_fingerprint: String,
    salvage_plan_summary: PlanSummary,
    verified_block_count: u64,
    exported_artifact_count: usize,
    exported_block_artifact_count: usize,
    exported_extent_artifact_count: usize,
    exported_full_file_artifact_count: usize,
    salvageable_file_count: u64,
    unsalvageable_file_count: u64,
    unmappable_file_count: u64,
}

#[derive(Debug, Deserialize)]
struct RunMetadataInput {
    archive_path: String,
    archive_fingerprint: String,
    verified_block_count: u64,
    salvageable_file_count: u64,
    unsalvageable_file_count: u64,
    unmappable_file_count: u64,
    #[serde(default)]
    exported_block_artifact_count: Option<usize>,
    #[serde(default)]
    exported_extent_artifact_count: Option<usize>,
    #[serde(default)]
    exported_full_file_artifact_count: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ExperimentSummary {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    experiment_id: String,
    verification_label: &'static str,
    run_count: usize,
    archives_with_verified_blocks: usize,
    archives_with_salvageable_files: usize,
    archives_with_only_orphan_evidence: usize,
    archives_with_no_verified_evidence: usize,
    total_verified_blocks: u64,
    total_exported_block_artifacts: usize,
    total_exported_extent_artifacts: usize,
    total_exported_full_file_artifacts: usize,
    total_salvageable_files: u64,
    total_unsalvageable_files: u64,
    total_unmappable_files: u64,
    runs: Vec<RunSummaryRow>,
}

#[derive(Debug, Serialize)]
struct RunSummaryRow {
    archive_id: String,
    archive_path: String,
    archive_fingerprint: String,
    verified_block_count: u64,
    salvageable_file_count: u64,
    unsalvageable_file_count: u64,
    unmappable_file_count: u64,
    exported_block_artifact_count: usize,
    exported_extent_artifact_count: usize,
    exported_full_file_artifact_count: usize,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct PlanSummary {
    salvageable_files: u64,
    unsalvageable_files: u64,
    unmappable_files: u64,
}

#[derive(Debug, Serialize)]
struct ExperimentAnalysis {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    experiment_id: String,
    verification_label: &'static str,
    run_count: usize,
    grouping_strategy: GroupingStrategy,
    outcome_groups: Vec<OutcomeGroup>,
    export_mode_groups: Vec<ExportModeGroup>,
    evidence_rankings: EvidenceRankings,
    profile_groups: Vec<ProfileGroup>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GroupingStrategy {
    outcome_order: Vec<&'static str>,
    profile_inference_priority: Vec<&'static str>,
    ranking_tie_breaker: &'static str,
}

#[derive(Debug, Serialize)]
struct OutcomeGroup {
    outcome: &'static str,
    run_count: usize,
    archive_ids: Vec<String>,
    aggregate_verified_block_count: u64,
    aggregate_salvageable_file_count: u64,
    aggregate_exported_block_artifact_count: usize,
    aggregate_exported_extent_artifact_count: usize,
    aggregate_exported_full_file_artifact_count: usize,
}

#[derive(Debug, Serialize)]
struct ExportModeGroup {
    export_fragments_enabled: bool,
    run_count: usize,
    aggregate_exported_block_artifact_count: usize,
    aggregate_exported_extent_artifact_count: usize,
    aggregate_exported_full_file_artifact_count: usize,
}

#[derive(Debug, Serialize)]
struct EvidenceRankings {
    top_runs_by_verified_blocks: Vec<RankingEntry>,
    top_runs_by_salvageable_files: Vec<RankingEntry>,
    top_runs_by_exported_full_files: Vec<RankingEntry>,
    bottom_runs_with_no_verified_evidence: Vec<RankingEntry>,
}

#[derive(Debug, Serialize)]
struct RankingEntry {
    archive_id: String,
    archive_path: String,
    verified_block_count: u64,
    salvageable_file_count: u64,
    exported_full_file_artifact_count: usize,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct ProfileGroup {
    profile_key: String,
    run_count: usize,
    outcome_counts: BTreeMap<&'static str, usize>,
    aggregate_verified_block_count: u64,
    aggregate_salvageable_file_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ScenarioRow {
    scenario_id: String,
    dataset: String,
    corruption_model: String,
    corruption_target: String,
    magnitude: String,
    seed: u64,
    old_outcome: String,
    new_outcome: String,
    old_verified_block_count: u64,
    new_verified_block_count: u64,
    old_salvageable_file_count: u64,
    new_salvageable_file_count: u64,
    old_exported_full_file_count: u64,
    new_exported_full_file_count: u64,
    improvement_class: String,
}

#[derive(Debug, Serialize)]
struct ComparisonSummary {
    schema_version: &'static str,
    tool: &'static str,
    tool_version: &'static str,
    verification_label: &'static str,
    scenario_count: usize,
    old_archive_count: usize,
    new_archive_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    new_outcome_counts: BTreeMap<String, u64>,
    orphan_to_salvage_improvements: u64,
    no_evidence_to_salvage_improvements: u64,
    unchanged_outcome_count: u64,
    degraded_outcome_count: u64,
    total_verified_block_delta: i64,
    total_salvageable_file_delta: i64,
    total_exported_full_file_delta: i64,
    by_dataset: Vec<ComparisonGroup>,
    by_corruption_target: Vec<ComparisonGroup>,
    by_corruption_model: Vec<ComparisonGroup>,
    by_magnitude: Vec<ComparisonGroup>,
    per_scenario_rows: Vec<ScenarioRow>,
}

#[derive(Debug, Serialize)]
struct ComparisonGroup {
    key: String,
    scenario_count: usize,
    old_outcome_counts: BTreeMap<String, u64>,
    new_outcome_counts: BTreeMap<String, u64>,
    verified_block_delta: i64,
    salvageable_file_delta: i64,
    exported_full_file_delta: i64,
}

#[derive(Debug, Clone)]
struct ComparisonScenario {
    scenario_id: String,
    dataset: &'static str,
    corruption_model: &'static str,
    corruption_target: &'static str,
    magnitude: &'static str,
    seed: u64,
    break_redundant_map: bool,
}

#[derive(Debug)]
struct OutcomeMetrics {
    outcome: String,
    verified_block_count: u64,
    salvageable_file_count: u64,
    exported_full_file_count: u64,
}

fn parse_cli_options() -> Result<CliOptions> {
    let mut args = std::env::args().skip(1);

    if let Some(first) = args.next() {
        if first == "--help" || first == "-h" || first == "help" {
            return Ok(CliOptions {
                mode: Mode::Help,
                export_fragments: false,
                limit: None,
                verbose: false,
            });
        }

        if first == "run-redundant-map-comparison" {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: Mode::RunRedundantMapComparison {
                    comparison_dir: output_dir.context(USAGE)?,
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }
        if first == "run-experimental-resilience-comparison" {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: Mode::RunExperimentalResilienceComparison {
                    comparison_dir: output_dir.context(USAGE)?,
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }

        if first == "run-file-identity-comparison"
            || first == "run-format04-comparison"
            || first == "run-format05-comparison"
        {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: if first == "run-format04-comparison" {
                    Mode::RunFormat04Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format05-comparison" {
                    Mode::RunFormat05Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else {
                    Mode::RunFileIdentityComparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }

        let mut input_dir = None;
        let mut output_dir = None;
        let mut resummarize_dir = None;
        let mut export_fragments = false;
        let mut limit = None;
        let mut verbose = false;

        let mut pending = Some(first);
        loop {
            let arg = if let Some(value) = pending.take() {
                value
            } else if let Some(value) = args.next() {
                value
            } else {
                break;
            };

            match arg.as_str() {
                "--output" => {
                    output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                }
                "--export-fragments" => {
                    export_fragments = true;
                }
                "--limit" => {
                    let value = args.next().context(USAGE)?;
                    limit = Some(
                        value
                            .parse::<usize>()
                            .with_context(|| format!("invalid --limit value: {value}"))?,
                    );
                }
                "--verbose" => {
                    verbose = true;
                }
                "--resummarize" => {
                    resummarize_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                }
                "--help" | "-h" => {
                    return Ok(CliOptions {
                        mode: Mode::Help,
                        export_fragments: false,
                        limit: None,
                        verbose: false,
                    });
                }
                "run-redundant-map-comparison"
                | "run-experimental-resilience-comparison"
                | "run-file-identity-comparison"
                | "run-format04-comparison"
                | "run-format05-comparison" => {
                    bail!("subcommand `{arg}` must be used as the first argument\n{USAGE}")
                }
                _ if arg.starts_with('-') => bail!("unsupported flag: {arg}"),
                _ if input_dir.is_none() => input_dir = Some(PathBuf::from(arg)),
                _ => bail!("unexpected argument: {arg}"),
            }
        }

        let mode = if let Some(experiment_dir) = resummarize_dir {
            if input_dir.is_some() || limit.is_some() || export_fragments || output_dir.is_some() {
                bail!("--resummarize cannot be combined with run flags");
            }
            Mode::Resummarize { experiment_dir }
        } else {
            Mode::RunExperiment {
                input_dir: input_dir.context(USAGE)?,
                experiment_dir: output_dir.context(USAGE)?,
            }
        };

        Ok(CliOptions {
            mode,
            export_fragments,
            limit,
            verbose,
        })
    } else {
        bail!(USAGE)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn archive_id(archive_path: &str, archive_fingerprint: &str) -> String {
    let digest = blake3::hash(format!("{archive_path}\n{archive_fingerprint}").as_bytes());
    format!(
        "{}-{}",
        sanitize_component(archive_path),
        to_hex(&digest.as_bytes()[..8])
    )
}

fn sanitize_component(value: &str) -> String {
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

fn collect_archives(opts: &CliOptions) -> Result<Vec<ArchiveRun>> {
    let input_dir = match &opts.mode {
        Mode::RunExperiment { input_dir, .. } => input_dir,
        Mode::Help
        | Mode::Resummarize { .. }
        | Mode::RunRedundantMapComparison { .. }
        | Mode::RunExperimentalResilienceComparison { .. }
        | Mode::RunFileIdentityComparison { .. }
        | Mode::RunFormat04Comparison { .. }
        | Mode::RunFormat05Comparison { .. } => {
            bail!("internal error: collect_archives outside run mode")
        }
    };
    let mut archives = Vec::new();
    for entry in fs::read_dir(input_dir).with_context(|| format!("read {}", input_dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        if !is_archive_bytes(&bytes) {
            if opts.verbose {
                eprintln!("skip non-archive: {}", path.display());
            }
            continue;
        }

        let rel = path
            .strip_prefix(input_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let fingerprint = to_hex(blake3::hash(&bytes).as_bytes());
        archives.push(ArchiveRun {
            source_path: path,
            archive_path: rel.clone(),
            archive_fingerprint: fingerprint.clone(),
            archive_id: archive_id(&rel, &fingerprint),
        });
    }

    archives.sort_by(|a, b| a.archive_path.cmp(&b.archive_path));
    if let Some(limit) = opts.limit {
        archives.truncate(limit);
    }
    Ok(archives)
}

fn is_archive_bytes(bytes: &[u8]) -> bool {
    if bytes.starts_with(&BLK3_MAGIC) {
        return true;
    }

    if bytes.len() < FTR4_LEN {
        return false;
    }

    let footer_offset = bytes.len() - FTR4_LEN;
    let Ok(footer) = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..])) else {
        return false;
    };
    let Ok(index_offset) = usize::try_from(footer.index_offset) else {
        return false;
    };
    let Ok(index_len) = usize::try_from(footer.index_len) else {
        return false;
    };
    if index_len < 4 {
        return false;
    }
    let Some(index_end) = index_offset.checked_add(index_len) else {
        return false;
    };
    if index_end > footer_offset {
        return false;
    }

    bytes[index_offset..index_offset + 4] == *b"IDX3"
}

fn resolve_salvage_bin() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CRUSHR_SALVAGE_BIN") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        bail!(
            "CRUSHR_SALVAGE_BIN points to missing/non-file path: {}",
            candidate.display()
        );
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let candidate = exe_dir.join(format!("crushr-salvage{}", std::env::consts::EXE_SUFFIX));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if let Ok(path) = std::env::var("CARGO_BIN_EXE_crushr-salvage") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    bail!(
        "unable to resolve crushr-salvage binary; expected sibling executable near current binary or set CRUSHR_SALVAGE_BIN to an explicit path"
    );
}

fn exported_artifact_counts(plan: &Value) -> (usize, usize, usize) {
    let Some(exported) = plan.get("exported_artifacts") else {
        return (0, 0, 0);
    };

    let count = |field: &str| {
        exported
            .get(field)
            .and_then(Value::as_array)
            .map_or(0, |entries| entries.len())
    };

    (
        count("exported_block_artifacts"),
        count("exported_fragment_artifacts"),
        count("exported_complete_file_artifacts"),
    )
}

fn run_salvage(archive: &ArchiveRun, run_dir: &Path, opts: &CliOptions) -> Result<RunMetadata> {
    let plan_path = run_dir.join("salvage_plan.json");
    let export_dir = run_dir.join("exported_artifacts");

    let salvage_bin = resolve_salvage_bin()?;
    let mut cmd = Command::new(&salvage_bin);
    cmd.arg(&archive.source_path)
        .arg("--json-out")
        .arg(&plan_path);
    if opts.export_fragments {
        cmd.arg("--export-fragments").arg(&export_dir);
    }

    let output = cmd.output().with_context(|| format!("run {:?}", cmd))?;
    if !output.status.success() {
        bail!(
            "crushr-salvage failed for {}\nstdout:\n{}\nstderr:\n{}",
            archive.source_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let plan: Value = serde_json::from_slice(
        &fs::read(&plan_path).with_context(|| format!("read {}", plan_path.display()))?,
    )
    .with_context(|| format!("parse {}", plan_path.display()))?;

    let summary = plan.get("summary").context("missing summary")?;
    let salvageable = summary
        .get("salvageable_files")
        .and_then(Value::as_u64)
        .context("missing summary.salvageable_files")?;
    let unsalvageable = summary
        .get("unsalvageable_files")
        .and_then(Value::as_u64)
        .context("missing summary.unsalvageable_files")?;
    let unmappable = summary
        .get("unmappable_files")
        .and_then(Value::as_u64)
        .context("missing summary.unmappable_files")?;

    let verified_block_count = plan
        .get("block_candidates")
        .and_then(Value::as_array)
        .map_or(0, |candidates| {
            candidates
                .iter()
                .filter(|candidate| {
                    candidate
                        .get("content_verification_status")
                        .and_then(Value::as_str)
                        == Some("content_verified")
                })
                .count() as u64
        });

    let (
        exported_block_artifact_count,
        exported_extent_artifact_count,
        exported_full_file_artifact_count,
    ) = exported_artifact_counts(&plan);

    Ok(RunMetadata {
        archive_id: archive.archive_id.clone(),
        archive_path: archive.archive_path.clone(),
        archive_fingerprint: archive.archive_fingerprint.clone(),
        salvage_plan_summary: PlanSummary {
            salvageable_files: salvageable,
            unsalvageable_files: unsalvageable,
            unmappable_files: unmappable,
        },
        verified_block_count,
        exported_artifact_count: exported_block_artifact_count
            + exported_extent_artifact_count
            + exported_full_file_artifact_count,
        exported_block_artifact_count,
        exported_extent_artifact_count,
        exported_full_file_artifact_count,
        salvageable_file_count: salvageable,
        unsalvageable_file_count: unsalvageable,
        unmappable_file_count: unmappable,
    })
}

fn classify_outcome(run: &RunMetadata) -> &'static str {
    if run.exported_full_file_artifact_count > 0 {
        "FULL_FILE_SALVAGE_AVAILABLE"
    } else if run.salvageable_file_count > 0 {
        "PARTIAL_FILE_SALVAGE"
    } else if run.verified_block_count > 0 {
        "ORPHAN_EVIDENCE_ONLY"
    } else {
        "NO_VERIFIED_EVIDENCE"
    }
}

fn infer_profile_key(path: &str) -> String {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    for marker in ["_profile_", "-profile-", "_damage_", "-damage-"] {
        if let Some(idx) = name.find(marker) {
            let suffix = &name[idx + marker.len()..];
            let key = suffix.split(['.', '_', '-']).next().unwrap_or("").trim();
            if !key.is_empty() {
                return key.to_ascii_uppercase();
            }
        }
    }
    "UNKNOWN_PROFILE".to_string()
}

fn to_ranking_entry(run: &RunMetadata) -> RankingEntry {
    RankingEntry {
        archive_id: run.archive_id.clone(),
        archive_path: run.archive_path.clone(),
        verified_block_count: run.verified_block_count,
        salvageable_file_count: run.salvageable_file_count,
        exported_full_file_artifact_count: run.exported_full_file_artifact_count,
        outcome: classify_outcome(run),
    }
}

fn generate_analysis_files(
    experiment_dir: &Path,
    experiment_id: &str,
    export_fragments_enabled: bool,
    runs: &[RunMetadata],
) -> Result<()> {
    let mut ordered_runs: Vec<&RunMetadata> = runs.iter().collect();
    ordered_runs.sort_by(|a, b| a.archive_id.cmp(&b.archive_id));

    let mut outcome_groups = Vec::new();
    for outcome in OUTCOME_ORDER {
        let grouped: Vec<&RunMetadata> = ordered_runs
            .iter()
            .copied()
            .filter(|run| classify_outcome(run) == outcome)
            .collect();
        outcome_groups.push(OutcomeGroup {
            outcome,
            run_count: grouped.len(),
            archive_ids: grouped.iter().map(|run| run.archive_id.clone()).collect(),
            aggregate_verified_block_count: grouped
                .iter()
                .map(|run| run.verified_block_count)
                .sum(),
            aggregate_salvageable_file_count: grouped
                .iter()
                .map(|run| run.salvageable_file_count)
                .sum(),
            aggregate_exported_block_artifact_count: grouped
                .iter()
                .map(|run| run.exported_block_artifact_count)
                .sum(),
            aggregate_exported_extent_artifact_count: grouped
                .iter()
                .map(|run| run.exported_extent_artifact_count)
                .sum(),
            aggregate_exported_full_file_artifact_count: grouped
                .iter()
                .map(|run| run.exported_full_file_artifact_count)
                .sum(),
        });
    }

    let export_mode_groups = vec![ExportModeGroup {
        export_fragments_enabled,
        run_count: ordered_runs.len(),
        aggregate_exported_block_artifact_count: ordered_runs
            .iter()
            .map(|run| run.exported_block_artifact_count)
            .sum(),
        aggregate_exported_extent_artifact_count: ordered_runs
            .iter()
            .map(|run| run.exported_extent_artifact_count)
            .sum(),
        aggregate_exported_full_file_artifact_count: ordered_runs
            .iter()
            .map(|run| run.exported_full_file_artifact_count)
            .sum(),
    }];

    let mut top_verified = ordered_runs.clone();
    top_verified.sort_by(|a, b| {
        b.verified_block_count
            .cmp(&a.verified_block_count)
            .then_with(|| a.archive_id.cmp(&b.archive_id))
    });
    let mut top_salvageable = ordered_runs.clone();
    top_salvageable.sort_by(|a, b| {
        b.salvageable_file_count
            .cmp(&a.salvageable_file_count)
            .then_with(|| a.archive_id.cmp(&b.archive_id))
    });
    let mut top_full_exports = ordered_runs.clone();
    top_full_exports.sort_by(|a, b| {
        b.exported_full_file_artifact_count
            .cmp(&a.exported_full_file_artifact_count)
            .then_with(|| a.archive_id.cmp(&b.archive_id))
    });

    let no_verified: Vec<RankingEntry> = ordered_runs
        .iter()
        .copied()
        .filter(|run| run.verified_block_count == 0)
        .map(to_ranking_entry)
        .collect();

    let evidence_rankings = EvidenceRankings {
        top_runs_by_verified_blocks: top_verified.into_iter().map(to_ranking_entry).collect(),
        top_runs_by_salvageable_files: top_salvageable.into_iter().map(to_ranking_entry).collect(),
        top_runs_by_exported_full_files: top_full_exports
            .into_iter()
            .map(to_ranking_entry)
            .collect(),
        bottom_runs_with_no_verified_evidence: no_verified,
    };

    let mut profile_map: BTreeMap<String, Vec<&RunMetadata>> = BTreeMap::new();
    for run in &ordered_runs {
        profile_map
            .entry(infer_profile_key(&run.archive_path))
            .or_default()
            .push(*run);
    }

    let mut profile_groups = Vec::new();
    for (profile_key, profile_runs) in profile_map {
        let mut outcome_counts: BTreeMap<&'static str, usize> =
            OUTCOME_ORDER.iter().map(|o| (*o, 0usize)).collect();
        for run in &profile_runs {
            let outcome = classify_outcome(run);
            *outcome_counts.entry(outcome).or_insert(0) += 1;
        }

        profile_groups.push(ProfileGroup {
            profile_key,
            run_count: profile_runs.len(),
            outcome_counts,
            aggregate_verified_block_count: profile_runs
                .iter()
                .map(|run| run.verified_block_count)
                .sum(),
            aggregate_salvageable_file_count: profile_runs
                .iter()
                .map(|run| run.salvageable_file_count)
                .sum(),
        });
    }

    let analysis = ExperimentAnalysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        experiment_id: experiment_id.to_string(),
        verification_label: VERIFICATION_LABEL,
        run_count: ordered_runs.len(),
        grouping_strategy: GroupingStrategy {
            outcome_order: OUTCOME_ORDER.to_vec(),
            profile_inference_priority: vec![
                "explicit_profile_metadata",
                "filename_path_derived_profile",
                "UNKNOWN_PROFILE_fallback",
            ],
            ranking_tie_breaker: "archive_id_ascending",
        },
        outcome_groups,
        export_mode_groups,
        evidence_rankings,
        profile_groups,
        notes: vec![
            "Compact deterministic grouped analysis over experiment metadata only".to_string(),
            "Research-only outputs; not canonical extraction semantics".to_string(),
        ],
    };

    fs::write(
        experiment_dir.join("analysis.json"),
        serde_json::to_string_pretty(&analysis)?,
    )
    .with_context(|| format!("write {}", experiment_dir.join("analysis.json").display()))?;

    let mut markdown = String::new();
    markdown.push_str("# Salvage Experiment Grouped Analysis\n\n");
    markdown.push_str(&format!("- Experiment ID: `{}`\n", analysis.experiment_id));
    markdown.push_str(&format!("- Run count: `{}`\n", analysis.run_count));
    markdown.push_str(&format!(
        "- Verification label: `{}`\n",
        analysis.verification_label
    ));
    markdown.push_str(&format!(
        "- Fragment export enabled: `{}`\n\n",
        if export_fragments_enabled {
            "yes"
        } else {
            "no"
        }
    ));
    markdown.push_str(
        "All grouped analysis outputs are research-only (`UNVERIFIED_RESEARCH_OUTPUT`).\n\n",
    );
    markdown.push_str("## Grouped outcome counts\n\n");
    for group in &analysis.outcome_groups {
        markdown.push_str(&format!(
            "- {}: {} runs (verified_blocks={}, salvageable_files={}, exported b/e/f={}/{}/{})\n",
            group.outcome,
            group.run_count,
            group.aggregate_verified_block_count,
            group.aggregate_salvageable_file_count,
            group.aggregate_exported_block_artifact_count,
            group.aggregate_exported_extent_artifact_count,
            group.aggregate_exported_full_file_artifact_count
        ));
    }
    markdown.push_str("\n## Grouped profile counts\n\n");
    for group in &analysis.profile_groups {
        markdown.push_str(&format!(
            "- `{}`: {} runs, verified_blocks={}, salvageable_files={}\n",
            group.profile_key,
            group.run_count,
            group.aggregate_verified_block_count,
            group.aggregate_salvageable_file_count
        ));
    }
    markdown.push_str("\n## Top evidence\n\n");
    for entry in analysis
        .evidence_rankings
        .top_runs_by_verified_blocks
        .iter()
        .take(5)
    {
        markdown.push_str(&format!(
            "- `{}` (`{}`): verified_blocks={}, salvageable_files={}, outcome={}\n",
            entry.archive_id,
            entry.archive_path,
            entry.verified_block_count,
            entry.salvageable_file_count,
            entry.outcome
        ));
    }
    markdown.push_str("\n## No verified evidence\n\n");
    for entry in &analysis
        .evidence_rankings
        .bottom_runs_with_no_verified_evidence
    {
        markdown.push_str(&format!(
            "- `{}` (`{}`): outcome={}\n",
            entry.archive_id, entry.archive_path, entry.outcome
        ));
    }
    markdown.push_str("\n## Orphan-only evidence\n\n");
    for entry in analysis
        .evidence_rankings
        .top_runs_by_verified_blocks
        .iter()
        .filter(|entry| entry.outcome == "ORPHAN_EVIDENCE_ONLY")
        .take(5)
    {
        markdown.push_str(&format!(
            "- `{}` (`{}`): verified_blocks={}\n",
            entry.archive_id, entry.archive_path, entry.verified_block_count
        ));
    }

    fs::write(experiment_dir.join("analysis.md"), markdown)
        .with_context(|| format!("write {}", experiment_dir.join("analysis.md").display()))?;

    Ok(())
}

fn generate_summary_files(
    experiment_dir: &Path,
    experiment_id: &str,
    export_fragments_enabled: bool,
    mut runs: Vec<RunMetadata>,
) -> Result<()> {
    runs.sort_by(|a, b| a.archive_id.cmp(&b.archive_id));
    let rows: Vec<RunSummaryRow> = runs
        .iter()
        .map(|run| RunSummaryRow {
            archive_id: run.archive_id.clone(),
            archive_path: run.archive_path.clone(),
            archive_fingerprint: run.archive_fingerprint.clone(),
            verified_block_count: run.verified_block_count,
            salvageable_file_count: run.salvageable_file_count,
            unsalvageable_file_count: run.unsalvageable_file_count,
            unmappable_file_count: run.unmappable_file_count,
            exported_block_artifact_count: run.exported_block_artifact_count,
            exported_extent_artifact_count: run.exported_extent_artifact_count,
            exported_full_file_artifact_count: run.exported_full_file_artifact_count,
            outcome: classify_outcome(run),
        })
        .collect();

    let summary = ExperimentSummary {
        schema_version: SUMMARY_SCHEMA_VERSION,
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        experiment_id: experiment_id.to_string(),
        verification_label: VERIFICATION_LABEL,
        run_count: rows.len(),
        archives_with_verified_blocks: rows.iter().filter(|r| r.verified_block_count > 0).count(),
        archives_with_salvageable_files: rows
            .iter()
            .filter(|r| r.salvageable_file_count > 0)
            .count(),
        archives_with_only_orphan_evidence: rows
            .iter()
            .filter(|r| r.outcome == "ORPHAN_EVIDENCE_ONLY")
            .count(),
        archives_with_no_verified_evidence: rows
            .iter()
            .filter(|r| r.outcome == "NO_VERIFIED_EVIDENCE")
            .count(),
        total_verified_blocks: rows.iter().map(|r| r.verified_block_count).sum(),
        total_exported_block_artifacts: rows.iter().map(|r| r.exported_block_artifact_count).sum(),
        total_exported_extent_artifacts: rows
            .iter()
            .map(|r| r.exported_extent_artifact_count)
            .sum(),
        total_exported_full_file_artifacts: rows
            .iter()
            .map(|r| r.exported_full_file_artifact_count)
            .sum(),
        total_salvageable_files: rows.iter().map(|r| r.salvageable_file_count).sum(),
        total_unsalvageable_files: rows.iter().map(|r| r.unsalvageable_file_count).sum(),
        total_unmappable_files: rows.iter().map(|r| r.unmappable_file_count).sum(),
        runs: rows,
    };

    fs::write(
        experiment_dir.join("summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )
    .with_context(|| format!("write {}", experiment_dir.join("summary.json").display()))?;

    let mut outcome_counts = BTreeMap::new();
    for row in &summary.runs {
        *outcome_counts.entry(row.outcome).or_insert(0usize) += 1;
    }

    let mut markdown = String::new();
    markdown.push_str("# Salvage Experiment Summary\n\n");
    markdown.push_str(&format!("- Experiment ID: `{}`\n", summary.experiment_id));
    markdown.push_str(&format!(
        "- Verification label: `{}`\n",
        summary.verification_label
    ));
    markdown.push_str(&format!("- Run count: `{}`\n", summary.run_count));
    markdown.push_str(&format!(
        "- Fragment export enabled: `{}`\n\n",
        if export_fragments_enabled {
            "yes"
        } else {
            "no"
        }
    ));
    markdown.push_str(
        "All outputs are research-only and non-canonical (`UNVERIFIED_RESEARCH_OUTPUT`).\n\n",
    );
    markdown.push_str("## Aggregate totals\n\n");
    markdown.push_str(&format!(
        "- verified blocks: {}\n",
        summary.total_verified_blocks
    ));
    markdown.push_str(&format!(
        "- salvageable / unsalvageable / unmappable files: {} / {} / {}\n",
        summary.total_salvageable_files,
        summary.total_unsalvageable_files,
        summary.total_unmappable_files
    ));
    markdown.push_str(&format!(
        "- exported artifacts (block / extent / full-file): {} / {} / {}\n\n",
        summary.total_exported_block_artifacts,
        summary.total_exported_extent_artifacts,
        summary.total_exported_full_file_artifacts
    ));
    markdown.push_str("## Outcome counts\n\n");
    for outcome in [
        "NO_VERIFIED_EVIDENCE",
        "ORPHAN_EVIDENCE_ONLY",
        "PARTIAL_FILE_SALVAGE",
        "FULL_FILE_SALVAGE_AVAILABLE",
    ] {
        markdown.push_str(&format!(
            "- {}: {}\n",
            outcome,
            outcome_counts.get(outcome).copied().unwrap_or(0)
        ));
    }
    markdown.push_str("\n## Runs\n\n");
    markdown.push_str("| archive_id | archive_path | outcome | verified_blocks | salvageable | unsalvageable | unmappable | exported (b/e/f) |\n");
    markdown.push_str("|---|---|---|---:|---:|---:|---:|---:|\n");
    for row in &summary.runs {
        markdown.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} | {} | {} | {}/{}/{} |\n",
            row.archive_id,
            row.archive_path,
            row.outcome,
            row.verified_block_count,
            row.salvageable_file_count,
            row.unsalvageable_file_count,
            row.unmappable_file_count,
            row.exported_block_artifact_count,
            row.exported_extent_artifact_count,
            row.exported_full_file_artifact_count
        ));
    }

    fs::write(experiment_dir.join("summary.md"), markdown)
        .with_context(|| format!("write {}", experiment_dir.join("summary.md").display()))?;

    generate_analysis_files(
        experiment_dir,
        experiment_id,
        export_fragments_enabled,
        &runs,
    )
}

fn load_runs_from_experiment(
    manifest: &ExperimentManifestInput,
    runs_root: &Path,
) -> Result<Vec<RunMetadata>> {
    let mut runs = Vec::new();

    for archive_id in &manifest.archive_list {
        let run_dir = runs_root.join(archive_id);
        let run_metadata_path = run_dir.join("run_metadata.json");
        let plan_path = run_dir.join("salvage_plan.json");
        let mut metadata: RunMetadataInput = serde_json::from_slice(
            &fs::read(&run_metadata_path)
                .with_context(|| format!("read {}", run_metadata_path.display()))?,
        )
        .with_context(|| format!("parse {}", run_metadata_path.display()))?;

        let (plan_block_count, plan_extent_count, plan_full_count) = if metadata
            .exported_block_artifact_count
            .is_none()
            || metadata.exported_extent_artifact_count.is_none()
            || metadata.exported_full_file_artifact_count.is_none()
        {
            let plan: Value = serde_json::from_slice(
                &fs::read(&plan_path).with_context(|| format!("read {}", plan_path.display()))?,
            )
            .with_context(|| format!("parse {}", plan_path.display()))?;
            exported_artifact_counts(&plan)
        } else {
            (0, 0, 0)
        };

        let exported_block_artifact_count = metadata
            .exported_block_artifact_count
            .take()
            .unwrap_or(plan_block_count);
        let exported_extent_artifact_count = metadata
            .exported_extent_artifact_count
            .take()
            .unwrap_or(plan_extent_count);
        let exported_full_file_artifact_count = metadata
            .exported_full_file_artifact_count
            .take()
            .unwrap_or(plan_full_count);

        runs.push(RunMetadata {
            archive_id: archive_id.clone(),
            archive_path: metadata.archive_path,
            archive_fingerprint: metadata.archive_fingerprint,
            salvage_plan_summary: PlanSummary {
                salvageable_files: metadata.salvageable_file_count,
                unsalvageable_files: metadata.unsalvageable_file_count,
                unmappable_files: metadata.unmappable_file_count,
            },
            verified_block_count: metadata.verified_block_count,
            exported_artifact_count: exported_block_artifact_count
                + exported_extent_artifact_count
                + exported_full_file_artifact_count,
            exported_block_artifact_count,
            exported_extent_artifact_count,
            exported_full_file_artifact_count,
            salvageable_file_count: metadata.salvageable_file_count,
            unsalvageable_file_count: metadata.unsalvageable_file_count,
            unmappable_file_count: metadata.unmappable_file_count,
        });
    }

    Ok(runs)
}

fn resolve_pack_bin() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CRUSHR_PACK_BIN") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        bail!(
            "CRUSHR_PACK_BIN points to missing/non-file path: {}",
            candidate.display()
        );
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let candidate = exe_dir.join(format!("crushr-pack{}", std::env::consts::EXE_SUFFIX));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if let Ok(path) = std::env::var("CARGO_BIN_EXE_crushr-pack") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    bail!("unable to resolve crushr-pack binary")
}

fn comparison_scenarios() -> Vec<ComparisonScenario> {
    let datasets = ["smallfiles", "mixed", "largefiles"];
    let targets = [
        ("header", "byte_flip"),
        ("index", "byte_flip"),
        ("payload", "byte_flip"),
        ("tail", "truncate"),
    ];
    let magnitudes = ["small", "medium"];

    let mut scenarios = Vec::new();
    let mut seed = 100u64;
    for dataset in datasets {
        for (target, model) in targets {
            for magnitude in magnitudes {
                let break_redundant_map =
                    dataset == "mixed" && target == "index" && magnitude == "medium";
                scenarios.push(ComparisonScenario {
                    scenario_id: format!("{}_{}_{}_{}", dataset, target, model, magnitude),
                    dataset,
                    corruption_model: model,
                    corruption_target: target,
                    magnitude,
                    seed,
                    break_redundant_map,
                });
                seed += 1;
            }
        }
    }
    scenarios
}

fn write_dataset_fixture(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input).with_context(|| format!("create {}", input.display()))?;

    match dataset {
        "smallfiles" => {
            fs::write(input.join("tiny.txt"), b"small-dataset-payload")?;
        }
        "mixed" => {
            fs::write(
                input.join("mixed.bin"),
                (0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
            )?;
        }
        "largefiles" => {
            fs::write(input.join("large.dat"), vec![13u8; 8192])?;
        }
        _ => bail!("unsupported dataset {dataset}"),
    }

    Ok(())
}

fn build_archive_with_pack(pack_bin: &Path, input: &Path, output: &Path) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn build_archive_with_pack_experimental(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--experimental-self-describing-extents")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn build_archive_with_pack_file_identity(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--experimental-file-identity-extents")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn build_archive_with_pack_format05(pack_bin: &Path, input: &Path, output: &Path) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--experimental-self-identifying-blocks")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn remove_ledger_for_old_style(archive_path: &Path) -> Result<()> {
    let bytes = fs::read(archive_path)?;
    let footer_offset = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let tail = parse_tail_frame(&bytes[blocks_end..])?;
    let mut rewritten = bytes[..blocks_end].to_vec();
    let tail_bytes = assemble_tail_frame(footer.blocks_end_offset, None, &tail.idx3_bytes, None)?;
    rewritten.extend_from_slice(&tail_bytes);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

fn corrupt_archive(archive_path: &Path, scenario: &ComparisonScenario) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    let len = bytes.len();
    let shift = (scenario.seed as usize) % 11;
    let mag = if scenario.magnitude == "small" {
        1usize
    } else {
        8usize
    };

    match scenario.corruption_target {
        "header" => {
            for (i, byte) in bytes.iter_mut().enumerate().take(mag.min(len)) {
                *byte ^= 0x11 + ((i + shift) % 7) as u8;
            }
        }
        "payload" => {
            let start = (len / 3).saturating_add(shift);
            for i in 0..mag {
                let idx = (start + i).min(len.saturating_sub(1));
                bytes[idx] ^= 0x33 + (i % 5) as u8;
            }
        }
        "tail" => {
            let cut = if scenario.magnitude == "small" {
                32
            } else {
                128
            };
            let new_len = len.saturating_sub(cut);
            bytes.truncate(new_len.max(16));
        }
        "index" => {
            let footer_offset = len.saturating_sub(FTR4_LEN);
            let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
            if scenario.dataset == "smallfiles" {
                let blocks_end = footer.blocks_end_offset as usize;
                let mut tail = parse_tail_frame(&bytes[blocks_end..])?;
                if tail.idx3_bytes.len() > 4 {
                    tail.idx3_bytes[4] ^= 0x7F;
                }
                for i in 0..mag {
                    let idx = 5 + ((i + shift) % 16);
                    if idx < tail.idx3_bytes.len() {
                        tail.idx3_bytes[idx] ^= 0x5A + (i % 3) as u8;
                    }
                }
                let mut rewritten = bytes[..blocks_end].to_vec();
                let tail_bytes = assemble_tail_frame(
                    footer.blocks_end_offset,
                    None,
                    &tail.idx3_bytes,
                    tail.ldg1.as_ref(),
                )?;
                rewritten.extend_from_slice(&tail_bytes);
                fs::write(archive_path, rewritten)?;
                return Ok(());
            }

            let index_offset = footer.index_offset as usize;
            if index_offset < footer_offset {
                bytes[index_offset] ^= 0x7F;
            }
            for i in 0..mag {
                let idx = index_offset + 4 + ((i + shift) % 16);
                if idx < footer_offset {
                    bytes[idx] ^= 0x5A + (i % 3) as u8;
                }
            }
        }
        _ => bail!(
            "unsupported corruption target {}",
            scenario.corruption_target
        ),
    }

    fs::write(archive_path, bytes)?;
    Ok(())
}

fn damage_redundant_map_ledger(archive_path: &Path) -> Result<()> {
    let bytes = fs::read(archive_path)?;
    let footer_offset = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let tail = parse_tail_frame(&bytes[blocks_end..])?;
    let Some(mut ledger) = tail.ldg1 else {
        return Ok(());
    };

    let mut value: Value = serde_json::from_slice(&ledger.json)?;
    if let Some(files) = value.get_mut("files").and_then(Value::as_array_mut) {
        if let Some(first) = files.first_mut() {
            if let Some(extents) = first.get_mut("extents").and_then(Value::as_array_mut) {
                if let Some(ext) = extents.first_mut() {
                    ext["block_id"] = Value::from(999999u64);
                }
            }
        }
    }
    ledger.json = serde_json::to_vec(&value)?;

    let mut rewritten = bytes[..blocks_end].to_vec();
    let tail_bytes = assemble_tail_frame(
        footer.blocks_end_offset,
        None,
        &tail.idx3_bytes,
        Some(&ledger),
    )?;
    rewritten.extend_from_slice(&tail_bytes);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

fn run_salvage_plan(salvage_bin: &Path, archive_path: &Path, plan_path: &Path) -> Result<Value> {
    let out = Command::new(salvage_bin)
        .arg(archive_path)
        .arg("--json-out")
        .arg(plan_path)
        .output()
        .with_context(|| format!("run {:?}", salvage_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-salvage failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(serde_json::from_slice(&fs::read(plan_path)?)?)
}

fn outcome_from_plan(plan: &Value) -> OutcomeMetrics {
    let verified_block_count = plan
        .get("block_candidates")
        .and_then(Value::as_array)
        .map_or(0, |rows| {
            rows.iter()
                .filter(|r| {
                    r.get("content_verification_status").and_then(Value::as_str)
                        == Some("content_verified")
                })
                .count() as u64
        });
    let salvageable_file_count = plan["summary"]["salvageable_files"].as_u64().unwrap_or(0);
    let exported_full_file_count =
        plan.get("file_plans")
            .and_then(Value::as_array)
            .map_or(0, |rows| {
                rows.iter()
                    .filter(|r| {
                        r.get("salvage_status").and_then(Value::as_str) == Some("fully_salvageable")
                    })
                    .count() as u64
            });
    let outcome = if exported_full_file_count > 0 {
        "FULL_FILE_SALVAGE_AVAILABLE"
    } else if salvageable_file_count > 0 {
        "PARTIAL_FILE_SALVAGE"
    } else if verified_block_count > 0 {
        "ORPHAN_EVIDENCE_ONLY"
    } else {
        "NO_VERIFIED_EVIDENCE"
    };

    OutcomeMetrics {
        outcome: outcome.to_string(),
        verified_block_count,
        salvageable_file_count,
        exported_full_file_count,
    }
}

fn outcome_rank(value: &str) -> i32 {
    match value {
        "NO_VERIFIED_EVIDENCE" => 0,
        "ORPHAN_EVIDENCE_ONLY" => 1,
        "PARTIAL_FILE_SALVAGE" => 2,
        "FULL_FILE_SALVAGE_AVAILABLE" => 3,
        _ => -1,
    }
}

fn classify_improvement(old: &str, new: &str) -> String {
    match (old, new) {
        ("ORPHAN_EVIDENCE_ONLY", "PARTIAL_FILE_SALVAGE") => {
            "IMPROVED_ORPHAN_TO_PARTIAL".to_string()
        }
        ("ORPHAN_EVIDENCE_ONLY", "FULL_FILE_SALVAGE_AVAILABLE") => {
            "IMPROVED_ORPHAN_TO_FULL".to_string()
        }
        ("NO_VERIFIED_EVIDENCE", "PARTIAL_FILE_SALVAGE") => "IMPROVED_NONE_TO_PARTIAL".to_string(),
        ("NO_VERIFIED_EVIDENCE", "FULL_FILE_SALVAGE_AVAILABLE") => {
            "IMPROVED_NONE_TO_FULL".to_string()
        }
        _ if outcome_rank(new) > outcome_rank(old) => "IMPROVED_OTHER".to_string(),
        _ if outcome_rank(new) < outcome_rank(old) => "DEGRADED".to_string(),
        _ => "UNCHANGED".to_string(),
    }
}

fn build_groups(
    rows: &[ScenarioRow],
    key_fn: impl Fn(&ScenarioRow) -> &str,
) -> Vec<ComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&ScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| ComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            new_outcome_counts: count_outcomes(values.iter().map(|r| r.new_outcome.as_str())),
            verified_block_delta: values
                .iter()
                .map(|r| r.new_verified_block_count as i64 - r.old_verified_block_count as i64)
                .sum(),
            salvageable_file_delta: values
                .iter()
                .map(|r| r.new_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
                .sum(),
            exported_full_file_delta: values
                .iter()
                .map(|r| {
                    r.new_exported_full_file_count as i64 - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

fn count_outcomes<'a>(items: impl Iterator<Item = &'a str>) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for item in items {
        *map.entry(item.to_string()).or_insert(0) += 1;
    }
    map
}

fn render_comparison_markdown(summary: &ComparisonSummary) -> String {
    let mut md = String::new();
    md.push_str(
        "# Redundant Map Salvage Comparison

",
    );
    md.push_str("Purpose: targeted deterministic before/after comparison of old-style (no redundant map) and new-style (redundant map) archives.

");
    md.push_str("Strict boundary reminder: research-only salvage outputs; canonical extraction semantics remain unchanged.

");
    md.push_str(&format!(
        "- Scenario count: `{}`
",
        summary.scenario_count
    ));
    md.push_str(&format!(
        "- Old archive count: `{}`
",
        summary.old_archive_count
    ));
    md.push_str(&format!(
        "- New archive count: `{}`
",
        summary.new_archive_count
    ));
    md.push_str(&format!(
        "- Orphan→salvage improvements: `{}`
",
        summary.orphan_to_salvage_improvements
    ));
    md.push_str(&format!(
        "- No-evidence→salvage improvements: `{}`

",
        summary.no_evidence_to_salvage_improvements
    ));

    md.push_str(
        "## Outcome totals

",
    );
    md.push_str(
        "- Old:
",
    );
    for (k, v) in &summary.old_outcome_counts {
        md.push_str(&format!(
            "  - {}: {}
",
            k, v
        ));
    }
    md.push_str(
        "- New:
",
    );
    for (k, v) in &summary.new_outcome_counts {
        md.push_str(&format!(
            "  - {}: {}
",
            k, v
        ));
    }

    md.push_str(
        "
## Grouped by dataset

",
    );
    for g in &summary.by_dataset {
        md.push_str(&format!(
            "- `{}`: scenarios={}, Δverified={}, Δsalvageable={}
",
            g.key, g.scenario_count, g.verified_block_delta, g.salvageable_file_delta
        ));
    }

    md.push_str(
        "
## Grouped by corruption target

",
    );
    for g in &summary.by_corruption_target {
        md.push_str(&format!(
            "- `{}`: scenarios={}, Δverified={}, Δsalvageable={}
",
            g.key, g.scenario_count, g.verified_block_delta, g.salvageable_file_delta
        ));
    }

    md.push_str(
        "
## Most improved scenarios

",
    );
    for row in summary
        .per_scenario_rows
        .iter()
        .filter(|r| r.improvement_class.starts_with("IMPROVED"))
        .take(5)
    {
        md.push_str(&format!(
            "- `{}`: {} → {} ({})
",
            row.scenario_id, row.old_outcome, row.new_outcome, row.improvement_class
        ));
    }

    md.push_str(
        "
## No improvement / degraded scenarios

",
    );
    for row in summary
        .per_scenario_rows
        .iter()
        .filter(|r| r.improvement_class == "UNCHANGED" || r.improvement_class == "DEGRADED")
        .take(8)
    {
        md.push_str(&format!(
            "- `{}`: {} → {} ({})
",
            row.scenario_id, row.old_outcome, row.new_outcome, row.improvement_class
        ));
    }

    md
}

fn run_redundant_map_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "crushr-lab-comparison-{}-{}",
        std::process::id(),
        unique
    ));
    if temp.exists() {
        fs::remove_dir_all(&temp).with_context(|| format!("remove {}", temp.display()))?;
    }
    fs::create_dir_all(&temp).with_context(|| format!("create {}", temp.display()))?;
    let datasets_root = temp.join("datasets");
    let archives_root = temp.join("archives");
    fs::create_dir_all(&datasets_root)?;
    fs::create_dir_all(&archives_root)?;

    let pack_bin = resolve_pack_bin()?;
    let salvage_bin = resolve_salvage_bin()?;

    let mut rows = Vec::new();
    for scenario in comparison_scenarios() {
        let dataset_input = datasets_root.join(scenario.dataset);
        if !dataset_input.exists() {
            write_dataset_fixture(&datasets_root, scenario.dataset)?;
        }

        let old_archive = archives_root.join(format!("{}_old.crushr", scenario.scenario_id));
        let new_archive = archives_root.join(format!("{}_new.crushr", scenario.scenario_id));
        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &new_archive)?;
        remove_ledger_for_old_style(&old_archive)?;

        if scenario.break_redundant_map {
            damage_redundant_map_ledger(&new_archive)?;
        }
        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&new_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let new_plan = run_salvage_plan(
            &salvage_bin,
            &new_archive,
            &archives_root.join(format!("{}_new_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let new_metrics = outcome_from_plan(&new_plan);
        let improvement = classify_improvement(&old_metrics.outcome, &new_metrics.outcome);
        if verbose {
            eprintln!("scenario {} => {}", scenario.scenario_id, improvement);
        }

        rows.push(ScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            new_outcome: new_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            new_verified_block_count: new_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            new_salvageable_file_count: new_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            new_exported_full_file_count: new_metrics.exported_full_file_count,
            improvement_class: improvement,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));

    let summary = ComparisonSummary {
        schema_version: "crushr-lab-salvage-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_archive_count: rows.len(),
        new_archive_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        new_outcome_counts: count_outcomes(rows.iter().map(|r| r.new_outcome.as_str())),
        orphan_to_salvage_improvements: rows
            .iter()
            .filter(|r| {
                matches!(
                    r.improvement_class.as_str(),
                    "IMPROVED_ORPHAN_TO_PARTIAL" | "IMPROVED_ORPHAN_TO_FULL"
                )
            })
            .count() as u64,
        no_evidence_to_salvage_improvements: rows
            .iter()
            .filter(|r| {
                matches!(
                    r.improvement_class.as_str(),
                    "IMPROVED_NONE_TO_PARTIAL" | "IMPROVED_NONE_TO_FULL"
                )
            })
            .count() as u64,
        unchanged_outcome_count: rows
            .iter()
            .filter(|r| r.improvement_class == "UNCHANGED")
            .count() as u64,
        degraded_outcome_count: rows
            .iter()
            .filter(|r| r.improvement_class == "DEGRADED")
            .count() as u64,
        total_verified_block_delta: rows
            .iter()
            .map(|r| r.new_verified_block_count as i64 - r.old_verified_block_count as i64)
            .sum(),
        total_salvageable_file_delta: rows
            .iter()
            .map(|r| r.new_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
            .sum(),
        total_exported_full_file_delta: rows
            .iter()
            .map(|r| r.new_exported_full_file_count as i64 - r.old_exported_full_file_count as i64)
            .sum(),
        by_dataset: build_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_groups(&rows, |r| &r.corruption_target),
        by_corruption_model: build_groups(&rows, |r| &r.corruption_model),
        by_magnitude: build_groups(&rows, |r| &r.magnitude),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("comparison_summary.md"),
        render_comparison_markdown(&summary),
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn build_experimental_groups(
    rows: &[ExperimentalScenarioRow],
    key_fn: impl Fn(&ExperimentalScenarioRow) -> &str,
) -> Vec<ExperimentalComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&ExperimentalScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| ExperimentalComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            redundant_outcome_counts: count_outcomes(
                values.iter().map(|r| r.redundant_outcome.as_str()),
            ),
            experimental_outcome_counts: count_outcomes(
                values.iter().map(|r| r.experimental_outcome.as_str()),
            ),
            file_identity_outcome_counts: count_outcomes(
                values.iter().map(|r| r.file_identity_outcome.as_str()),
            ),
            verified_block_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_verified_block_count as i64 - r.old_verified_block_count as i64
                })
                .sum(),
            salvageable_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_salvageable_file_count as i64
                        - r.old_salvageable_file_count as i64
                })
                .sum(),
            exported_full_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_exported_full_file_count as i64
                        - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

fn build_format05_groups(
    rows: &[Format05ScenarioRow],
    key_fn: impl Fn(&Format05ScenarioRow) -> &str,
) -> Vec<Format05ComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&Format05ScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| Format05ComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            redundant_outcome_counts: count_outcomes(
                values.iter().map(|r| r.redundant_outcome.as_str()),
            ),
            experimental_outcome_counts: count_outcomes(
                values.iter().map(|r| r.experimental_outcome.as_str()),
            ),
            file_identity_outcome_counts: count_outcomes(
                values.iter().map(|r| r.file_identity_outcome.as_str()),
            ),
            format05_outcome_counts: count_outcomes(
                values.iter().map(|r| r.format05_outcome.as_str()),
            ),
            verified_block_delta_vs_old: values
                .iter()
                .map(|r| r.format05_verified_block_count as i64 - r.old_verified_block_count as i64)
                .sum(),
            salvageable_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.format05_salvageable_file_count as i64 - r.old_salvageable_file_count as i64
                })
                .sum(),
            exported_full_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.format05_exported_full_file_count as i64
                        - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

fn run_experimental_resilience_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let scenarios = comparison_scenarios();
    let pack_bin = resolve_pack_bin()?;
    let salvage_bin = resolve_salvage_bin()?;
    let temp = comparison_dir.join(".tmp_experimental");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;
    let archives_root = temp.join("archives");
    fs::create_dir_all(&archives_root)?;

    let mut rows = Vec::new();
    for scenario in scenarios {
        let dataset_input = temp.join("datasets").join(&scenario.scenario_id);
        write_dataset_fixture(&dataset_input, scenario.dataset)?;

        let old_archive = archives_root.join(format!("{}_old.crushr", scenario.scenario_id));
        let redundant_archive =
            archives_root.join(format!("{}_redundant.crushr", scenario.scenario_id));
        let experimental_archive =
            archives_root.join(format!("{}_experimental.crushr", scenario.scenario_id));
        let file_identity_archive =
            archives_root.join(format!("{}_file_identity.crushr", scenario.scenario_id));
        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &redundant_archive)?;
        build_archive_with_pack_experimental(&pack_bin, &dataset_input, &experimental_archive)?;
        build_archive_with_pack_file_identity(&pack_bin, &dataset_input, &file_identity_archive)?;
        remove_ledger_for_old_style(&old_archive)?;

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&experimental_archive, &scenario)?;
        corrupt_archive(&file_identity_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &archives_root.join(format!("{}_redundant_plan.json", scenario.scenario_id)),
        )?;
        let experimental_plan = run_salvage_plan(
            &salvage_bin,
            &experimental_archive,
            &archives_root.join(format!("{}_experimental_plan.json", scenario.scenario_id)),
        )?;
        let file_identity_plan = run_salvage_plan(
            &salvage_bin,
            &file_identity_archive,
            &archives_root.join(format!("{}_file_identity_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let experimental_metrics = outcome_from_plan(&experimental_plan);
        let file_identity_metrics = outcome_from_plan(&file_identity_plan);
        if verbose {
            eprintln!(
                "scenario {} => {} / {} / {} / {}",
                scenario.scenario_id,
                old_metrics.outcome,
                redundant_metrics.outcome,
                experimental_metrics.outcome,
                file_identity_metrics.outcome
            );
        }

        rows.push(ExperimentalScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            redundant_outcome: redundant_metrics.outcome,
            experimental_outcome: experimental_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            redundant_verified_block_count: redundant_metrics.verified_block_count,
            experimental_verified_block_count: experimental_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            redundant_salvageable_file_count: redundant_metrics.salvageable_file_count,
            experimental_salvageable_file_count: experimental_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            redundant_exported_full_file_count: redundant_metrics.exported_full_file_count,
            experimental_exported_full_file_count: experimental_metrics.exported_full_file_count,
            file_identity_outcome: file_identity_metrics.outcome,
            file_identity_verified_block_count: file_identity_metrics.verified_block_count,
            file_identity_salvageable_file_count: file_identity_metrics.salvageable_file_count,
            file_identity_exported_full_file_count: file_identity_metrics.exported_full_file_count,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let summary = ExperimentalComparisonSummary {
        schema_version: "crushr-lab-salvage-experimental-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        redundant_outcome_counts: count_outcomes(rows.iter().map(|r| r.redundant_outcome.as_str())),
        experimental_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.experimental_outcome.as_str()),
        ),
        file_identity_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.file_identity_outcome.as_str()),
        ),
        orphan_to_salvage_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                outcome_rank(&r.old_outcome) <= 1 && outcome_rank(&r.file_identity_outcome) >= 2
            })
            .count() as u64,
        orphan_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.file_identity_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        orphan_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.file_identity_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        orphan_to_salvage_improvements_vs_redundant: rows
            .iter()
            .filter(|r| {
                outcome_rank(&r.redundant_outcome) <= 1
                    && outcome_rank(&r.file_identity_outcome) >= 2
            })
            .count() as u64,
        no_evidence_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.file_identity_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        no_evidence_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.file_identity_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        total_verified_block_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_verified_block_count as i64 - r.old_verified_block_count as i64
            })
            .sum(),
        total_salvageable_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_salvageable_file_count as i64 - r.old_salvageable_file_count as i64
            })
            .sum(),
        total_exported_full_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_exported_full_file_count as i64
                    - r.old_exported_full_file_count as i64
            })
            .sum(),
        by_dataset: build_experimental_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_experimental_groups(&rows, |r| &r.corruption_target),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("format04_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("file_identity_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    let md = format!(
        "# Format-04 comparison\n\nScenarios: {}\n\n- orphan->salvage vs old: {}\n- orphan->salvage vs redundant: {}\n- orphan->partial vs old: {}\n- orphan->full vs old: {}\n- no-evidence->partial vs old: {}\n- no-evidence->full vs old: {}\n",
        summary.scenario_count,
        summary.orphan_to_salvage_improvements_vs_old,
        summary.orphan_to_salvage_improvements_vs_redundant,
        summary.orphan_to_partial_improvements_vs_old,
        summary.orphan_to_full_improvements_vs_old,
        summary.no_evidence_to_partial_improvements_vs_old,
        summary.no_evidence_to_full_improvements_vs_old
    );
    fs::write(
        comparison_dir.join("format04_comparison_summary.md"),
        md.clone(),
    )?;
    fs::write(
        comparison_dir.join("file_identity_comparison_summary.md"),
        md,
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn run_format05_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = comparison_dir.join(".tmp_format05");
    let datasets_root = temp.join("datasets");
    let archives_root = temp.join("archives");
    fs::create_dir_all(&datasets_root)?;
    fs::create_dir_all(&archives_root)?;

    let pack_bin = resolve_pack_bin()?;
    let salvage_bin = resolve_salvage_bin()?;

    let mut rows = Vec::new();
    for scenario in comparison_scenarios() {
        let dataset_input = datasets_root.join(scenario.dataset);
        if !dataset_input.exists() {
            write_dataset_fixture(&datasets_root, scenario.dataset)?;
        }

        let old_archive = archives_root.join(format!("{}_old.crushr", scenario.scenario_id));
        let redundant_archive =
            archives_root.join(format!("{}_redundant.crushr", scenario.scenario_id));
        let experimental_archive =
            archives_root.join(format!("{}_experimental.crushr", scenario.scenario_id));
        let file_identity_archive =
            archives_root.join(format!("{}_file_identity.crushr", scenario.scenario_id));
        let format05_archive =
            archives_root.join(format!("{}_format05.crushr", scenario.scenario_id));

        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &redundant_archive)?;
        build_archive_with_pack_experimental(&pack_bin, &dataset_input, &experimental_archive)?;
        build_archive_with_pack_file_identity(&pack_bin, &dataset_input, &file_identity_archive)?;
        build_archive_with_pack_format05(&pack_bin, &dataset_input, &format05_archive)?;

        remove_ledger_for_old_style(&old_archive)?;
        if scenario.break_redundant_map {
            damage_redundant_map_ledger(&redundant_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&experimental_archive, &scenario)?;
        corrupt_archive(&file_identity_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &archives_root.join(format!("{}_redundant_plan.json", scenario.scenario_id)),
        )?;
        let experimental_plan = run_salvage_plan(
            &salvage_bin,
            &experimental_archive,
            &archives_root.join(format!("{}_experimental_plan.json", scenario.scenario_id)),
        )?;
        let file_identity_plan = run_salvage_plan(
            &salvage_bin,
            &file_identity_archive,
            &archives_root.join(format!("{}_file_identity_plan.json", scenario.scenario_id)),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &archives_root.join(format!("{}_format05_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let experimental_metrics = outcome_from_plan(&experimental_plan);
        let file_identity_metrics = outcome_from_plan(&file_identity_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        if verbose {
            eprintln!(
                "scenario {} => format05 {}",
                scenario.scenario_id, format05_metrics.outcome
            );
        }

        rows.push(Format05ScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            redundant_outcome: redundant_metrics.outcome,
            experimental_outcome: experimental_metrics.outcome,
            file_identity_outcome: file_identity_metrics.outcome,
            format05_outcome: format05_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            redundant_verified_block_count: redundant_metrics.verified_block_count,
            experimental_verified_block_count: experimental_metrics.verified_block_count,
            file_identity_verified_block_count: file_identity_metrics.verified_block_count,
            format05_verified_block_count: format05_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            redundant_salvageable_file_count: redundant_metrics.salvageable_file_count,
            experimental_salvageable_file_count: experimental_metrics.salvageable_file_count,
            file_identity_salvageable_file_count: file_identity_metrics.salvageable_file_count,
            format05_salvageable_file_count: format05_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            redundant_exported_full_file_count: redundant_metrics.exported_full_file_count,
            experimental_exported_full_file_count: experimental_metrics.exported_full_file_count,
            file_identity_exported_full_file_count: file_identity_metrics.exported_full_file_count,
            format05_exported_full_file_count: format05_metrics.exported_full_file_count,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let summary = Format05ComparisonSummary {
        schema_version: "crushr-lab-salvage-format05-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        redundant_outcome_counts: count_outcomes(rows.iter().map(|r| r.redundant_outcome.as_str())),
        experimental_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.experimental_outcome.as_str()),
        ),
        file_identity_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.file_identity_outcome.as_str()),
        ),
        format05_outcome_counts: count_outcomes(rows.iter().map(|r| r.format05_outcome.as_str())),
        orphan_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.format05_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        orphan_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.format05_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        no_evidence_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.format05_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        no_evidence_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.format05_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        total_verified_block_delta_vs_old: rows
            .iter()
            .map(|r| r.format05_verified_block_count as i64 - r.old_verified_block_count as i64)
            .sum(),
        total_salvageable_file_delta_vs_old: rows
            .iter()
            .map(|r| r.format05_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
            .sum(),
        total_exported_full_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.format05_exported_full_file_count as i64 - r.old_exported_full_file_count as i64
            })
            .sum(),
        by_dataset: build_format05_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_format05_groups(&rows, |r| &r.corruption_target),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("format05_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    let md = format!(
        "# Format-05 comparison\n\nScenarios: {}\n\n- orphan->partial vs old: {}\n- orphan->full vs old: {}\n- no-evidence->partial vs old: {}\n- no-evidence->full vs old: {}\n- total verified block delta vs old: {}\n- total salvageable file delta vs old: {}\n- total exported full-file delta vs old: {}\n",
        summary.scenario_count,
        summary.orphan_to_partial_improvements_vs_old,
        summary.orphan_to_full_improvements_vs_old,
        summary.no_evidence_to_partial_improvements_vs_old,
        summary.no_evidence_to_full_improvements_vs_old,
        summary.total_verified_block_delta_vs_old,
        summary.total_salvageable_file_delta_vs_old,
        summary.total_exported_full_file_delta_vs_old,
    );
    fs::write(comparison_dir.join("format05_comparison_summary.md"), md)?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn run_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{now}")
}
fn run() -> Result<()> {
    let opts = parse_cli_options()?;
    if let Mode::Help = &opts.mode {
        println!("{USAGE}");
        return Ok(());
    }
    if let Mode::RunRedundantMapComparison { comparison_dir } = &opts.mode {
        return run_redundant_map_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunExperimentalResilienceComparison { comparison_dir } = &opts.mode {
        return run_experimental_resilience_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFileIdentityComparison { comparison_dir } = &opts.mode {
        return run_experimental_resilience_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat04Comparison { comparison_dir } = &opts.mode {
        return run_experimental_resilience_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat05Comparison { comparison_dir } = &opts.mode {
        return run_format05_comparison(comparison_dir, opts.verbose);
    }

    let (experiment_dir, experiment_id, export_fragments_enabled, runs) = match &opts.mode {
        Mode::RunExperiment {
            input_dir: _,
            experiment_dir,
        } => {
            fs::create_dir_all(experiment_dir)
                .with_context(|| format!("create {}", experiment_dir.display()))?;

            let archives = collect_archives(&opts)?;
            let runs_root = experiment_dir.join("runs");
            fs::create_dir_all(&runs_root)
                .with_context(|| format!("create {}", runs_root.display()))?;

            let mut archive_ids = Vec::new();
            let mut runs = Vec::new();

            for archive in &archives {
                if opts.verbose {
                    eprintln!("salvage: {}", archive.archive_path);
                }

                let run_dir = runs_root.join(&archive.archive_id);
                fs::create_dir_all(&run_dir)
                    .with_context(|| format!("create {}", run_dir.display()))?;

                let metadata = run_salvage(archive, &run_dir, &opts)?;
                fs::write(
                    run_dir.join("run_metadata.json"),
                    serde_json::to_string_pretty(&metadata)?,
                )
                .with_context(|| {
                    format!("write {}", run_dir.join("run_metadata.json").display())
                })?;
                archive_ids.push(archive.archive_id.clone());
                runs.push(metadata);
            }

            let experiment_id = {
                let mut hasher = blake3::Hasher::new();
                hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
                hasher.update(if opts.export_fragments {
                    b"export"
                } else {
                    b"plan"
                });
                for archive_id in &archive_ids {
                    hasher.update(archive_id.as_bytes());
                    hasher.update(b"\n");
                }
                to_hex(&hasher.finalize().as_bytes()[..10])
            };

            let manifest = ExperimentManifest {
                experiment_id: experiment_id.clone(),
                tool_version: env!("CARGO_PKG_VERSION"),
                schema_version: EXPERIMENT_SCHEMA_VERSION,
                run_count: archive_ids.len(),
                run_timestamp: run_timestamp(),
                verification_label: VERIFICATION_LABEL,
                export_fragments_enabled: opts.export_fragments,
                archive_list: archive_ids,
            };

            fs::write(
                experiment_dir.join("experiment_manifest.json"),
                serde_json::to_string_pretty(&manifest)?,
            )
            .with_context(|| {
                format!(
                    "write {}",
                    experiment_dir.join("experiment_manifest.json").display()
                )
            })?;

            (
                experiment_dir.clone(),
                experiment_id,
                opts.export_fragments,
                runs,
            )
        }
        Mode::Resummarize { experiment_dir } => {
            let manifest_path = experiment_dir.join("experiment_manifest.json");
            let manifest: ExperimentManifestInput = serde_json::from_slice(
                &fs::read(&manifest_path)
                    .with_context(|| format!("read {}", manifest_path.display()))?,
            )
            .with_context(|| format!("parse {}", manifest_path.display()))?;
            if manifest.verification_label != VERIFICATION_LABEL {
                bail!(
                    "unsupported verification label in {}: {}",
                    manifest_path.display(),
                    manifest.verification_label
                );
            }
            let runs = load_runs_from_experiment(&manifest, &experiment_dir.join("runs"))?;
            (
                experiment_dir.clone(),
                manifest.experiment_id,
                manifest.export_fragments_enabled,
                runs,
            )
        }
        Mode::Help => bail!("internal error: help mode in summary pipeline"),
        Mode::RunRedundantMapComparison { .. } => {
            bail!("internal error: comparison mode in summary pipeline")
        }
        Mode::RunExperimentalResilienceComparison { .. }
        | Mode::RunFileIdentityComparison { .. }
        | Mode::RunFormat04Comparison { .. }
        | Mode::RunFormat05Comparison { .. } => {
            bail!("internal error: comparison mode in summary pipeline")
        }
    };
    generate_summary_files(
        &experiment_dir,
        &experiment_id,
        export_fragments_enabled,
        runs,
    )?;

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}
