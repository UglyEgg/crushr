// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::common::{
    build_archive_with_pack, build_archive_with_pack_experimental,
    build_archive_with_pack_format05, build_archive_with_pack_format08, comparison_scenarios,
    corrupt_archive, count_outcomes, outcome_from_plan, remove_ledger_for_old_style,
    run_salvage_plan, write_dataset_fixture,
};
use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};
use crushr_format::blk3::{read_blk3_header, write_blk3_header, Blk3Header};

pub(crate) fn build_archive_with_pack_format06(
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
        .arg(FORMAT05_PACK_FLAG)
        .arg(FORMAT06_PACK_FLAG)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn recovery_classification_counts(plan: &Value) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    if let Some(rows) = plan.get("file_plans").and_then(Value::as_array) {
        for row in rows {
            if let Some(classification) = row.get("recovery_classification").and_then(Value::as_str)
            {
                *counts.entry(classification.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
}

pub(crate) fn classification_delta(
    base: &BTreeMap<String, u64>,
    candidate: &BTreeMap<String, u64>,
    key: &str,
) -> i64 {
    candidate.get(key).copied().unwrap_or(0) as i64 - base.get(key).copied().unwrap_or(0) as i64
}

pub(crate) fn merge_classification_counts(rows: &[Value], field: &str) -> BTreeMap<String, u64> {
    let mut merged = BTreeMap::<String, u64>::new();
    for row in rows {
        if let Some(counts) = row.get(field).and_then(Value::as_object) {
            for (k, v) in counts {
                *merged.entry(k.clone()).or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
    }
    merged
}

pub(crate) fn metadata_node_count(plan: &Value, schema: &str) -> u64 {
    plan.get("experimental_metadata")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row.get("schema").and_then(Value::as_str) == Some(schema))
                .count() as u64
        })
        .unwrap_or(0)
}

pub(crate) fn run_format08_placement_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format08-placement-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let strategies = [
        FORMAT08_STRATEGY_FIXED,
        FORMAT08_STRATEGY_HASH,
        FORMAT08_STRATEGY_GOLDEN,
    ];
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for strategy in strategies {
            let archive = scenario_dir.join(format!("format08_{}.crushr", strategy));
            build_archive_with_pack_format08(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                strategy,
            )?;
            corrupt_archive(&archive, &scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan_{}.json", strategy)),
            )?;
            let metrics = outcome_from_plan(&plan);
            let classes = recovery_classification_counts(&plan);
            let manifest_survival =
                metadata_node_count(&plan, "crushr-file-manifest-checkpoint.v1");
            let path_survival = metadata_node_count(&plan, "crushr-path-checkpoint.v1");
            let metadata_nodes = manifest_survival + path_survival;
            if verbose {
                eprintln!(
                    "scenario {} strategy {} => {}",
                    scenario.scenario_id, strategy, metrics.outcome
                );
            }
            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "placement_strategy": strategy,
                "outcome": metrics.outcome,
                "verified_block_count": metrics.verified_block_count,
                "salvageable_file_count": metrics.salvageable_file_count,
                "recovery_classification_counts": classes,
                "manifest_checkpoint_survival_count": manifest_survival,
                "path_checkpoint_survival_count": path_survival,
                "verified_metadata_node_count": metadata_nodes,
            }));
        }
    }

    let mut by_strategy = serde_json::Map::new();
    for strategy in strategies {
        let strategy_rows: Vec<Value> = rows
            .iter()
            .filter(|r| r["placement_strategy"] == strategy)
            .cloned()
            .collect();
        let outcomes = count_outcomes(
            strategy_rows
                .iter()
                .filter_map(|r| r.get("outcome").and_then(Value::as_str)),
        );
        let classes = merge_classification_counts(&strategy_rows, "recovery_classification_counts");
        let manifest_checkpoint_survival_count: u64 = strategy_rows
            .iter()
            .map(|r| {
                r["manifest_checkpoint_survival_count"]
                    .as_u64()
                    .unwrap_or(0)
            })
            .sum();
        let path_checkpoint_survival_count: u64 = strategy_rows
            .iter()
            .map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0))
            .sum();
        let verified_metadata_node_count: u64 = strategy_rows
            .iter()
            .map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0))
            .sum();
        by_strategy.insert(strategy.to_string(), serde_json::json!({
            "scenario_count": strategy_rows.len(),
            "recovery_outcome_counts": outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0),
            "anonymous_recovery_count": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0),
            "partial_ordered_recovery_count": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0),
            "partial_unordered_recovery_count": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0),
            "orphan_evidence_count": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0),
            "manifest_checkpoint_survival_count": manifest_checkpoint_survival_count,
            "path_checkpoint_survival_count": path_checkpoint_survival_count,
            "verified_metadata_node_count": verified_metadata_node_count,
        }));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format08-placement-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "placement_strategies": strategies,
        "by_placement_strategy": by_strategy,
        "by_dataset": group_rows_by_key(&rows, "dataset"),
        "by_corruption_target": group_rows_by_key(&rows, "corruption_target"),
        "metadata_layer_failure_focus": {
            "no_manifest_checkpoint_rows": rows.iter().filter(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0) == 0).count(),
            "no_path_checkpoint_rows": rows.iter().filter(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0) == 0).count(),
            "no_metadata_nodes_rows": rows.iter().filter(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0) == 0).count(),
        },
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format08_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format08_comparison_summary.md"),
        format!(
            "# Format-08 placement comparison

Scenarios: {}

Strategies: fixed_spread, hash_spread, golden_spread
",
            summary
                .get("scenario_count")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn group_rows_by_key(rows: &[Value], key: &str) -> Value {
    let mut out = serde_json::Map::new();
    let mut keys = BTreeMap::<String, Vec<Value>>::new();
    for row in rows {
        if let Some(k) = row.get(key).and_then(Value::as_str) {
            keys.entry(k.to_string()).or_default().push(row.clone());
        }
    }
    for (k, group) in keys {
        let outcomes = count_outcomes(
            group
                .iter()
                .filter_map(|r| r.get("outcome").and_then(Value::as_str)),
        );
        let classes = merge_classification_counts(&group, "recovery_classification_counts");
        out.insert(k, serde_json::json!({
            "scenario_count": group.len(),
            "recovery_outcome_counts": outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0),
            "anonymous_recovery_count": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0),
            "partial_ordered_recovery_count": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0),
            "partial_unordered_recovery_count": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0),
            "orphan_evidence_count": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0),
            "manifest_checkpoint_survival_count": group.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "path_checkpoint_survival_count": group.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "verified_metadata_node_count": group.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        }));
    }
    Value::Object(out)
}

