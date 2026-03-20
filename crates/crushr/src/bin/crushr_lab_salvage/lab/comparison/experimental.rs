// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::common::{
    build_archive_with_pack, build_archive_with_pack_experimental,
    build_archive_with_pack_file_identity, build_archive_with_pack_format05, comparison_scenarios,
    corrupt_archive, count_outcomes, damage_redundant_map_ledger, outcome_from_plan, outcome_rank,
    remove_ledger_for_old_style, run_salvage_plan, write_dataset_fixture,
};
use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};

pub(crate) fn build_experimental_groups(
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

pub(crate) fn build_format05_groups(
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

pub(crate) fn run_experimental_resilience_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
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
        tool_version: crushr::product_version(),
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

pub(crate) fn run_format05_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
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
        tool_version: crushr::product_version(),
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
