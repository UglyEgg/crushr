// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crushr_format::{
    ftr4::{FTR4_LEN, Ftr4},
    tailframe::{assemble_tail_frame, parse_tail_frame},
};
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

fn build_archive(
    pack_bin: &Path,
    path: &Path,
    experimental: bool,
    file_identity: bool,
    self_identifying_blocks: bool,
    file_manifest_checkpoints: bool,
) {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("data.txt"), b"experimental-resilience-payload").unwrap();

    let mut cmd = Command::new(pack_bin);
    cmd.args([
        input.to_str().unwrap(),
        "-o",
        path.to_str().unwrap(),
        "--level",
        "3",
    ]);
    if experimental {
        cmd.arg("--experimental-self-describing-extents");
    }
    if file_identity {
        cmd.arg("--experimental-file-identity-extents");
    }
    if self_identifying_blocks {
        cmd.arg("--experimental-self-identifying-blocks");
    }
    if file_manifest_checkpoints {
        cmd.arg("--experimental-file-manifest-checkpoints");
    }
    run(&mut cmd);
}

fn rewrite_tail_without_ledger_and_with_corrupt_index(archive_path: &Path) {
    let bytes = fs::read(archive_path).unwrap();
    let footer_off = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&bytes[footer_off..]).unwrap();
    let blocks_end = footer.blocks_end_offset;
    let tail = parse_tail_frame(&bytes[blocks_end as usize..]).unwrap();

    let mut bad_idx = tail.idx3_bytes;
    bad_idx[4] ^= 0x7f;

    let mut rewritten = bytes[..blocks_end as usize].to_vec();
    let rebuilt = assemble_tail_frame(blocks_end, None, &bad_idx, None).unwrap();
    rewritten.extend_from_slice(&rebuilt);
    fs::write(archive_path, rewritten).unwrap();
}

fn run_salvage(salvage_bin: &Path, archive: &Path, out: &Path) -> Value {
    run(Command::new(salvage_bin).args([
        archive.to_str().unwrap(),
        "--json-out",
        out.to_str().unwrap(),
    ]));
    serde_json::from_slice(&fs::read(out).unwrap()).unwrap()
}

#[test]
fn experimental_archive_uses_checkpoint_path_when_primary_and_ledger_are_unusable() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive, true, false, false, false);
    rewrite_tail_without_ledger_and_with_corrupt_index(&archive);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "CHECKPOINT_MAP_PATH"
    );
}

#[test]
fn experimental_comparison_outputs_file_identity_summary() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison");

    run(Command::new(lab_bin)
        .arg("run-experimental-resilience-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format04_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format04_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["old_outcome_counts"].is_object());
    assert!(summary["redundant_outcome_counts"].is_object());
    assert!(summary["experimental_outcome_counts"].is_object());
    assert!(summary["file_identity_outcome_counts"].is_object());
    assert!(summary["no_evidence_to_partial_improvements_vs_old"].is_number());
    assert!(summary["no_evidence_to_full_improvements_vs_old"].is_number());
}

#[test]
fn format04_comparison_command_is_invokable() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison");

    run(Command::new(lab_bin)
        .arg("run-format04-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format04_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format04_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["file_identity_outcome_counts"].is_object());
    assert!(summary["no_evidence_to_partial_improvements_vs_old"].is_number());
    assert!(summary["no_evidence_to_full_improvements_vs_old"].is_number());
}

#[test]
fn file_identity_archive_uses_file_identity_path_when_primary_and_ledger_are_unusable() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive-file-identity.crushr");

    build_archive(pack_bin, &archive, false, true, false, false);
    rewrite_tail_without_ledger_and_with_corrupt_index(&archive);

    let plan = run_salvage(
        salvage_bin,
        &archive,
        &td.path().join("plan-file-identity.json"),
    );
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "FILE_IDENTITY_EXTENT_PATH"
    );
}

#[test]
fn file_identity_archive_recovers_via_bootstrap_scan_when_tail_is_truncated() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive-truncated-tail.crushr");

    build_archive(pack_bin, &archive, false, true, false, false);
    let mut bytes = fs::read(&archive).unwrap();
    let new_len = bytes.len().saturating_sub(96);
    bytes.truncate(new_len);
    bytes[0] ^= 0x10;
    fs::write(&archive, bytes).unwrap();

    let plan = run_salvage(
        salvage_bin,
        &archive,
        &td.path().join("plan-truncated-tail.json"),
    );
    assert_eq!(plan["footer_analysis"]["status"], "invalid");
    assert_eq!(plan["bootstrap_anchor_analysis"]["status"], "available");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "FILE_IDENTITY_EXTENT_PATH"
    );
}