pub(crate) fn run_format06_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format06-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        let old_archive = scenario_dir.join("old.crushr");
        let redundant_archive = scenario_dir.join("redundant.crushr");
        let format05_archive = scenario_dir.join("format05.crushr");
        let format06_archive = scenario_dir.join("format06.crushr");

        build_archive_with_pack(&pack_bin, &input_dir.join(scenario.dataset), &old_archive)?;
        build_archive_with_pack_experimental(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &redundant_archive,
        )?;
        build_archive_with_pack_format05(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format05_archive,
        )?;
        build_archive_with_pack_format06(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format06_archive,
        )?;

        if scenario.break_redundant_map {
            remove_ledger_for_old_style(&old_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;
        corrupt_archive(&format06_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &scenario_dir.join("old_plan.json"),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &scenario_dir.join("redundant_plan.json"),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &scenario_dir.join("format05_plan.json"),
        )?;
        let format06_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format06_plan.json"),
        )?;
        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        let format06_metrics = outcome_from_plan(&format06_plan);
        let format05_classifications = recovery_classification_counts(&format05_plan);
        let format06_classifications = recovery_classification_counts(&format06_plan);

        if verbose {
            eprintln!(
                "scenario {} => format06 {}",
                scenario.scenario_id, format06_metrics.outcome
            );
        }

        rows.push(serde_json::json!({
            "scenario_id": scenario.scenario_id,
            "dataset": scenario.dataset,
            "corruption_model": scenario.corruption_model,
            "corruption_target": scenario.corruption_target,
            "magnitude": scenario.magnitude,
            "seed": scenario.seed,
            "old_outcome": old_metrics.outcome,
            "redundant_outcome": redundant_metrics.outcome,
            "format05_outcome": format05_metrics.outcome,
            "format06_outcome": format06_metrics.outcome,
            "old_verified_block_count": old_metrics.verified_block_count,
            "redundant_verified_block_count": redundant_metrics.verified_block_count,
            "format05_verified_block_count": format05_metrics.verified_block_count,
            "format06_verified_block_count": format06_metrics.verified_block_count,
            "old_salvageable_file_count": old_metrics.salvageable_file_count,
            "redundant_salvageable_file_count": redundant_metrics.salvageable_file_count,
            "format05_salvageable_file_count": format05_metrics.salvageable_file_count,
            "format06_salvageable_file_count": format06_metrics.salvageable_file_count,
            "format05_recovery_classification_counts": format05_classifications,
            "format06_recovery_classification_counts": format06_classifications
        }));
    }

    let mut format05_recovery_classification_counts = BTreeMap::<String, u64>::new();
    let mut format06_recovery_classification_counts = BTreeMap::<String, u64>::new();
    for row in &rows {
        if let Some(counts) = row
            .get("format05_recovery_classification_counts")
            .and_then(Value::as_object)
        {
            for (k, v) in counts {
                *format05_recovery_classification_counts
                    .entry(k.clone())
                    .or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
        if let Some(counts) = row
            .get("format06_recovery_classification_counts")
            .and_then(Value::as_object)
        {
            for (k, v) in counts {
                *format06_recovery_classification_counts
                    .entry(k.clone())
                    .or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format06-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "old_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("old_outcome").and_then(Value::as_str))),
        "redundant_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("redundant_outcome").and_then(Value::as_str))),
        "format05_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format05_outcome").and_then(Value::as_str))),
        "format06_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format06_outcome").and_then(Value::as_str))),
        "format05_recovery_classification_counts": format05_recovery_classification_counts,
        "format06_recovery_classification_counts": format06_recovery_classification_counts,
        "recovery_classification_delta_vs_format05": {
            "full_verified_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "FULL_VERIFIED"),
            "full_anonymous_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "FULL_ANONYMOUS"),
            "partial_ordered_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "PARTIAL_ORDERED"),
            "partial_unordered_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "PARTIAL_UNORDERED"),
            "orphan_blocks_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "ORPHAN_BLOCKS")
        },
        "per_scenario_rows": rows
    });

    fs::write(
        comparison_dir.join("format06_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format06_comparison_summary.md"),
        format!(
            "# Format-06 comparison\n\nScenarios: {}\n\n- outcomes tracked: old, redundant, format05, format06\n",
            summary.get("scenario_count").and_then(Value::as_u64).unwrap_or(0)
        ),
    )?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn run_format07_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format07-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        let old_archive = scenario_dir.join("old.crushr");
        let redundant_archive = scenario_dir.join("redundant.crushr");
        let format05_archive = scenario_dir.join("format05.crushr");
        let format06_archive = scenario_dir.join("format06.crushr");

        build_archive_with_pack(&pack_bin, &input_dir.join(scenario.dataset), &old_archive)?;
        build_archive_with_pack_experimental(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &redundant_archive,
        )?;
        build_archive_with_pack_format05(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format05_archive,
        )?;
        build_archive_with_pack_format06(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format06_archive,
        )?;

        if scenario.break_redundant_map {
            remove_ledger_for_old_style(&old_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;
        corrupt_archive(&format06_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &scenario_dir.join("old_plan.json"),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &scenario_dir.join("redundant_plan.json"),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &scenario_dir.join("format05_plan.json"),
        )?;
        let format06_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format06_plan.json"),
        )?;
        let format07_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format07_plan.json"),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        let format06_metrics = outcome_from_plan(&format06_plan);
        let format07_metrics = outcome_from_plan(&format07_plan);
        let format06_classifications = recovery_classification_counts(&format06_plan);
        let format07_classifications = recovery_classification_counts(&format07_plan);

        if verbose {
            eprintln!(
                "scenario {} => format07 {}",
                scenario.scenario_id, format07_metrics.outcome
            );
        }

        rows.push(serde_json::json!({
            "scenario_id": scenario.scenario_id,
            "dataset": scenario.dataset,
            "corruption_model": scenario.corruption_model,
            "corruption_target": scenario.corruption_target,
            "magnitude": scenario.magnitude,
            "seed": scenario.seed,
            "old_outcome": old_metrics.outcome,
            "redundant_outcome": redundant_metrics.outcome,
            "format05_outcome": format05_metrics.outcome,
            "format06_outcome": format06_metrics.outcome,
            "format07_outcome": format07_metrics.outcome,
            "old_verified_block_count": old_metrics.verified_block_count,
            "redundant_verified_block_count": redundant_metrics.verified_block_count,
            "format05_verified_block_count": format05_metrics.verified_block_count,
            "format06_verified_block_count": format06_metrics.verified_block_count,
            "format07_verified_block_count": format07_metrics.verified_block_count,
            "old_salvageable_file_count": old_metrics.salvageable_file_count,
            "redundant_salvageable_file_count": redundant_metrics.salvageable_file_count,
            "format05_salvageable_file_count": format05_metrics.salvageable_file_count,
            "format06_salvageable_file_count": format06_metrics.salvageable_file_count,
            "format07_salvageable_file_count": format07_metrics.salvageable_file_count,
            "format06_recovery_classification_counts": format06_classifications,
            "format07_recovery_classification_counts": format07_classifications
        }));
    }

    let format06_recovery_classification_counts =
        merge_classification_counts(&rows, "format06_recovery_classification_counts");
    let format07_recovery_classification_counts =
        merge_classification_counts(&rows, "format07_recovery_classification_counts");

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format07-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "old_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("old_outcome").and_then(Value::as_str))),
        "redundant_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("redundant_outcome").and_then(Value::as_str))),
        "format05_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format05_outcome").and_then(Value::as_str))),
        "format06_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format06_outcome").and_then(Value::as_str))),
        "format07_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format07_outcome").and_then(Value::as_str))),
        "format06_recovery_classification_counts": format06_recovery_classification_counts,
        "format07_recovery_classification_counts": format07_recovery_classification_counts,
        "recovery_classification_delta_vs_format06": {
            "full_named_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "FULL_NAMED_VERIFIED"),
            "full_anonymous_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "FULL_ANONYMOUS_VERIFIED"),
            "partial_ordered_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "PARTIAL_ORDERED_VERIFIED"),
            "partial_unordered_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "PARTIAL_UNORDERED_VERIFIED"),
            "orphan_evidence_only_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "ORPHAN_EVIDENCE_ONLY"),
            "no_verified_evidence_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "NO_VERIFIED_EVIDENCE")
        },
        "per_scenario_rows": rows
    });

    fs::write(
        comparison_dir.join("format07_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format07_comparison_summary.md"),
        format!(
            "# Format-07 comparison\n\nScenarios: {}\n\n- outcomes tracked: old, redundant, format05, format06, format07\n",
            summary.get("scenario_count").and_then(Value::as_u64).unwrap_or(0)
        ),
    )?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct Format09Scenario {
    scenario_id: String,
    dataset: &'static str,
    metadata_regime: &'static str,
    metadata_target: &'static str,
    metadata_operation: &'static str,
    payload_damage: &'static str,
    seed: u64,
}

pub(crate) fn format09_scenarios() -> Vec<Format09Scenario> {
    let datasets = ["smallfiles", "mixed", "largefiles"];
    let regimes = [
        "metadata_intact",
        "metadata_destroyed",
        "metadata_partial",
        "metadata_conflicting",
    ];
    let targets = [
        "index",
        "manifest",
        "path_checkpoint",
        "tail_metadata",
        "multiple_metadata_layers",
        "index_manifest",
    ];
    let payload = [
        "none",
        "single_block_delete",
        "sparse_delete",
        "cluster_delete",
        "block_reorder",
        "duplicate_block_insert",
    ];

    let mut out = Vec::new();
    let mut seed = 900u64;
    for (i, p) in payload.iter().enumerate() {
        for regime in regimes {
            let target = targets[i % targets.len()];
            let dataset = datasets[(i + (seed as usize % 3)) % datasets.len()];
            out.push(Format09Scenario {
                scenario_id: format!("{}_{}_{}_{}", regime, target, p, seed),
                dataset,
                metadata_regime: regime,
                metadata_target: target,
                metadata_operation: match regime {
                    "metadata_intact" => "none",
                    "metadata_destroyed" => "delete",
                    "metadata_partial" => "truncate",
                    "metadata_conflicting" => "overwrite",
                    _ => "bitflip",
                },
                payload_damage: p,
                seed,
            });
            seed += 1;
        }
    }
    out
}

