use super::*;

pub(super) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(super) fn archive_id(archive_path: &str, archive_fingerprint: &str) -> String {
    let digest = blake3::hash(format!("{archive_path}\n{archive_fingerprint}").as_bytes());
    format!(
        "{}-{}",
        sanitize_component(archive_path),
        to_hex(&digest.as_bytes()[..8])
    )
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

pub(super) fn collect_archives(opts: &CliOptions) -> Result<Vec<ArchiveRun>> {
    let input_dir = match &opts.mode {
        Mode::RunExperiment { input_dir, .. } => input_dir,
        Mode::Help
        | Mode::Resummarize { .. }
        | Mode::RunRedundantMapComparison { .. }
        | Mode::RunExperimentalResilienceComparison { .. }
        | Mode::RunFileIdentityComparison { .. }
        | Mode::RunFormat04Comparison { .. }
        | Mode::RunFormat05Comparison { .. }
        | Mode::RunFormat06Comparison { .. }
        | Mode::RunFormat07Comparison { .. }
        | Mode::RunFormat08PlacementComparison { .. }
        | Mode::RunFormat09Comparison { .. }
        | Mode::RunFormat10PruningComparison { .. }
        | Mode::RunFormat11ExtentIdentityComparison { .. }
        | Mode::RunFormat12InlinePathComparison { .. }
        | Mode::RunFormat12StressComparison { .. }
        | Mode::RunFormat13Comparison { .. }
        | Mode::RunFormat13StressComparison { .. }
        | Mode::RunFormat14aDictionaryResilienceComparison { .. }
        | Mode::RunFormat14aDictionaryResilienceStressComparison { .. }
        | Mode::RunFormat15Comparison { .. }
        | Mode::RunFormat15StressComparison { .. } => {
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

pub(super) fn is_archive_bytes(bytes: &[u8]) -> bool {
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

pub(super) fn resolve_salvage_bin() -> Result<PathBuf> {
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
            let _ = build_salvage_bin_with_cargo(&candidate);
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

pub(super) fn build_salvage_bin_with_cargo(salvage_candidate: &Path) -> Result<()> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo_bin);
    cmd.args(["build", "-p", "crushr", "--bin", "crushr-salvage"]);
    if let Some(target_dir) = salvage_candidate.parent().and_then(Path::parent) {
        cmd.arg("--target-dir").arg(target_dir);
    }
    let out = cmd.output().with_context(|| format!("run {cargo_bin}"))?;
    if out.status.success() {
        Ok(())
    } else {
        bail!(
            "failed to build crushr-salvage with cargo\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }
}

pub(super) fn exported_artifact_counts(plan: &Value) -> (usize, usize, usize) {
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

pub(super) fn run_salvage(
    archive: &ArchiveRun,
    run_dir: &Path,
    opts: &CliOptions,
) -> Result<RunMetadata> {
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

pub(super) fn classify_outcome(run: &RunMetadata) -> &'static str {
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

pub(super) fn infer_profile_key(path: &str) -> String {
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

pub(super) fn to_ranking_entry(run: &RunMetadata) -> RankingEntry {
    RankingEntry {
        archive_id: run.archive_id.clone(),
        archive_path: run.archive_path.clone(),
        verified_block_count: run.verified_block_count,
        salvageable_file_count: run.salvageable_file_count,
        exported_full_file_artifact_count: run.exported_full_file_artifact_count,
        outcome: classify_outcome(run),
    }
}

pub(super) fn generate_analysis_files(
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

pub(super) fn generate_summary_files(
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

pub(super) fn load_runs_from_experiment(
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

pub(super) fn resolve_pack_bin() -> Result<PathBuf> {
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
            let _ = build_pack_bin_with_cargo(&candidate);
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

pub(super) fn build_pack_bin_with_cargo(pack_candidate: &Path) -> Result<()> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo_bin);
    cmd.args(["build", "-p", "crushr", "--bin", "crushr-pack"]);
    if let Some(target_dir) = pack_candidate.parent().and_then(Path::parent) {
        cmd.arg("--target-dir").arg(target_dir);
    }
    let out = cmd.output().with_context(|| format!("run {cargo_bin}"))?;
    if out.status.success() {
        Ok(())
    } else {
        bail!(
            "failed to build crushr-pack with cargo\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }
}