#[test]
fn format05_archive_recovers_via_payload_block_identity_when_index_is_unusable() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive-format05.crushr");

    build_archive(pack_bin, &archive, false, false, true, false);
    rewrite_tail_without_ledger_and_with_corrupt_index(&archive);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan-format05.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "PAYLOAD_BLOCK_IDENTITY_PATH"
    );
}

#[test]
fn format05_comparison_command_is_invokable() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format05");

    run(Command::new(lab_bin)
        .arg("run-format05-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format05_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format05_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["format05_outcome_counts"].is_object());
    assert!(summary["orphan_to_partial_improvements_vs_old"].is_number());
    assert!(summary["orphan_to_full_improvements_vs_old"].is_number());
    assert!(summary["no_evidence_to_partial_improvements_vs_old"].is_number());
    assert!(summary["no_evidence_to_full_improvements_vs_old"].is_number());
}

#[test]
fn format06_archive_uses_manifest_path_when_primary_and_ledger_are_unusable() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive-format06.crushr");

    build_archive(pack_bin, &archive, false, false, true, true);
    rewrite_tail_without_ledger_and_with_corrupt_index(&archive);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan-format06.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "FILE_MANIFEST_PATH"
    );
    assert_eq!(
        plan["file_plans"][0]["recovery_classification"],
        "FULL_VERIFIED"
    );
}

#[test]
fn format06_comparison_command_reports_recovery_classification_deltas() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format06");

    run(Command::new(lab_bin)
        .arg("run-format06-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format06_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format06_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["format05_recovery_classification_counts"].is_object());
    assert!(summary["format06_recovery_classification_counts"].is_object());
    assert!(
        summary["recovery_classification_delta_vs_format05"]["full_verified_delta"].is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format05"]["full_anonymous_delta"].is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format05"]["partial_ordered_delta"].is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format05"]["partial_unordered_delta"].is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format05"]["orphan_blocks_delta"].is_number()
    );
}

#[test]
fn format07_comparison_command_reports_graph_classification_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format07");

    run(Command::new(lab_bin)
        .arg("run-format07-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format07_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format07_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["format07_outcome_counts"].is_object());
    assert!(summary["format07_recovery_classification_counts"].is_object());
    assert!(
        summary["recovery_classification_delta_vs_format06"]["full_named_verified_delta"]
            .is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format06"]["full_anonymous_verified_delta"]
            .is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format06"]["partial_ordered_verified_delta"]
            .is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format06"]["partial_unordered_verified_delta"]
            .is_number()
    );
    assert!(
        summary["recovery_classification_delta_vs_format06"]["orphan_evidence_only_delta"]
            .is_number()
    );
}

#[test]
fn pack_help_lists_format05_flag() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(pack_bin)
        .arg("--help")
        .output()
        .expect("run --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--experimental-self-identifying-blocks"));
}

#[test]
fn pack_accepts_format05_flag_and_emits_archive() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("a.txt"), b"format05-writer").unwrap();
    let archive = td.path().join("archive.crushr");

    run(Command::new(pack_bin)
        .arg(&input)
        .arg("-o")
        .arg(&archive)
        .arg("--level")
        .arg("3")
        .arg("--experimental-self-identifying-blocks"));

    assert!(archive.exists());
    assert!(fs::metadata(archive).unwrap().len() > 0);
}