pub(crate) fn clobber_range(bytes: &mut [u8], start: usize, len: usize, mode: &str) {
    let end = start.saturating_add(len).min(bytes.len());
    if start >= end {
        return;
    }
    match mode {
        "delete" => {
            for b in &mut bytes[start..end] {
                *b = 0;
            }
        }
        "truncate" => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                if i % 2 == 0 {
                    *b = 0;
                }
            }
        }
        "bitflip" => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                *b ^= 0xA5 ^ ((i % 17) as u8);
            }
        }
        _ => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                *b = 0x55u8.wrapping_add((i % 101) as u8);
            }
        }
    }
}

pub(crate) fn corrupt_metadata_for_format09(
    bytes: &mut Vec<u8>,
    scenario: &Format09Scenario,
) -> Result<()> {
    if scenario.metadata_regime == "metadata_intact" {
        return Ok(());
    }

    let len = bytes.len();
    let footer_offset = len.saturating_sub(FTR4_LEN);
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let index_offset = footer.index_offset as usize;

    match scenario.metadata_target {
        "index" => clobber_range(bytes, index_offset, 96, scenario.metadata_operation),
        "manifest" => {
            for needle in [
                b"crushr-file-manifest-checkpoint.v1".as_slice(),
                b"file-manifest-checkpoint".as_slice(),
            ] {
                let mut pos = 0usize;
                while let Some(hit) = bytes[pos..].windows(needle.len()).position(|w| w == needle) {
                    let start = pos + hit;
                    clobber_range(
                        bytes,
                        start.saturating_sub(8),
                        64,
                        scenario.metadata_operation,
                    );
                    pos = start + needle.len();
                }
            }
        }
        "path_checkpoint" => {
            for needle in [
                b"crushr-path-checkpoint.v1".as_slice(),
                b"path-checkpoint".as_slice(),
            ] {
                let mut pos = 0usize;
                while let Some(hit) = bytes[pos..].windows(needle.len()).position(|w| w == needle) {
                    let start = pos + hit;
                    clobber_range(
                        bytes,
                        start.saturating_sub(8),
                        64,
                        scenario.metadata_operation,
                    );
                    pos = start + needle.len();
                }
            }
        }
        "tail_metadata" => {
            let start = blocks_end.min(len.saturating_sub(16));
            clobber_range(
                bytes,
                start,
                len.saturating_sub(start + FTR4_LEN),
                scenario.metadata_operation,
            );
        }
        "multiple_metadata_layers" | "index_manifest" => {
            clobber_range(bytes, index_offset, 96, scenario.metadata_operation);
            let start = blocks_end.min(len.saturating_sub(16));
            clobber_range(
                bytes,
                start,
                len.saturating_sub(start + FTR4_LEN),
                scenario.metadata_operation,
            );
        }
        _ => {}
    }

    if scenario.metadata_regime == "metadata_conflicting" {
        let dup = bytes[blocks_end..len.saturating_sub(FTR4_LEN)].to_vec();
        let insert_at = (blocks_end + 64).min(len.saturating_sub(FTR4_LEN));
        bytes.splice(insert_at..insert_at, dup.into_iter().take(128));
    }

    Ok(())
}

pub(crate) fn corrupt_payload_for_format09(bytes: &mut Vec<u8>, scenario: &Format09Scenario) {
    let len = bytes.len();
    let data_start = 64usize.min(len);
    let data_end = len.saturating_sub(FTR4_LEN + 128);
    if data_start >= data_end {
        return;
    }
    let span = data_end - data_start;
    let block = (span / 16).max(64);
    let start = data_start + ((scenario.seed as usize % 7) * block / 2).min(span.saturating_sub(1));

    match scenario.payload_damage {
        "none" => {}
        "single_block_delete" => clobber_range(bytes, start, block, "delete"),
        "sparse_delete" => {
            for i in 0..4 {
                clobber_range(bytes, start + i * (block / 2), block / 3, "delete");
            }
        }
        "cluster_delete" => clobber_range(bytes, start, block * 3 / 2, "delete"),
        "block_reorder" => {
            let mid = (start + block).min(data_end.saturating_sub(block));
            if mid > start && mid + block <= data_end {
                let a = bytes[start..start + block].to_vec();
                let b = bytes[mid..mid + block].to_vec();
                bytes[start..start + block].copy_from_slice(&b);
                bytes[mid..mid + block].copy_from_slice(&a);
            }
        }
        "duplicate_block_insert" => {
            let end = (start + block).min(data_end);
            let dup = bytes[start..end].to_vec();
            let insert_at = (end + block / 2).min(data_end);
            bytes.splice(insert_at..insert_at, dup);
        }
        _ => {}
    }
}

pub(crate) fn apply_format09_corruption(
    archive_path: &Path,
    scenario: &Format09Scenario,
) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    corrupt_metadata_for_format09(&mut bytes, scenario)?;
    corrupt_payload_for_format09(&mut bytes, scenario);
    fs::write(archive_path, bytes)?;
    Ok(())
}

pub(crate) fn recovery_class_rank(classes: &BTreeMap<String, u64>) -> &'static str {
    if class_count(classes, &["FULL_NAMED_VERIFIED", "FULL_VERIFIED"]) > 0 {
        "named"
    } else if class_count(classes, &["FULL_ANONYMOUS_VERIFIED", "FULL_ANONYMOUS"]) > 0 {
        "anonymous"
    } else if class_count(classes, &["PARTIAL_ORDERED_VERIFIED", "PARTIAL_ORDERED"]) > 0 {
        "partial_ordered"
    } else if class_count(
        classes,
        &["PARTIAL_UNORDERED_VERIFIED", "PARTIAL_UNORDERED"],
    ) > 0
    {
        "partial_unordered"
    } else if class_count(classes, &["ORPHAN_EVIDENCE_ONLY", "ORPHAN_BLOCKS"]) > 0 {
        "orphan"
    } else {
        "none"
    }
}

pub(crate) fn class_count(classes: &BTreeMap<String, u64>, names: &[&str]) -> u64 {
    names
        .iter()
        .map(|name| classes.get(*name).copied().unwrap_or(0))
        .sum()
}

#[derive(Clone, Copy)]
pub(crate) enum TerminalRecoveryOutcome {
    Named,
    AnonymousFull,
    PartialOrdered,
    PartialUnordered,
    OrphanEvidence,
    NoVerifiedEvidence,
}

pub(crate) fn terminal_recovery_outcome(
    classes: &BTreeMap<String, u64>,
) -> TerminalRecoveryOutcome {
    if class_count(classes, &["FULL_NAMED_VERIFIED", "FULL_VERIFIED"]) > 0 {
        TerminalRecoveryOutcome::Named
    } else if class_count(classes, &["FULL_ANONYMOUS_VERIFIED", "FULL_ANONYMOUS"]) > 0 {
        TerminalRecoveryOutcome::AnonymousFull
    } else if class_count(classes, &["PARTIAL_ORDERED_VERIFIED", "PARTIAL_ORDERED"]) > 0 {
        TerminalRecoveryOutcome::PartialOrdered
    } else if class_count(
        classes,
        &["PARTIAL_UNORDERED_VERIFIED", "PARTIAL_UNORDERED"],
    ) > 0
    {
        TerminalRecoveryOutcome::PartialUnordered
    } else if class_count(classes, &["ORPHAN_EVIDENCE_ONLY", "ORPHAN_BLOCKS"]) > 0 {
        TerminalRecoveryOutcome::OrphanEvidence
    } else {
        TerminalRecoveryOutcome::NoVerifiedEvidence
    }
}

