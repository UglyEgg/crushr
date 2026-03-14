use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: crushr-lab-salvage <input_dir> --output <experiment_dir> [--export-fragments] [--limit <N>] [--verbose]\n       crushr-lab-salvage --resummarize <experiment_dir>";
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
    RunExperiment {
        input_dir: PathBuf,
        experiment_dir: PathBuf,
    },
    Resummarize {
        experiment_dir: PathBuf,
    },
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

fn parse_cli_options() -> Result<CliOptions> {
    let mut input_dir = None;
    let mut output_dir = None;
    let mut resummarize_dir = None;
    let mut export_fragments = false;
    let mut limit = None;
    let mut verbose = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
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
        Mode::Resummarize { .. } => bail!("internal error: collect_archives in resummarize mode"),
    };
    let mut archives = Vec::new();
    for entry in fs::read_dir(input_dir).with_context(|| format!("read {}", input_dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|v| v.to_str()) != Some("crushr") {
            continue;
        }

        let rel = path
            .strip_prefix(input_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
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

    let salvage_bin =
        std::env::var("CRUSHR_SALVAGE_BIN").unwrap_or_else(|_| "crushr-salvage".to_string());
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
