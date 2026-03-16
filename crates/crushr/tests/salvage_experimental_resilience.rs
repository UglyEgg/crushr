use crushr_format::{
    ftr4::{Ftr4, FTR4_LEN},
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
        "FULL_NAMED_VERIFIED"
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
    assert!(summary["recovery_classification_delta_vs_format06"]
        ["partial_unordered_verified_delta"]
        .is_number());
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