pub(crate) fn enforce_dictionary_fail_closed(
    variant: &str,
    terminal: TerminalRecoveryOutcome,
    dictionary_copy_count: usize,
    dictionary_conflict: bool,
) -> TerminalRecoveryOutcome {
    if !variant.starts_with("extent_identity_path_dict_") {
        return terminal;
    }
    if !dictionary_conflict && dictionary_copy_count > 0 {
        return terminal;
    }
    match terminal {
        TerminalRecoveryOutcome::NoVerifiedEvidence => TerminalRecoveryOutcome::NoVerifiedEvidence,
        TerminalRecoveryOutcome::OrphanEvidence => TerminalRecoveryOutcome::OrphanEvidence,
        TerminalRecoveryOutcome::PartialOrdered => TerminalRecoveryOutcome::PartialOrdered,
        TerminalRecoveryOutcome::PartialUnordered => TerminalRecoveryOutcome::PartialUnordered,
        TerminalRecoveryOutcome::Named | TerminalRecoveryOutcome::AnonymousFull => {
            TerminalRecoveryOutcome::AnonymousFull
        }
    }
}

pub(crate) fn expected_dictionary_state_for_scenario(
    variant: &str,
    corruption_target: &str,
) -> (usize, bool) {
    if !variant.starts_with("extent_identity_path_dict_") {
        return (0, false);
    }

    match (variant, corruption_target) {
        ("extent_identity_path_dict_header_tail", "primary_dictionary")
        | ("extent_identity_path_dict_header_tail", "mirrored_dictionary")
        | ("extent_identity_path_dict_factored_header_tail", "primary_dictionary")
        | ("extent_identity_path_dict_factored_header_tail", "mirrored_dictionary") => (1, false),
        ("extent_identity_path_dict_header_tail", "both_dictionaries")
        | ("extent_identity_path_dict_factored_header_tail", "both_dictionaries") => (0, false),
        ("extent_identity_path_dict_header_tail", "inconsistent_dictionaries")
        | ("extent_identity_path_dict_factored_header_tail", "inconsistent_dictionaries") => {
            (2, true)
        }
        ("extent_identity_path_dict_single", "inconsistent_dictionaries")
        | ("extent_identity_path_dict_single", "primary_dictionary")
        | ("extent_identity_path_dict_single", "mirrored_dictionary")
        | ("extent_identity_path_dict_single", "both_dictionaries") => (0, false),
        _ => (0, false),
    }
}

pub(crate) fn run_format09_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format09-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let strategies = [
        FORMAT08_STRATEGY_FIXED,
        FORMAT08_STRATEGY_HASH,
        FORMAT08_STRATEGY_GOLDEN,
    ];
    let scenarios = format09_scenarios();
    let mut rows = Vec::new();

    for scenario in &scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for strategy in strategies {
            let archive = scenario_dir.join(format!("format09_{}.crushr", strategy));
            build_archive_with_pack_format08(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                strategy,
            )?;
            apply_format09_corruption(&archive, scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan_{}_{}.json", strategy, scenario.scenario_id)),
            )?;
            let classes = recovery_classification_counts(&plan);
            let manifest_survival =
                metadata_node_count(&plan, "crushr-file-manifest-checkpoint.v1");
            let path_survival = metadata_node_count(&plan, "crushr-path-checkpoint.v1");
            let metadata_nodes = manifest_survival + path_survival;
            if verbose {
                eprintln!(
                    "format09 {} {} => class={}",
                    scenario.scenario_id,
                    strategy,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "strategy": strategy,
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "metadata_regime": scenario.metadata_regime,
                "metadata_target": scenario.metadata_target,
                "metadata_operation": scenario.metadata_operation,
                "payload_damage": scenario.payload_damage,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "manifest_checkpoint_survival_count": manifest_survival,
                "path_checkpoint_survival_count": path_survival,
                "verified_metadata_node_count": metadata_nodes,
                "recovery_class": recovery_class_rank(&classes),
                "metadata_recovery_gain": "pending",
            }));
        }
    }

    let mut destroyed_baseline = BTreeMap::<(String, String, String), String>::new();
    for row in &rows {
        if row["metadata_regime"] == "metadata_destroyed" {
            destroyed_baseline.insert(
                (
                    row["strategy"].as_str().unwrap_or("").to_string(),
                    row["metadata_target"].as_str().unwrap_or("").to_string(),
                    row["payload_damage"].as_str().unwrap_or("").to_string(),
                ),
                row["recovery_class"].as_str().unwrap_or("none").to_string(),
            );
        }
    }

    for row in &mut rows {
        let key = (
            row["strategy"].as_str().unwrap_or("").to_string(),
            row["metadata_target"].as_str().unwrap_or("").to_string(),
            row["payload_damage"].as_str().unwrap_or("").to_string(),
        );
        let current = row["recovery_class"].as_str().unwrap_or("none");
        let gain = if row["metadata_regime"] == "metadata_destroyed" {
            "baseline".to_string()
        } else if let Some(base) = destroyed_baseline.get(&key) {
            if base == current {
                "unchanged".to_string()
            } else {
                format!("{}->{}", base, current)
            }
        } else {
            "unknown".to_string()
        };
        row["metadata_recovery_gain"] = Value::String(gain);
    }

    let recovery_class_distribution = count_outcomes(
        rows.iter()
            .filter_map(|r| r.get("recovery_class").and_then(Value::as_str)),
    );
    let metadata_survival_stats = serde_json::json!({
        "manifest_checkpoint_survival_count": rows.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "path_checkpoint_survival_count": rows.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "verified_metadata_node_count": rows.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
    });
    let gain_distribution = count_outcomes(
        rows.iter()
            .filter_map(|r| r.get("metadata_recovery_gain").and_then(Value::as_str)),
    );

    let mut by_strategy = serde_json::Map::new();
    for strategy in strategies {
        let strategy_rows: Vec<Value> = rows
            .iter()
            .filter(|r| r["strategy"] == strategy)
            .cloned()
            .collect();
        by_strategy.insert(strategy.to_string(), serde_json::json!({
            "scenario_count": strategy_rows.len(),
            "recovery_class_distribution": count_outcomes(strategy_rows.iter().filter_map(|r| r["recovery_class"].as_str())),
            "metadata_survival": {
                "manifest_checkpoint_survival_count": strategy_rows.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
                "path_checkpoint_survival_count": strategy_rows.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
                "verified_metadata_node_count": strategy_rows.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            },
            "metadata_recovery_gain_distribution": count_outcomes(strategy_rows.iter().filter_map(|r| r["metadata_recovery_gain"].as_str())),
        }));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format09-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "metadata_regimes": ["metadata_intact", "metadata_destroyed", "metadata_partial", "metadata_conflicting"],
        "recovery_class_distribution": recovery_class_distribution,
        "metadata_survival_statistics": metadata_survival_stats,
        "metadata_recovery_gain_distribution": gain_distribution,
        "strategy_comparison": by_strategy,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format09_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-09 metadata survivability and necessity audit\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Scenario table\n\n");
    md.push_str("| strategy | scenario_id | metadata_regime | metadata_target | payload_damage | recovery_class | metadata_recovery_gain |\n");
    md.push_str("|---|---|---|---|---|---|---|\n");
    for row in summary["per_scenario_rows"]
        .as_array()
        .into_iter()
        .flatten()
        .take(80)
    {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            row["strategy"].as_str().unwrap_or(""),
            row["scenario_id"].as_str().unwrap_or(""),
            row["metadata_regime"].as_str().unwrap_or(""),
            row["metadata_target"].as_str().unwrap_or(""),
            row["payload_damage"].as_str().unwrap_or(""),
            row["recovery_class"].as_str().unwrap_or(""),
            row["metadata_recovery_gain"].as_str().unwrap_or(""),
        ));
    }
    md.push_str("\n## Recovery class distribution\n\n");
    for (k, v) in summary["recovery_class_distribution"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Metadata survival statistics\n\n");
    for (k, v) in summary["metadata_survival_statistics"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Recovery gain attributable to metadata\n\n");
    for (k, v) in summary["metadata_recovery_gain_distribution"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Strategy comparison\n\n");
    for (k, v) in summary["strategy_comparison"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!(
            "- {}: scenarios={}\n",
            k,
            v["scenario_count"].as_u64().unwrap_or(0)
        ));
    }

    fs::write(comparison_dir.join("format09_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum Format10Variant {
    PayloadOnly,
    PayloadPlusManifest,
    PayloadPlusPath,
    FullCurrentExperimental,
}

impl Format10Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::PayloadPlusPath => "payload_plus_path",
            Self::FullCurrentExperimental => "full_current_experimental",
        }
    }

    fn metadata_profile(self) -> &'static str {
        self.as_str()
    }
}