#[test]
fn format05_comparison_succeeds_when_pack_help_is_unsupported() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format05-no-help");
    let shim_path = td.path().join("pack-no-help-shim.sh");

    fs::write(
        &shim_path,
        format!(
            "#!/usr/bin/env bash
set -euo pipefail
if [[ \"${{1:-}}\" == \"--help\" ]]; then
  echo \"unsupported flag: --help\" >&2
  exit 1
fi
out=\"\"
has_format05=0
prev=\"\"
for arg in \"$@\"; do
  if [[ \"$arg\" == \"--experimental-self-identifying-blocks\" ]]; then
    has_format05=1
  fi
  if [[ \"$prev\" == \"-o\" || \"$prev\" == \"--output\" ]]; then
    out=\"$arg\"
  fi
  prev=\"$arg\"
done
if [[ \"$out\" == *\"_format05.crushr\" ]] && [[ $has_format05 -ne 1 ]]; then
  echo \"missing required format05 flag\" >&2
  exit 97
fi
exec \"{}\" \"$@\"
",
            pack_bin.display()
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_path, perms).unwrap();
    }

    run(Command::new(lab_bin)
        .arg("run-format05-comparison")
        .arg("--output")
        .arg(&out_dir)
        .env("CRUSHR_PACK_BIN", &shim_path));

    let summary_path = out_dir.join("format05_comparison_summary.json");
    assert!(summary_path.exists());
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
}

#[test]
fn format08_comparison_command_reports_required_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format08");

    run(Command::new(lab_bin)
        .arg("run-format08-placement-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format08_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format08_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["placement_strategies"].as_array().unwrap().len(), 3);
    let by = summary["by_placement_strategy"].as_object().unwrap();
    assert!(by.contains_key("fixed_spread"));
    assert!(by.contains_key("hash_spread"));
    assert!(by.contains_key("golden_spread"));
    for key in ["fixed_spread", "hash_spread", "golden_spread"] {
        let row = &by[key];
        assert!(row["recovery_outcome_counts"].is_object());
        assert!(row["recovery_classification_counts"].is_object());
        assert!(row["manifest_checkpoint_survival_count"].is_u64());
        assert!(row["path_checkpoint_survival_count"].is_u64());
        assert!(row["verified_metadata_node_count"].is_u64());
    }
}

#[test]
fn format08_comparison_command_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("placeholder-input")
        .arg("run-format08-placement-comparison")
        .arg("--output")
        .arg("/tmp/nowhere")
        .output()
        .expect("run misplaced command");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unsupported argument")
            || stderr.contains("usage:")
    );
}

#[test]
fn format07_comparison_command_still_dispatches_after_format08() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format07-regression");

    run(Command::new(lab_bin)
        .arg("run-format07-comparison")
        .arg("--output")
        .arg(&out_dir));

    assert!(out_dir.join("format07_comparison_summary.json").exists());
    assert!(out_dir.join("format07_comparison_summary.md").exists());
}

#[test]
fn format09_comparison_command_reports_required_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format09");

    run(Command::new(lab_bin)
        .arg("run-format09-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format09_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format09_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert!(summary["scenario_count"].as_u64().unwrap_or(0) >= 24);
    assert!(summary["metadata_survival_statistics"].is_object());
    assert!(summary["metadata_recovery_gain_distribution"].is_object());

    let rows = summary["per_scenario_rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    let first = &rows[0];
    assert!(first["strategy"].is_string());
    assert!(first["scenario_id"].is_string());
    assert!(first["metadata_regime"].is_string());
    assert!(first["metadata_target"].is_string());
    assert!(first["payload_damage"].is_string());
    assert!(first["named_recovery"].is_boolean());
    assert!(first["anonymous_full_recovery"].is_boolean());
    assert!(first["partial_ordered_recovery"].is_boolean());
    assert!(first["partial_unordered_recovery"].is_boolean());
    assert!(first["orphan_evidence"].is_boolean());
    assert!(first["manifest_checkpoint_survival_count"].is_u64());
    assert!(first["path_checkpoint_survival_count"].is_u64());
    assert!(first["verified_metadata_node_count"].is_u64());
    assert!(first["metadata_recovery_gain"].is_string());
}

#[test]
fn format09_comparison_command_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("placeholder-input")
        .arg("run-format09-comparison")
        .arg("--output")
        .arg("/tmp/nowhere")
        .output()
        .expect("run misplaced command");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unsupported argument")
            || stderr.contains("usage:")
    );
}

#[test]
fn format10_pruning_comparison_command_reports_required_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format10");

    run(Command::new(lab_bin)
        .arg("run-format10-pruning-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format10_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format10_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let variants = summary["variants"].as_array().unwrap();
    assert!(variants.iter().any(|v| v == "payload_only"));
    assert!(variants.iter().any(|v| v == "payload_plus_manifest"));
    assert!(variants.iter().any(|v| v == "payload_plus_path"));
    assert!(variants.iter().any(|v| v == "full_current_experimental"));

    let by = summary["by_variant"].as_object().unwrap();
    for key in [
        "payload_only",
        "payload_plus_manifest",
        "payload_plus_path",
        "full_current_experimental",
    ] {
        let row = &by[key];
        assert!(row["recovery_outcome_counts"].is_object());
        assert!(row["recovery_classification_counts"].is_object());
        assert!(row["named_recovery_count"].is_u64());
        assert!(row["anonymous_full_recovery_count"].is_u64());
        assert!(row["partial_ordered_recovery_count"].is_u64());
        assert!(row["partial_unordered_recovery_count"].is_u64());
        assert!(row["orphan_evidence_count"].is_u64());
        assert!(row["no_verified_evidence_count"].is_u64());
        assert!(row["archive_byte_size"].is_u64());
        assert!(row["metadata_byte_estimate"].is_u64());
        assert!(row["overhead_delta_vs_payload_only"].is_i64());
        assert!(row["recovery_delta_vs_full_current_experimental"].is_object());
    }

    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
}

#[test]
fn format10_pruning_comparison_command_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("placeholder-input")
        .arg("run-format10-pruning-comparison")
        .arg("--output")
        .arg("/tmp/nowhere")
        .output()
        .expect("run misplaced command");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unsupported argument")
            || stderr.contains("usage:")
    );
}

#[test]
fn format09_comparison_command_still_dispatches_after_format10() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format09-regression");

    run(Command::new(lab_bin)
        .arg("run-format09-comparison")
        .arg("--output")
        .arg(&out_dir));

    assert!(out_dir.join("format09_comparison_summary.json").exists());
    assert!(out_dir.join("format09_comparison_summary.md").exists());
}

#[test]
fn format11_extent_identity_comparison_command_reports_required_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format11");

    run(Command::new(lab_bin)
        .arg("run-format11-extent-identity-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format11_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format11_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let variants = summary["variants"].as_array().unwrap();
    assert!(variants.iter().any(|v| v == "payload_only"));
    assert!(variants.iter().any(|v| v == "payload_plus_manifest"));
    assert!(variants.iter().any(|v| v == "full_current_experimental"));
    assert!(variants.iter().any(|v| v == "extent_identity_only"));

    let by = summary["by_variant"].as_object().unwrap();
    for key in [
        "payload_only",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_only",
    ] {
        let row = &by[key];
        assert!(row["named_recovery_count"].is_i64() || row["named_recovery_count"].is_u64());
        assert!(row["anonymous_full_recovery_count"].is_u64());
        assert!(row["partial_ordered_recovery_count"].is_u64());
        assert!(row["partial_unordered_recovery_count"].is_u64());
        assert!(row["orphan_evidence_count"].is_u64());
        assert!(row["no_verified_evidence_count"].is_u64());
        assert!(row["archive_byte_size"].is_u64());
        assert!(row["overhead_delta_vs_payload_only"].is_i64());
        assert!(row["recovery_delta_vs_payload_plus_manifest"].is_object());
    }

    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
}

#[test]
fn format11_comparison_command_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("placeholder-input")
        .arg("run-format11-extent-identity-comparison")
        .arg("--output")
        .arg("/tmp/nowhere")
        .output()
        .expect("run misplaced command");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unsupported argument")
            || stderr.contains("usage:")
    );
}

#[test]
fn format10_comparison_command_still_dispatches_after_format11() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format10-regression");

    run(Command::new(lab_bin)
        .arg("run-format10-pruning-comparison")
        .arg("--output")
        .arg(&out_dir));

    assert!(out_dir.join("format10_comparison_summary.json").exists());
    assert!(out_dir.join("format10_comparison_summary.md").exists());
}

#[test]
fn format12_stress_comparison_command_reports_required_fields() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format12-stress");

    run(Command::new(lab_bin)
        .arg("run-format12-stress-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format12_stress_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(
        out_dir
            .join("format12_stress_comparison_summary.md")
            .exists()
    );

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let variants = summary["variants"].as_array().unwrap();
    assert!(variants.iter().any(|v| v == "payload_only"));
    assert!(variants.iter().any(|v| v == "extent_identity_only"));
    assert!(variants.iter().any(|v| v == "extent_identity_inline_path"));
    assert!(variants.iter().any(|v| v == "payload_plus_manifest"));

    let by = summary["by_variant"].as_object().unwrap();
    let first = &by["extent_identity_inline_path"];
    assert!(first["scenario_count"].is_u64());
    assert!(first["archive_byte_size"].is_u64());
    assert!(first["overhead_delta_vs_payload_only"].is_i64());
    assert!(first["overhead_delta_vs_extent_identity_only"].is_i64());
    assert!(first["named_recovery_count"].is_i64() || first["named_recovery_count"].is_u64());
    assert!(first["anonymous_full_recovery_count"].is_u64());
    assert!(first["partial_ordered_recovery_count"].is_u64());
    assert!(first["partial_unordered_recovery_count"].is_u64());
    assert!(first["orphan_evidence_count"].is_u64());
    assert!(first["no_verified_evidence_count"].is_u64());
    assert!(first["recovery_per_kib_overhead"].is_f64());
    assert!(first["average_path_length"].is_f64());
    assert!(first["max_path_length"].is_u64());
    assert!(first["total_extent_count"].is_u64());
    assert!(first["average_extents_per_file"].is_f64());
    assert!(first["max_extents_per_file"].is_u64());
    assert!(first["bytes_added_per_extent_vs_extent_identity_only"].is_f64());
    assert!(first["bytes_added_per_path_character_vs_extent_identity_only"].is_f64());

    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
    assert!(grouped.contains_key("path_length_bucket"));
    assert!(grouped.contains_key("extent_density_bucket"));
}

#[test]
fn format12_stress_datasets_exceed_normal_path_and_extent_visibility() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format12-stress-visibility");

    run(Command::new(lab_bin)
        .arg("run-format12-stress-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format12_stress_comparison_summary.json");
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let by = summary["by_variant"].as_object().unwrap();

    let inline = &by["extent_identity_inline_path"];
    assert!(inline["average_path_length"].as_f64().unwrap_or(0.0) > 120.0);
    assert!(inline["max_path_length"].as_u64().unwrap_or(0) >= 180);
    assert!(inline["average_extents_per_file"].as_f64().unwrap_or(0.0) >= 8.0);
    assert!(inline["max_extents_per_file"].as_u64().unwrap_or(0) >= 24);
}

#[test]
fn format12_stress_comparison_command_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("placeholder-input")
        .arg("run-format12-stress-comparison")
        .arg("--output")
        .arg("/tmp/nowhere")
        .output()
        .expect("run misplaced command");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unsupported argument")
            || stderr.contains("usage:")
    );
}

