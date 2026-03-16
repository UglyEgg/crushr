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
       crushr-lab-salvage run-format05-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format06-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format07-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format08-placement-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format09-comparison --output <comparison_dir> [--verbose]
       crushr-lab-salvage run-format10-pruning-comparison --output <comparison_dir> [--verbose]";
const VERIFICATION_LABEL: &str = "UNVERIFIED_RESEARCH_OUTPUT";
const EXPERIMENT_SCHEMA_VERSION: &str = "crushr-lab-salvage-experiment.v1";
const SUMMARY_SCHEMA_VERSION: &str = "crushr-lab-salvage-summary.v1";
const ANALYSIS_SCHEMA_VERSION: &str = "crushr-lab-salvage-analysis.v1";
const FORMAT05_PACK_FLAG: &str = "--experimental-self-identifying-blocks";
const FORMAT06_PACK_FLAG: &str = "--experimental-file-manifest-checkpoints";
const FORMAT08_STRATEGY_FIXED: &str = "fixed_spread";
const FORMAT08_STRATEGY_HASH: &str = "hash_spread";
const FORMAT08_STRATEGY_GOLDEN: &str = "golden_spread";
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
    RunFormat06Comparison {
        comparison_dir: PathBuf,
    },
    RunFormat07Comparison {
        comparison_dir: PathBuf,
    },
    RunFormat08PlacementComparison {
        comparison_dir: PathBuf,
    },
    RunFormat09Comparison {
        comparison_dir: PathBuf,
    },
    RunFormat10PruningComparison {
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

// Internal responsibility split for safer lab workflow edits.
// cli => command parsing/dispatch, runner => archive scanning + salvage execution + summaries,
// comparison => scenario generation/corruption/comparison reporting.
#[path = "crushr_lab_salvage/cli.rs"]
mod cli;
#[path = "crushr_lab_salvage/comparison.rs"]
mod comparison;
#[path = "crushr_lab_salvage/runner.rs"]
mod runner;

use cli::parse_cli_options;
use comparison::{
    run_experimental_resilience_comparison, run_format05_comparison, run_format06_comparison,
    run_format07_comparison, run_format08_placement_comparison, run_format09_comparison,
    run_format10_pruning_comparison, run_redundant_map_comparison,
};
use runner::{
    collect_archives, generate_summary_files, load_runs_from_experiment, run_salvage, to_hex,
};

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
    if let Mode::RunFormat06Comparison { comparison_dir } = &opts.mode {
        return run_format06_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat07Comparison { comparison_dir } = &opts.mode {
        return run_format07_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat08PlacementComparison { comparison_dir } = &opts.mode {
        return run_format08_placement_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat09Comparison { comparison_dir } = &opts.mode {
        return run_format09_comparison(comparison_dir, opts.verbose);
    }
    if let Mode::RunFormat10PruningComparison { comparison_dir } = &opts.mode {
        return run_format10_pruning_comparison(comparison_dir, opts.verbose);
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
        | Mode::RunFormat05Comparison { .. }
        | Mode::RunFormat06Comparison { .. }
        | Mode::RunFormat07Comparison { .. }
        | Mode::RunFormat08PlacementComparison { .. }
        | Mode::RunFormat09Comparison { .. }
        | Mode::RunFormat10PruningComparison { .. } => {
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