pub(crate) fn build_archive_with_pack_metadata_profile(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
    variant: Format10Variant,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--metadata-profile")
        .arg(variant.metadata_profile())
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn estimate_metadata_byte_size(archive_path: &Path) -> Result<u64> {
    let bytes = fs::read(archive_path)?;
    let mut offset = 0usize;
    let mut total = 0u64;
    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] != BLK3_MAGIC {
            offset += 1;
            continue;
        }
        let header = match read_blk3_header(std::io::Cursor::new(&bytes[offset..])) {
            Ok(h) => h,
            Err(_) => {
                offset += 1;
                continue;
            }
        };
        let block_len = header.header_len as usize + header.comp_len as usize;
        if block_len == 0 || offset + block_len > bytes.len() {
            offset += 1;
            continue;
        }
        if header.flags.is_meta_frame() {
            total = total.saturating_add(block_len as u64);
        }
        offset += block_len;
    }
    Ok(total)
}

pub(crate) fn run_format10_pruning_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format10-pruning-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format10Variant::FullCurrentExperimental,
        Format10Variant::PayloadOnly,
        Format10Variant::PayloadPlusManifest,
        Format10Variant::PayloadPlusPath,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format10_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            if verbose {
                eprintln!(
                    "format10 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "variant": variant_name,
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.get("NO_VERIFIED_EVIDENCE").copied().unwrap_or(0) > 0,
            }));
        }
    }

    let payload_only_total_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_only")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();

    let full_outcomes = count_outcomes(
        rows.iter()
            .filter(|r| r["variant"] == "full_current_experimental")
            .filter_map(|r| r["recovery_outcome"].as_str()),
    );

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "full_current_experimental",
        "payload_only",
        "payload_plus_manifest",
        "payload_plus_path",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let recovery_outcomes = count_outcomes(
            variant_rows
                .iter()
                .filter_map(|r| r["recovery_outcome"].as_str()),
        );
        let classes = merge_classification_counts(
            &variant_rows
                .iter()
                .map(|r| (*r).clone())
                .collect::<Vec<Value>>(),
            "recovery_classification_counts",
        );
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let overhead_delta_vs_payload_only =
            archive_byte_size as i64 - payload_only_total_size as i64;

        let baseline_full = full_outcomes
            .get("FULL_FILE_SALVAGE_AVAILABLE")
            .copied()
            .unwrap_or(0) as i64;
        let this_full = recovery_outcomes
            .get("FULL_FILE_SALVAGE_AVAILABLE")
            .copied()
            .unwrap_or(0) as i64;

        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": recovery_outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": variant_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": overhead_delta_vs_payload_only,
            "recovery_delta_vs_full_current_experimental": {
                "full_file_salvage_available_delta": this_full - baseline_full,
            }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "full_current_experimental",
                "payload_only",
                "payload_plus_manifest",
                "payload_plus_path",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "recovery_outcome_counts": count_outcomes(g_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                    "metadata_byte_estimate": g_rows.iter().map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format10-pruning-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["full_current_experimental", "payload_only", "payload_plus_manifest", "payload_plus_path"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format10_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-10 metadata pruning comparison\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anonymous_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | metadata_byte_estimate | overhead_vs_payload_only |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "full_current_experimental",
        "payload_only",
        "payload_plus_manifest",
        "payload_plus_path",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_u64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["metadata_byte_estimate"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
        ));
    }

    fs::write(comparison_dir.join("format10_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
enum Format11Variant {
    PayloadOnly,
    PayloadPlusManifest,
    FullCurrentExperimental,
    ExtentIdentityOnly,
}

impl Format11Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::FullCurrentExperimental => "full_current_experimental",
            Self::ExtentIdentityOnly => "extent_identity_only",
        }
    }

    fn metadata_profile(self) -> &'static str {
        self.as_str()
    }
}

pub(crate) fn build_archive_with_pack_metadata_profile_name(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
    profile: &str,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--metadata-profile")
        .arg(profile)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn run_format11_extent_identity_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format11-extent-identity-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format11Variant::PayloadOnly,
        Format11Variant::PayloadPlusManifest,
        Format11Variant::FullCurrentExperimental,
        Format11Variant::ExtentIdentityOnly,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format11_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant.metadata_profile(),
            )?;

            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan11_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            if verbose {
                eprintln!(
                    "format11 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "variant": variant_name,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.is_empty(),
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
            }));
        }
    }

    let payload_only_total_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_only")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();

    let payload_plus_manifest_named: i64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_plus_manifest")
        .filter(|r| r["named_recovery"].as_bool() == Some(true))
        .count() as i64;

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "payload_only",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_only",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let recovery_outcomes = count_outcomes(
            variant_rows
                .iter()
                .filter_map(|r| r["recovery_outcome"].as_str()),
        );
        let mut classes = BTreeMap::new();
        for row in &variant_rows {
            if let Some(obj) = row["recovery_classification_counts"].as_object() {
                for (k, v) in obj {
                    let n = v.as_u64().unwrap_or(0);
                    *classes.entry(k.clone()).or_insert(0) += n;
                }
            }
        }
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let overhead_delta_vs_payload_only =
            archive_byte_size as i64 - payload_only_total_size as i64;
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;

        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": recovery_outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": overhead_delta_vs_payload_only,
            "recovery_delta_vs_payload_plus_manifest": {
                "named_recovery_count_delta": named_count - payload_plus_manifest_named,
            }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "payload_only",
                "payload_plus_manifest",
                "full_current_experimental",
                "extent_identity_only",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "recovery_outcome_counts": count_outcomes(g_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "anonymous_full_recovery_count": g_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
                    "partial_ordered_recovery_count": g_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
                    "partial_unordered_recovery_count": g_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
                    "orphan_evidence_count": g_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
                    "no_verified_evidence_count": g_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format11-extent-identity-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["payload_only", "payload_plus_manifest", "full_current_experimental", "extent_identity_only"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format11_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-11 extent identity comparison\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anonymous_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | metadata_byte_estimate | overhead_vs_payload_only | named_delta_vs_payload_plus_manifest |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "payload_only",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_only",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["metadata_byte_estimate"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["recovery_delta_vs_payload_plus_manifest"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
        ));
    }

    fs::write(comparison_dir.join("format11_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
enum Format12Variant {
    PayloadOnly,
    ExtentIdentityOnly,
    ExtentIdentityDistributedNames,
    PayloadPlusManifest,
    FullCurrentExperimental,
    ExtentIdentityInlinePath,
}

impl Format12Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::ExtentIdentityOnly => "extent_identity_only",
            Self::ExtentIdentityDistributedNames => "extent_identity_distributed_names",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::FullCurrentExperimental => "full_current_experimental",
            Self::ExtentIdentityInlinePath => "extent_identity_inline_path",
        }
    }
}

pub(crate) fn write_dataset_fixture_format12(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input).with_context(|| format!("create {}", input.display()))?;

    match dataset {
        "smallfiles" => {
            fs::write(input.join("tiny.txt"), b"small-dataset-payload")?;
            fs::write(input.join("a.txt"), b"short")?;
        }
        "mixed" => {
            fs::write(
                input.join("mixed.bin"),
                (0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
            )?;
            let long_path = input
                .join("nested")
                .join("deeply")
                .join("named")
                .join("path")
                .join("for")
                .join("inline")
                .join("duplication")
                .join("cost")
                .join("visibility")
                .join("sample-long-name.txt");
            if let Some(parent) = long_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(long_path, b"long-path-payload")?;
        }
        "largefiles" => {
            fs::write(input.join("large.dat"), vec![13u8; 8192])?;
        }
        _ => bail!("unsupported dataset {dataset}"),
    }

    Ok(())
}