#[test]
fn format13_comparison_command_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format13");

    run(Command::new(lab_bin)
        .arg("run-format13-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format13_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format13_comparison_summary.md").exists());
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let variants = summary["variants"].as_array().unwrap();
    for v in [
        "payload_only",
        "extent_identity_inline_path",
        "extent_identity_path_dict_single",
        "extent_identity_path_dict_header_tail",
        "extent_identity_path_dict_quasi_uniform",
        "payload_plus_manifest",
    ] {
        assert!(variants.iter().any(|x| x == v));
        let row = &summary["by_variant"][v];
        assert!(row["dictionary_entry_count"].is_u64());
        assert!(row["dictionary_total_bytes"].is_u64());
        assert!(row["number_of_dictionary_copies"].is_u64());
    }
}

#[test]
fn format13_stress_comparison_command_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format13-stress");

    run(Command::new(lab_bin)
        .arg("run-format13-stress-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format13_stress_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(
        out_dir
            .join("format13_stress_comparison_summary.md")
            .exists()
    );
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
    assert!(grouped.contains_key("path_length_bucket"));
    assert!(grouped.contains_key("extent_density_bucket"));
}

#[test]
fn format14a_dictionary_resilience_comparison_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format14a");

    run(Command::new(lab_bin)
        .arg("run-format14a-dictionary-resilience-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format14a_dictionary_resilience_summary.json");
    assert!(summary_path.exists());
    assert!(
        out_dir
            .join("format14a_dictionary_resilience_summary.md")
            .exists()
    );
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();

    for v in [
        "extent_identity_inline_path",
        "extent_identity_path_dict_single",
        "extent_identity_path_dict_header_tail",
        "payload_plus_manifest",
    ] {
        let row = &summary["by_variant"][v];
        assert!(row["successful_named_recovery_with_primary_dictionary_loss"].is_u64());
        assert!(row["successful_named_recovery_with_mirror_dictionary_loss"].is_u64());
        assert!(row["successful_named_recovery_with_both_dictionary_losses"].is_u64());
        assert!(row["anonymous_fallback_with_primary_dictionary_loss"].is_u64());
        assert!(row["anonymous_fallback_with_both_dictionary_losses"].is_u64());
        assert!(row["conflict_fail_closed_count"].is_u64());
        assert!(row["dictionary_conflict_detected_count"].is_u64());
    }
}

