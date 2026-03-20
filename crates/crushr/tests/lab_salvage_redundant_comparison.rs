// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("run command");
    if !out.status.success() {
        panic!(
            "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn redundant_map_comparison_produces_required_outputs_and_metrics() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison");

    run(Command::new(lab_bin)
        .arg("run-redundant-map-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_json = out_dir.join("comparison_summary.json");
    let summary_md = out_dir.join("comparison_summary.md");
    assert!(summary_json.exists());
    assert!(summary_md.exists());

    let summary = read_json(&summary_json);
    assert_eq!(summary["scenario_count"], 24);
    assert_eq!(summary["old_archive_count"], 24);
    assert_eq!(summary["new_archive_count"], 24);

    let rows = summary["per_scenario_rows"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    let mut sorted = rows.clone();
    sorted.sort_by(|a, b| a["scenario_id"].as_str().cmp(&b["scenario_id"].as_str()));
    assert_eq!(*rows, sorted);

    let allowed = [
        "IMPROVED_ORPHAN_TO_PARTIAL",
        "IMPROVED_ORPHAN_TO_FULL",
        "IMPROVED_NONE_TO_PARTIAL",
        "IMPROVED_NONE_TO_FULL",
        "UNCHANGED",
        "DEGRADED",
        "IMPROVED_OTHER",
    ];
    for row in rows {
        assert!(allowed.contains(&row["improvement_class"].as_str().unwrap()));
    }

    let datasets = summary["by_dataset"].as_array().unwrap();
    assert_eq!(datasets.len(), 3);
    let targets = summary["by_corruption_target"].as_array().unwrap();
    assert_eq!(targets.len(), 4);
    let magnitudes = summary["by_magnitude"].as_array().unwrap();
    assert_eq!(magnitudes.len(), 2);

    let unchanged = summary["unchanged_outcome_count"].as_u64().unwrap();
    assert!(
        unchanged >= 1,
        "must include strict-boundary unchanged controls"
    );
}

#[test]
fn redundant_map_comparison_is_deterministic_for_row_order_and_aggregates() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();

    let out_a = td.path().join("a");
    let out_b = td.path().join("b");

    run(Command::new(lab_bin)
        .arg("run-redundant-map-comparison")
        .arg("--output")
        .arg(&out_a));
    run(Command::new(lab_bin)
        .arg("run-redundant-map-comparison")
        .arg("--output")
        .arg(&out_b));

    let a = read_json(&out_a.join("comparison_summary.json"));
    let b = read_json(&out_b.join("comparison_summary.json"));

    assert_eq!(a["per_scenario_rows"], b["per_scenario_rows"]);
    assert_eq!(a["old_outcome_counts"], b["old_outcome_counts"]);
    assert_eq!(a["new_outcome_counts"], b["new_outcome_counts"]);
    assert_eq!(a["by_dataset"], b["by_dataset"]);
    assert_eq!(a["by_corruption_target"], b["by_corruption_target"]);
}

#[test]
fn redundant_map_comparison_aggregate_deltas_match_rows() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison");

    run(Command::new(lab_bin)
        .arg("run-redundant-map-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary = read_json(&out_dir.join("comparison_summary.json"));
    let rows = summary["per_scenario_rows"].as_array().unwrap();

    let verified_delta: i64 = rows
        .iter()
        .map(|row| {
            row["new_verified_block_count"].as_i64().unwrap()
                - row["old_verified_block_count"].as_i64().unwrap()
        })
        .sum();
    let salvageable_delta: i64 = rows
        .iter()
        .map(|row| {
            row["new_salvageable_file_count"].as_i64().unwrap()
                - row["old_salvageable_file_count"].as_i64().unwrap()
        })
        .sum();
    let full_delta: i64 = rows
        .iter()
        .map(|row| {
            row["new_exported_full_file_count"].as_i64().unwrap()
                - row["old_exported_full_file_count"].as_i64().unwrap()
        })
        .sum();

    assert_eq!(
        summary["total_verified_block_delta"].as_i64().unwrap(),
        verified_delta
    );
    assert_eq!(
        summary["total_salvageable_file_delta"].as_i64().unwrap(),
        salvageable_delta
    );
    assert_eq!(
        summary["total_exported_full_file_delta"].as_i64().unwrap(),
        full_delta
    );
}