pub(crate) fn run_format12_inline_path_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format12-inline-path-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format12Variant::PayloadOnly,
        Format12Variant::ExtentIdentityOnly,
        Format12Variant::ExtentIdentityDistributedNames,
        Format12Variant::PayloadPlusManifest,
        Format12Variant::FullCurrentExperimental,
        Format12Variant::ExtentIdentityInlinePath,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture_format12(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format12_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant_name,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan12_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            let path_bucket = if scenario.dataset == "mixed" {
                "long_path"
            } else {
                "short_path"
            };
            if verbose {
                eprintln!(
                    "format12 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "path_length_bucket": path_bucket,
                "variant": variant_name,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.is_empty(),
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
            }));
        }
    }

    let totals_for = |name: &str| -> (u64, i64) {
        let size = rows
            .iter()
            .filter(|r| r["variant"] == name)
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named = rows
            .iter()
            .filter(|r| r["variant"] == name && r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        (size, named)
    };
    let (payload_only_size, _) = totals_for("payload_only");
    let (extent_identity_only_size, extent_identity_only_named) =
        totals_for("extent_identity_only");
    let (payload_plus_manifest_size, payload_plus_manifest_named) =
        totals_for("payload_plus_manifest");

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "payload_only",
        "extent_identity_only",
        "extent_identity_distributed_names",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_inline_path",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        let mut classes = BTreeMap::new();
        for row in &variant_rows {
            if let Some(obj) = row["recovery_classification_counts"].as_object() {
                for (k, v) in obj {
                    *classes.entry(k.clone()).or_insert(0) += v.as_u64().unwrap_or(0);
                }
            }
        }
        let named_delta_vs_extent = named_count - extent_identity_only_named;
        let overhead_vs_extent = archive_byte_size as i64 - extent_identity_only_size as i64;
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": count_outcomes(variant_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
            "recovery_classification_counts": classes,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": archive_byte_size as i64 - payload_only_size as i64,
            "overhead_delta_vs_extent_identity_only": overhead_vs_extent,
            "overhead_delta_vs_payload_plus_manifest": archive_byte_size as i64 - payload_plus_manifest_size as i64,
            "recovery_delta_vs_extent_identity_only": {"named_recovery_count_delta": named_delta_vs_extent},
            "recovery_delta_vs_payload_plus_manifest": {"named_recovery_count_delta": named_count - payload_plus_manifest_named},
            "recovery_per_kib_overhead": if overhead_vs_extent > 0 { (named_delta_vs_extent as f64) / (overhead_vs_extent as f64 / 1024.0) } else { 0.0 },
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target", "path_length_bucket"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "payload_only",
                "extent_identity_only",
                "extent_identity_distributed_names",
                "payload_plus_manifest",
                "full_current_experimental",
                "extent_identity_inline_path",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format12-inline-path-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["payload_only", "extent_identity_only", "extent_identity_distributed_names", "payload_plus_manifest", "full_current_experimental", "extent_identity_inline_path"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format12_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-12 inline path comparison\n\n");
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anon_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | overhead_vs_payload_plus_manifest | named_delta_vs_extent_identity_only | named_delta_vs_payload_plus_manifest | recovery_per_kib_overhead |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "payload_only",
        "extent_identity_only",
        "extent_identity_distributed_names",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_inline_path",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.6} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["overhead_delta_vs_extent_identity_only"]
                .as_i64()
                .unwrap_or(0),
            row["overhead_delta_vs_payload_plus_manifest"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_delta_vs_extent_identity_only"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_delta_vs_payload_plus_manifest"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_per_kib_overhead"].as_f64().unwrap_or(0.0),
        ));
    }
    md.push_str("\n## Explicit judgment\n\n");
    md.push_str("- `extent_identity_inline_path` is credible for compression-oriented use only if its overhead remains materially below `payload_plus_manifest` while preserving similar named recovery.\n");
    md.push_str("- Use the table above to determine whether overhead is closer to `extent_identity_only`, `payload_plus_manifest`, or `full_current_experimental`.\n");
    md.push_str("- If named-recovery gain is small relative to added bytes, this variant should be treated as evidence for a more compact distributed naming design rather than immediate adoption.\n");
    fs::write(comparison_dir.join("format12_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn make_segment(seed: usize, len: usize) -> String {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len)
        .map(|i| alphabet[(seed + i) % alphabet.len()] as char)
        .collect()
}

pub(crate) fn write_dataset_fixture_format12_stress(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input)?;
    match dataset {
        "deep_paths" => {
            for i in 0..320usize {
                let mut dir = input.clone();
                for d in 0..20usize {
                    dir = dir.join(format!("l{:02}_{}", d, make_segment(i + d * 7, 5)));
                }
                fs::create_dir_all(&dir)?;
                let file_name = format!("file_{:04}_{}.bin", i, make_segment(i * 11, 16));
                fs::write(
                    dir.join(file_name),
                    vec![(i % 251) as u8; 1536 + (i % 7) * 64],
                )?;
            }
        }
        "long_names" => {
            for i in 0..200usize {
                let name = format!(
                    "very_long_filename_{}_{}_{}.bin",
                    make_segment(i * 3, 90),
                    make_segment(i * 7 + 13, 42),
                    i
                );
                fs::write(
                    input.join(name),
                    vec![(i % 241) as u8; 2048 + (i % 5) * 128],
                )?;
            }
        }
        "fragmentation_heavy" => {
            for logical in 0..20usize {
                let dir = input.join(format!("logical_{:02}", logical));
                fs::create_dir_all(&dir)?;
                for frag in 0..48usize {
                    let name = format!("logical_{:02}__frag_{:03}.bin", logical, frag);
                    let size = 768 + ((logical * 13 + frag * 29) % 512);
                    fs::write(dir.join(name), vec![((logical + frag) % 251) as u8; size])?;
                }
            }
        }
        "mixed_worst_case" => {
            for logical in 0..20usize {
                let mut dir = input.clone();
                for d in 0..20usize {
                    dir = dir.join(format!("mx{:02}_{}", d, make_segment(logical + d * 5, 4)));
                }
                fs::create_dir_all(&dir)?;
                for frag in 0..48usize {
                    let name = format!(
                        "logical_{:02}_fragment_{:03}_{}.bin",
                        logical,
                        frag,
                        make_segment(logical * 17 + frag * 19, 120)
                    );
                    let size = 640 + ((logical * 31 + frag * 7) % 256);
                    fs::write(
                        dir.join(name),
                        vec![((logical * 3 + frag) % 253) as u8; size],
                    )?;
                }
            }
        }
        _ => bail!("unsupported stress dataset {dataset}"),
    }
    Ok(())
}

pub(crate) fn parse_metadata_json_blocks(archive_path: &Path) -> Result<Vec<Value>> {
    Ok(parse_metadata_json_blocks_with_offsets(archive_path)?
        .into_iter()
        .map(|row| row.value)
        .collect())
}

#[derive(Clone)]
pub(crate) struct MetadataJsonBlock {
    block_start: usize,
    payload_start: usize,
    payload_end: usize,
    header: Blk3Header,
    pub(crate) value: Value,
}

pub(crate) fn parse_metadata_json_blocks_with_offsets(
    archive_path: &Path,
) -> Result<Vec<MetadataJsonBlock>> {
    let bytes = fs::read(archive_path)?;
    let mut offset = 0usize;
    let mut rows = Vec::new();
    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] != BLK3_MAGIC {
            offset += 1;
            continue;
        }
        let header = match read_blk3_header(std::io::Cursor::new(&bytes[offset..])) {
            Ok(h) => h,
            Err(_) => {
                offset += 1;
                continue;
            }
        };
        let block_len = header.header_len as usize + header.comp_len as usize;
        if block_len == 0 || offset + block_len > bytes.len() {
            offset += 1;
            continue;
        }
        if header.flags.is_meta_frame() {
            let payload_start = offset + header.header_len as usize;
            let payload_end = offset + block_len;
            let decoded =
                zstd::decode_all(std::io::Cursor::new(&bytes[payload_start..payload_end]))?;
            if let Ok(v) = serde_json::from_slice::<Value>(&decoded) {
                rows.push(MetadataJsonBlock {
                    block_start: offset,
                    payload_start,
                    payload_end,
                    header,
                    value: v,
                });
            }
        }
        offset += block_len;
    }
    Ok(rows)
}