#[test]
fn format14a_dictionary_resilience_stress_comparison_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format14a-stress");

    run(Command::new(lab_bin)
        .arg("run-format14a-dictionary-resilience-stress-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format14a_dictionary_resilience_stress_summary.json");
    assert!(summary_path.exists());
    assert!(
        out_dir
            .join("format14a_dictionary_resilience_stress_summary.md")
            .exists()
    );
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
    assert!(grouped.contains_key("stress"));
}

#[test]
fn format14a_dictionary_resilience_classification_and_fail_closed_semantics() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format14a-semantics");

    run(Command::new(lab_bin)
        .arg("run-format14a-dictionary-resilience-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format14a_dictionary_resilience_summary.json");
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();

    let rows = summary["per_scenario_rows"].as_array().unwrap();
    assert!(!rows.is_empty());

    for row in rows {
        let terminal_count = [
            row["named_recovery"].as_bool().unwrap_or(false),
            row["anonymous_full_recovery"].as_bool().unwrap_or(false),
            row["partial_ordered_recovery"].as_bool().unwrap_or(false),
            row["partial_unordered_recovery"].as_bool().unwrap_or(false),
            row["orphan_evidence"].as_bool().unwrap_or(false),
            row["no_verified_evidence"].as_bool().unwrap_or(false),
        ]
        .iter()
        .filter(|b| **b)
        .count();
        assert_eq!(terminal_count, 1, "row must classify exactly once: {row}");
    }

    let by_variant = summary["by_variant"].as_object().unwrap();

    let inline = &by_variant["extent_identity_inline_path"];
    assert!(inline["named_recovery_count"].as_u64().unwrap_or(0) > 0);

    let single = &by_variant["extent_identity_path_dict_single"];
    assert_eq!(
        single["successful_named_recovery_with_primary_dictionary_loss"]
            .as_u64()
            .unwrap_or(0),
        0
    );
    assert!(
        single["anonymous_fallback_with_primary_dictionary_loss"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );

    let header_tail = &by_variant["extent_identity_path_dict_header_tail"];
    assert!(
        header_tail["anonymous_fallback_with_both_dictionary_losses"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    let conflict_count = header_tail["dictionary_conflict_detected_count"]
        .as_u64()
        .unwrap_or(0);
    let fail_closed_count = header_tail["conflict_fail_closed_count"]
        .as_u64()
        .unwrap_or(0);
    assert!(conflict_count > 0);
    assert_eq!(fail_closed_count, conflict_count);
}

#[test]
fn format15_comparison_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format15");

    run(Command::new(lab_bin)
        .arg("run-format15-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format15_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("format15_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let row = &summary["by_variant"]["extent_identity_path_dict_factored_header_tail"];
    assert!(row["dictionary_total_bytes"].is_u64());
    assert!(row["directory_dictionary_bytes"].is_u64());
    assert!(row["basename_dictionary_bytes"].is_u64());
    assert!(row["file_binding_table_bytes"].is_u64());
    assert!(row["valid_dictionary_copy_count"].is_u64());
    assert!(row["rejected_hash_mismatch_count"].is_u64());
}

#[test]
fn format15_stress_comparison_writes_required_artifacts() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison-format15-stress");

    run(Command::new(lab_bin)
        .arg("run-format15-stress-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("format15_stress_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(
        out_dir
            .join("format15_stress_comparison_summary.md")
            .exists()
    );

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let grouped = summary["grouped_breakdown"].as_object().unwrap();
    assert!(grouped.contains_key("dataset"));
    assert!(grouped.contains_key("corruption_target"));
    assert!(grouped.contains_key("path_length_bucket"));
    assert!(grouped.contains_key("stress"));
}