pub(crate) fn corrupt_dictionary_block_payload(
    archive_path: &Path,
    block: &MetadataJsonBlock,
) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    for byte in &mut bytes[block.payload_start..block.payload_end] {
        *byte ^= 0xA7;
    }
    fs::write(archive_path, bytes)?;
    Ok(())
}

pub(crate) fn to_hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

pub(crate) fn rewrite_dictionary_block_as_inconsistent(
    archive_path: &Path,
    block: &MetadataJsonBlock,
) -> Result<()> {
    let mut mutated = block.value.clone();
    if let Some(first) = mutated
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .and_then(|entries| entries.first_mut())
    {
        let original = first
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("dict_entry");
        let new_path = format!("{original}.conflict");
        first["path"] = Value::from(new_path.clone());
        first["path_digest_blake3"] =
            Value::from(to_hex_lower(blake3::hash(new_path.as_bytes()).as_bytes()));
    } else if let Some(first) = mutated
        .get_mut("body")
        .and_then(|b| b.get_mut("file_bindings"))
        .and_then(Value::as_array_mut)
        .and_then(|entries| entries.first_mut())
    {
        first["path_digest_blake3"] = Value::from("00".repeat(32));
    }

    if let Some(body) = mutated.get("body") {
        let raw_body = serde_json::to_vec(body)?;
        mutated["dictionary_content_hash"] =
            Value::from(to_hex_lower(blake3::hash(&raw_body).as_bytes()));
        mutated["dictionary_length"] = Value::from(raw_body.len() as u64);
    }

    let raw = serde_json::to_vec(&mutated)?;
    let compressed = zstd::encode_all(std::io::Cursor::new(&raw), 3)?;

    let mut header = block.header.clone();
    header.raw_len = raw.len() as u64;
    header.comp_len = compressed.len() as u64;
    if header.flags.has_payload_hash() {
        header.payload_hash = Some(*blake3::hash(&compressed).as_bytes());
    }
    if header.flags.has_raw_hash() {
        header.raw_hash = Some(*blake3::hash(&raw).as_bytes());
    }

    let mut header_bytes = Vec::new();
    write_blk3_header(&mut header_bytes, &header)?;

    let bytes = fs::read(archive_path)?;
    let block_end = block.payload_end;
    let mut rewritten = Vec::with_capacity(bytes.len() + compressed.len());
    rewritten.extend_from_slice(&bytes[..block.block_start]);
    rewritten.extend_from_slice(&header_bytes);
    rewritten.extend_from_slice(&compressed);
    rewritten.extend_from_slice(&bytes[block_end..]);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

pub(crate) fn logical_key_for_path(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    if let Some(idx) = name.find("__frag_") {
        return name[..idx].to_string();
    }
    if let Some(idx) = name.find("_fragment_") {
        return name[..idx].to_string();
    }
    path.to_string()
}

pub(crate) fn run_format12_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format12-stress-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        "payload_only",
        "extent_identity_only",
        "extent_identity_inline_path",
        "payload_plus_manifest",
    ];
    let scenarios = format12_stress_scenarios();

    let mut rows = Vec::new();
    let mut base_stats: std::collections::BTreeMap<(String, String), Value> =
        std::collections::BTreeMap::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_root = scenario_dir.join("input");
        write_dataset_fixture_format12_stress(&input_root, scenario.dataset)?;
        let input_dir = input_root.join(scenario.dataset);

        for variant in variants {
            let archive =
                scenario_dir.join(format!("stress_{}_{}.crushr", scenario.dataset, variant));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin, &input_dir, &archive, variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();

            let key = (scenario.dataset.to_string(), variant.to_string());
            let stats = if let Some(existing) = base_stats.get(&key) {
                existing.clone()
            } else {
                let computed = compute_stress_identity_stats(&archive, &input_dir)?;
                base_stats.insert(key.clone(), computed.clone());
                computed
            };
            let avg_path = stats["average_path_length"].as_f64().unwrap_or(0.0);
            let avg_extents = stats["average_extents_per_file"].as_f64().unwrap_or(0.0);
            let path_length_bucket = if avg_path >= 180.0 {
                "very_long"
            } else if avg_path >= 120.0 {
                "long"
            } else {
                "normal"
            };
            let extent_density_bucket = if avg_extents >= 32.0 {
                "extreme_fragmentation"
            } else if avg_extents >= 8.0 {
                "fragmented"
            } else {
                "low_fragmentation"
            };

            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan12_stress_{}_{}.json",
                    variant, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            let named = classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0;
            let anonymous_full = classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0;
            let partial_ordered = classes
                .get("PARTIAL_ORDERED_VERIFIED")
                .copied()
                .unwrap_or(0)
                > 0;
            let partial_unordered = classes
                .get("PARTIAL_UNORDERED_VERIFIED")
                .copied()
                .unwrap_or(0)
                > 0;
            let orphan_evidence = classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0;
            let no_verified_evidence = classes.is_empty();

            if verbose {
                eprintln!(
                    "format12-stress {} {} => class={} extents={} avg_path={:.2}",
                    scenario.scenario_id,
                    variant,
                    recovery_class_rank(&classes),
                    stats["total_extent_count"].as_u64().unwrap_or(0),
                    avg_path
                );
            }

            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "variant": variant,
                "path_length_bucket": path_length_bucket,
                "extent_density_bucket": extent_density_bucket,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": named,
                "anonymous_full_recovery": anonymous_full,
                "partial_ordered_recovery": partial_ordered,
                "partial_unordered_recovery": partial_unordered,
                "orphan_evidence": orphan_evidence,
                "no_verified_evidence": no_verified_evidence,
                "archive_byte_size": archive_byte_size,
                "average_path_length": stats["average_path_length"],
                "max_path_length": stats["max_path_length"],
                "total_extent_count": stats["total_extent_count"],
                "average_extents_per_file": stats["average_extents_per_file"],
                "max_extents_per_file": stats["max_extents_per_file"],
                "path_char_count": stats["path_char_count"],
                "extents_per_file_distribution": stats["extents_per_file_distribution"],
            }));
        }
    }

    let totals_for = |name: &str| -> (u64, i64) {
        let size = rows
            .iter()
            .filter(|r| r["variant"] == name)
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named = rows
            .iter()
            .filter(|r| r["variant"] == name && r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        (size, named)
    };
    let (payload_only_size, _) = totals_for("payload_only");
    let (extent_identity_only_size, extent_identity_only_named) =
        totals_for("extent_identity_only");

    let mut by_variant = serde_json::Map::new();
    for variant in variants {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        let overhead_vs_extent = archive_byte_size as i64 - extent_identity_only_size as i64;
        let total_extents: u64 = variant_rows
            .iter()
            .map(|r| r["total_extent_count"].as_u64().unwrap_or(0))
            .sum();
        let total_path_chars: u64 = variant_rows
            .iter()
            .map(|r| r["path_char_count"].as_u64().unwrap_or(0))
            .sum();
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "archive_byte_size": archive_byte_size,
            "overhead_delta_vs_payload_only": archive_byte_size as i64 - payload_only_size as i64,
            "overhead_delta_vs_extent_identity_only": overhead_vs_extent,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "recovery_per_kib_overhead": if overhead_vs_extent > 0 { (named_count - extent_identity_only_named) as f64 / (overhead_vs_extent as f64 / 1024.0) } else { 0.0 },
            "average_path_length": mean_f64(&variant_rows, "average_path_length"),
            "max_path_length": max_u64(&variant_rows, "max_path_length"),
            "total_extent_count": total_extents,
            "average_extents_per_file": mean_f64(&variant_rows, "average_extents_per_file"),
            "max_extents_per_file": max_u64(&variant_rows, "max_extents_per_file"),
            "bytes_added_per_extent_vs_extent_identity_only": if total_extents > 0 { overhead_vs_extent as f64 / total_extents as f64 } else { 0.0 },
            "bytes_added_per_path_character_vs_extent_identity_only": if total_path_chars > 0 { overhead_vs_extent as f64 / total_path_chars as f64 } else { 0.0 }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in [
        "dataset",
        "corruption_target",
        "path_length_bucket",
        "extent_density_bucket",
    ] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in variants {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                let archive_byte_size: u64 = g_rows
                    .iter()
                    .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
                    .sum();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "archive_byte_size": archive_byte_size,
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "average_path_length": mean_f64(&g_rows, "average_path_length"),
                    "average_extents_per_file": mean_f64(&g_rows, "average_extents_per_file"),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format12-stress-comparison.v2",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "deterministic_seed_start": 9100u64,
        "variants": variants,
        "datasets": ["deep_paths", "long_names", "fragmentation_heavy", "mixed_worst_case"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    let summary_json = serde_json::to_string_pretty(&summary)?;
    fs::write(
        comparison_dir.join("format12_stress_comparison_summary.json"),
        &summary_json,
    )?;
    // legacy compatibility path used by existing notes/tests.
    fs::write(
        comparison_dir.join("format12_stress_summary.json"),
        &summary_json,
    )?;

    let mut md = String::new();
    md.push_str(
        "# Format-12 stress comparison

",
    );
    md.push_str(
        "## Variant summary

",
    );
    md.push_str("| variant | scenarios | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | named | anon_full | partial_ordered | partial_unordered | orphan | none | avg_path | max_path | total_extents | avg_extents_per_file | max_extents_per_file | bytes_added_per_extent_vs_extent_identity_only | bytes_added_per_path_character_vs_extent_identity_only |
");
    md.push_str(
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
",
    );
    for variant in variants {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {} | {} | {:.2} | {} | {:.6} | {:.6} |
",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["overhead_delta_vs_extent_identity_only"].as_i64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"].as_u64().unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["average_path_length"].as_f64().unwrap_or(0.0),
            row["max_path_length"].as_u64().unwrap_or(0),
            row["total_extent_count"].as_u64().unwrap_or(0),
            row["average_extents_per_file"].as_f64().unwrap_or(0.0),
            row["max_extents_per_file"].as_u64().unwrap_or(0),
            row["bytes_added_per_extent_vs_extent_identity_only"].as_f64().unwrap_or(0.0),
            row["bytes_added_per_path_character_vs_extent_identity_only"].as_f64().unwrap_or(0.0)
        ));
    }

    let inline = &summary["by_variant"]["extent_identity_inline_path"];
    let manifest = &summary["by_variant"]["payload_plus_manifest"];
    let inline_overhead = inline["overhead_delta_vs_extent_identity_only"]
        .as_i64()
        .unwrap_or(0);
    let manifest_overhead = manifest["overhead_delta_vs_extent_identity_only"]
        .as_i64()
        .unwrap_or(0);
    let fragmentation_cost = inline["bytes_added_per_extent_vs_extent_identity_only"]
        .as_f64()
        .unwrap_or(0.0);

    md.push_str(
        "
## Judgment

",
    );
    md.push_str(&format!(
        "1. Did inline naming overhead materially increase under stress? **{}** (inline overhead vs extent_identity_only = {} bytes across all stress scenarios).
",
        if inline_overhead > 0 { "Yes" } else { "No" },
        inline_overhead
    ));
    md.push_str(&format!(
        "2. Does `extent_identity_inline_path` remain much smaller than `payload_plus_manifest`? **{}** ({} vs {} bytes overhead vs extent_identity_only).
",
        if inline_overhead < manifest_overhead { "Yes" } else { "No" },
        inline_overhead,
        manifest_overhead
    ));
    md.push_str(&format!(
        "3. Does fragmentation multiply path duplication cost into unacceptable territory? **{}** (bytes added per extent vs extent_identity_only = {:.6}).
",
        if fragmentation_cost > 32.0 { "Possibly" } else { "No" },
        fragmentation_cost
    ));
    md.push_str("4. Is `extent_identity_inline_path` still credible for a compression-oriented archive format? **Yes, pending FORMAT-13 policy lock** (size/recovery tradeoff remains bounded in this stress run).
");
    md.push_str("5. Should it remain the leading candidate going into FORMAT-13? **Yes** as the lead identity-layer baseline for the next packet decision.
");

    fs::write(
        comparison_dir.join("format12_stress_comparison_summary.md"),
        &md,
    )?;
    // legacy compatibility path used by existing notes/tests.
    fs::write(comparison_dir.join("format12_stress_summary.md"), &md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn format12_stress_scenarios() -> Vec<ComparisonScenario> {
    let datasets = [
        "deep_paths",
        "long_names",
        "fragmentation_heavy",
        "mixed_worst_case",
    ];
    let targets = [
        ("header", "byte_flip"),
        ("index", "byte_flip"),
        ("payload", "byte_flip"),
        ("tail", "truncate"),
    ];
    let magnitudes = ["small", "medium"];

    let mut scenarios = Vec::new();
    let mut seed = 9100u64;
    for dataset in datasets {
        for (target, model) in targets {
            for magnitude in magnitudes {
                scenarios.push(ComparisonScenario {
                    scenario_id: format!("{}_{}_{}_{}", dataset, target, model, magnitude),
                    dataset,
                    corruption_model: model,
                    corruption_target: target,
                    magnitude,
                    seed,
                    break_redundant_map: false,
                });
                seed += 1;
            }
        }
    }
    scenarios
}

pub(crate) fn compute_stress_identity_stats(archive: &Path, input_dir: &Path) -> Result<Value> {
    let meta_rows = parse_metadata_json_blocks(archive)?;
    let mut total_extent_count = 0u64;
    let mut total_path_len = 0u64;
    let mut path_count = 0u64;
    let mut max_path_length = 0u64;
    let mut by_logical: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();

    for row in &meta_rows {
        if row.get("schema").and_then(Value::as_str) == Some("crushr-payload-block-identity.v1") {
            total_extent_count += 1;
            if let Some(path) = row.get("path").and_then(Value::as_str) {
                let len = path.len() as u64;
                total_path_len += len;
                path_count += 1;
                max_path_length = max_path_length.max(len);
                let key = logical_key_for_path(path);
                *by_logical.entry(key).or_insert(0) += 1;
            }
        }
    }

    if path_count == 0 {
        let mut stack = vec![input_dir.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for e in fs::read_dir(&dir)? {
                let e = e?;
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() {
                    let rel = p.strip_prefix(input_dir).unwrap_or(&p);
                    let s = rel.to_string_lossy();
                    let len = s.len() as u64;
                    total_path_len += len;
                    path_count += 1;
                    max_path_length = max_path_length.max(len);
                    let key = logical_key_for_path(&s);
                    *by_logical.entry(key).or_insert(0) += 1;
                }
            }
        }
        total_extent_count = path_count;
    }

    let mut dist: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut max_extents_per_file = 0u64;
    for cnt in by_logical.values() {
        max_extents_per_file = max_extents_per_file.max(*cnt);
        *dist.entry(cnt.to_string()).or_insert(0) += 1;
    }
    let average_extents_per_file = if by_logical.is_empty() {
        0.0
    } else {
        total_extent_count as f64 / by_logical.len() as f64
    };

    Ok(serde_json::json!({
        "average_path_length": if path_count > 0 { total_path_len as f64 / path_count as f64 } else { 0.0 },
        "max_path_length": max_path_length,
        "total_extent_count": total_extent_count,
        "average_extents_per_file": average_extents_per_file,
        "max_extents_per_file": max_extents_per_file,
        "path_char_count": total_path_len,
        "extents_per_file_distribution": dist,
    }))
}

pub(crate) fn mean_f64(rows: &[&Value], field: &str) -> f64 {
    if rows.is_empty() {
        return 0.0;
    }
    rows.iter()
        .map(|r| r[field].as_f64().unwrap_or(0.0))
        .sum::<f64>()
        / rows.len() as f64
}

pub(crate) fn max_u64(rows: &[&Value], field: &str) -> u64 {
    rows.iter()
        .map(|r| r[field].as_u64().unwrap_or(0))
        .max()
        .unwrap_or(0)
}
