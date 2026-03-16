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

fn build_archive(pack_bin: &Path, input_name: &str, payload: &str, output: &Path) {
    let td = TempDir::new().unwrap();
    let input = td.path().join(input_name);
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("data.txt"), payload.as_bytes()).unwrap();

    run(Command::new(pack_bin).args([
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--level",
        "3",
    ]));
}

fn run_harness(
    lab_bin: &Path,
    salvage_bin: &Path,
    input_dir: &Path,
    output_dir: &Path,
    export: bool,
) {
    let mut cmd = Command::new(lab_bin);
    cmd.arg(input_dir)
        .arg("--output")
        .arg(output_dir)
        .env("CRUSHR_SALVAGE_BIN", salvage_bin);
    if export {
        cmd.arg("--export-fragments");
    }
    run(&mut cmd);
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn run_harness_expect_fail(cmd: &mut Command) -> String {
    let out = cmd.output().expect("run command");
    assert!(!out.status.success(), "expected command failure");
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn help_lists_supported_comparison_commands() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let out = Command::new(lab_bin)
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("run-experimental-resilience-comparison"));
    assert!(stdout.contains("run-file-identity-comparison"));
    assert!(stdout.contains("run-format04-comparison"));
    assert!(stdout.contains("run-format05-comparison"));
    assert!(stdout.contains("run-format06-comparison"));
    assert!(stdout.contains("run-redundant-map-comparison"));
}

#[test]
fn known_subcommand_name_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let mut cmd = Command::new(lab_bin);
    cmd.arg(td.path())
        .arg("run-file-identity-comparison")
        .arg("--output")
        .arg(td.path().join("out"));
    let stderr = run_harness_expect_fail(&mut cmd);

    assert!(stderr.contains("must be used as the first argument"));
}

#[test]
fn format06_subcommand_name_is_not_treated_as_input_path() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let mut cmd = Command::new(lab_bin);
    cmd.arg(td.path())
        .arg("run-format06-comparison")
        .arg("--output")
        .arg(td.path().join("out"));
    let stderr = run_harness_expect_fail(&mut cmd);

    assert!(stderr.contains("must be used as the first argument"));
}

#[test]
fn harness_generates_manifest_and_summary_outputs() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "a", "alpha", &input_dir.join("b.crushr"));
    build_archive(pack_bin, "b", "beta", &input_dir.join("a.crushr"));

    let output_dir = td.path().join("experiment");
    run_harness(lab_bin, salvage_bin, &input_dir, &output_dir, false);

    assert!(output_dir.join("experiment_manifest.json").exists());
    assert!(output_dir.join("summary.json").exists());
    assert!(output_dir.join("summary.md").exists());
    assert!(output_dir.join("analysis.json").exists());
    assert!(output_dir.join("analysis.md").exists());

    let manifest = read_json(&output_dir.join("experiment_manifest.json"));
    let summary = read_json(&output_dir.join("summary.json"));
    assert_eq!(manifest["run_count"], 2);
    assert_eq!(summary["run_count"], 2);
    assert_eq!(summary["verification_label"], "UNVERIFIED_RESEARCH_OUTPUT");

    let archive_list = manifest["archive_list"].as_array().unwrap();
    assert_eq!(archive_list.len(), 2);

    let runs_dir = output_dir.join("runs");
    let mut totals = (0u64, 0u64, 0u64, 0u64);
    for archive_id in archive_list {
        let run_dir = runs_dir.join(archive_id.as_str().unwrap());
        assert!(run_dir.join("salvage_plan.json").exists());
        assert!(run_dir.join("run_metadata.json").exists());
        assert!(!run_dir.join("exported_artifacts").exists());

        let metadata = read_json(&run_dir.join("run_metadata.json"));
        totals.0 += metadata["verified_block_count"].as_u64().unwrap();
        totals.1 += metadata["salvageable_file_count"].as_u64().unwrap();
        totals.2 += metadata["unsalvageable_file_count"].as_u64().unwrap();
        totals.3 += metadata["unmappable_file_count"].as_u64().unwrap();
        assert_eq!(metadata["exported_artifact_count"], 0);
        assert_eq!(metadata["exported_block_artifact_count"], 0);
        assert_eq!(metadata["exported_extent_artifact_count"], 0);
        assert_eq!(metadata["exported_full_file_artifact_count"], 0);
    }

    assert_eq!(summary["total_verified_blocks"].as_u64().unwrap(), totals.0);
    assert_eq!(
        summary["total_salvageable_files"].as_u64().unwrap(),
        totals.1
    );
    assert_eq!(
        summary["total_unsalvageable_files"].as_u64().unwrap(),
        totals.2
    );
    assert_eq!(
        summary["total_unmappable_files"].as_u64().unwrap(),
        totals.3
    );
}

#[test]
fn harness_summary_order_is_deterministic() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "x", "one", &input_dir.join("z.crushr"));
    build_archive(pack_bin, "y", "two", &input_dir.join("a.crushr"));

    let out_one = td.path().join("experiment1");
    let out_two = td.path().join("experiment2");
    run_harness(lab_bin, salvage_bin, &input_dir, &out_one, false);
    run_harness(lab_bin, salvage_bin, &input_dir, &out_two, false);

    let first = read_json(&out_one.join("summary.json"));
    let second = read_json(&out_two.join("summary.json"));
    assert_eq!(first["runs"], second["runs"]);

    let first_analysis = read_json(&out_one.join("analysis.json"));
    let second_analysis = read_json(&out_two.join("analysis.json"));
    assert_eq!(
        first_analysis["outcome_groups"],
        second_analysis["outcome_groups"]
    );
    assert_eq!(
        first_analysis["profile_groups"],
        second_analysis["profile_groups"]
    );
}

#[test]
fn harness_export_toggle_controls_export_totals() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "x", "payload", &input_dir.join("only.crushr"));

    let out_disabled = td.path().join("disabled");
    run_harness(lab_bin, salvage_bin, &input_dir, &out_disabled, false);
    let disabled_summary = read_json(&out_disabled.join("summary.json"));
    assert_eq!(disabled_summary["total_exported_block_artifacts"], 0);
    assert_eq!(disabled_summary["total_exported_extent_artifacts"], 0);
    assert_eq!(disabled_summary["total_exported_full_file_artifacts"], 0);

    let out_enabled = td.path().join("enabled");
    run_harness(lab_bin, salvage_bin, &input_dir, &out_enabled, true);
    let enabled_manifest = read_json(&out_enabled.join("experiment_manifest.json"));
    assert_eq!(enabled_manifest["export_fragments_enabled"], true);
    let enabled_summary = read_json(&out_enabled.join("summary.json"));
    assert!(
        enabled_summary["total_exported_block_artifacts"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        enabled_summary["total_exported_extent_artifacts"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        enabled_summary["total_exported_full_file_artifacts"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn resummarize_regenerates_summary_without_rerunning_salvage() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "x", "payload", &input_dir.join("only.crushr"));
    let output_dir = td.path().join("experiment");
    run_harness(lab_bin, salvage_bin, &input_dir, &output_dir, true);

    let manifest = read_json(&output_dir.join("experiment_manifest.json"));
    let run_id = manifest["archive_list"][0].as_str().unwrap();
    let run_dir = output_dir.join("runs").join(run_id);

    fs::remove_file(output_dir.join("summary.json")).unwrap();
    fs::remove_file(output_dir.join("summary.md")).unwrap();
    fs::remove_file(output_dir.join("analysis.json")).unwrap();
    fs::remove_file(output_dir.join("analysis.md")).unwrap();
    fs::write(run_dir.join("salvage_plan.json"), b"{ not valid json }").unwrap();

    run(Command::new(lab_bin).arg("--resummarize").arg(&output_dir));

    assert!(output_dir.join("summary.json").exists());
    assert!(output_dir.join("summary.md").exists());
    assert!(output_dir.join("analysis.json").exists());
    assert!(output_dir.join("analysis.md").exists());
}

#[test]
fn resummarize_classifies_required_outcomes() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let experiment_dir = td.path().join("experiment");
    let runs_dir = experiment_dir.join("runs");
    fs::create_dir_all(&runs_dir).unwrap();

    let archive_ids = ["a", "b", "c", "d"];
    let metadata = [
        serde_json::json!({
            "archive_path": "a.crushr",
            "archive_fingerprint": "f1",
            "verified_block_count": 0,
            "salvageable_file_count": 0,
            "unsalvageable_file_count": 1,
            "unmappable_file_count": 0,
            "exported_block_artifact_count": 0,
            "exported_extent_artifact_count": 0,
            "exported_full_file_artifact_count": 0
        }),
        serde_json::json!({
            "archive_path": "b.crushr",
            "archive_fingerprint": "f2",
            "verified_block_count": 1,
            "salvageable_file_count": 0,
            "unsalvageable_file_count": 0,
            "unmappable_file_count": 2,
            "exported_block_artifact_count": 1,
            "exported_extent_artifact_count": 0,
            "exported_full_file_artifact_count": 0
        }),
        serde_json::json!({
            "archive_path": "c.crushr",
            "archive_fingerprint": "f3",
            "verified_block_count": 2,
            "salvageable_file_count": 1,
            "unsalvageable_file_count": 0,
            "unmappable_file_count": 0,
            "exported_block_artifact_count": 1,
            "exported_extent_artifact_count": 1,
            "exported_full_file_artifact_count": 0
        }),
        serde_json::json!({
            "archive_path": "d.crushr",
            "archive_fingerprint": "f4",
            "verified_block_count": 2,
            "salvageable_file_count": 1,
            "unsalvageable_file_count": 0,
            "unmappable_file_count": 0,
            "exported_block_artifact_count": 1,
            "exported_extent_artifact_count": 1,
            "exported_full_file_artifact_count": 1
        }),
    ];

    for (archive_id, run_metadata) in archive_ids.into_iter().zip(metadata.into_iter()) {
        let run_dir = runs_dir.join(archive_id);
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(
            run_dir.join("run_metadata.json"),
            serde_json::to_vec_pretty(&run_metadata).unwrap(),
        )
        .unwrap();
        fs::write(run_dir.join("salvage_plan.json"), b"{}").unwrap();
    }

    let manifest = serde_json::json!({
        "experiment_id": "manual",
        "tool_version": "test",
        "schema_version": "crushr-lab-salvage-experiment.v1",
        "run_count": 4,
        "run_timestamp": "unix:0",
        "verification_label": "UNVERIFIED_RESEARCH_OUTPUT",
        "export_fragments_enabled": true,
        "archive_list": archive_ids,
    });
    fs::create_dir_all(&experiment_dir).unwrap();
    fs::write(
        experiment_dir.join("experiment_manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    run(Command::new(lab_bin)
        .arg("--resummarize")
        .arg(&experiment_dir));

    let summary = read_json(&experiment_dir.join("summary.json"));
    let outcomes: Vec<String> = summary["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["outcome"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(
        outcomes,
        vec![
            "NO_VERIFIED_EVIDENCE",
            "ORPHAN_EVIDENCE_ONLY",
            "PARTIAL_FILE_SALVAGE",
            "FULL_FILE_SALVAGE_AVAILABLE",
        ]
    );

    let analysis = read_json(&experiment_dir.join("analysis.json"));
    assert_eq!(
        analysis["outcome_groups"][0]["outcome"],
        "FULL_FILE_SALVAGE_AVAILABLE"
    );
    assert_eq!(analysis["outcome_groups"][0]["run_count"], 1);
    assert_eq!(
        analysis["outcome_groups"][1]["outcome"],
        "PARTIAL_FILE_SALVAGE"
    );
    assert_eq!(
        analysis["outcome_groups"][2]["outcome"],
        "ORPHAN_EVIDENCE_ONLY"
    );
    assert_eq!(
        analysis["outcome_groups"][3]["outcome"],
        "NO_VERIFIED_EVIDENCE"
    );

    assert_eq!(
        analysis["evidence_rankings"]["top_runs_by_verified_blocks"][0]["archive_id"],
        "c"
    );
    assert_eq!(
        analysis["evidence_rankings"]["top_runs_by_verified_blocks"][1]["archive_id"],
        "d"
    );

    assert_eq!(
        analysis["profile_groups"][0]["profile_key"],
        "UNKNOWN_PROFILE"
    );

    let analysis_raw = fs::read_to_string(experiment_dir.join("analysis.json")).unwrap();
    assert!(!analysis_raw.contains("block_candidates"));
    assert!(!analysis_raw.contains("exported_block_artifacts"));
}

#[test]
fn resummarize_profile_grouping_from_filename_markers() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let experiment_dir = td.path().join("experiment");
    let runs_dir = experiment_dir.join("runs");
    fs::create_dir_all(&runs_dir).unwrap();

    let archive_ids = ["a", "b"];
    let metadata = [
        serde_json::json!({
            "archive_path": "set_alpha_profile_light.crushr",
            "archive_fingerprint": "f1",
            "verified_block_count": 2,
            "salvageable_file_count": 1,
            "unsalvageable_file_count": 0,
            "unmappable_file_count": 0,
            "exported_block_artifact_count": 0,
            "exported_extent_artifact_count": 0,
            "exported_full_file_artifact_count": 0
        }),
        serde_json::json!({
            "archive_path": "set_beta_profile_heavy.crushr",
            "archive_fingerprint": "f2",
            "verified_block_count": 1,
            "salvageable_file_count": 0,
            "unsalvageable_file_count": 1,
            "unmappable_file_count": 0,
            "exported_block_artifact_count": 0,
            "exported_extent_artifact_count": 0,
            "exported_full_file_artifact_count": 0
        }),
    ];

    for (archive_id, run_metadata) in archive_ids.into_iter().zip(metadata.into_iter()) {
        let run_dir = runs_dir.join(archive_id);
        fs::create_dir_all(&run_dir).unwrap();
        fs::write(
            run_dir.join("run_metadata.json"),
            serde_json::to_vec_pretty(&run_metadata).unwrap(),
        )
        .unwrap();
        fs::write(run_dir.join("salvage_plan.json"), b"{}").unwrap();
    }

    let manifest = serde_json::json!({
        "experiment_id": "manual-profile",
        "tool_version": "test",
        "schema_version": "crushr-lab-salvage-experiment.v1",
        "run_count": 2,
        "run_timestamp": "unix:0",
        "verification_label": "UNVERIFIED_RESEARCH_OUTPUT",
        "export_fragments_enabled": false,
        "archive_list": archive_ids,
    });
    fs::create_dir_all(&experiment_dir).unwrap();
    fs::write(
        experiment_dir.join("experiment_manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    run(Command::new(lab_bin)
        .arg("--resummarize")
        .arg(&experiment_dir));

    let analysis = read_json(&experiment_dir.join("analysis.json"));
    let profile_keys: Vec<String> = analysis["profile_groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|group| group["profile_key"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(profile_keys, vec!["HEAVY", "LIGHT"]);
}

#[test]
fn harness_accepts_identity_archives_and_stable_ordering() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "src_a", "alpha", &input_dir.join("b.crs"));
    build_archive(pack_bin, "src_b", "beta", &input_dir.join("a.crushr"));
    build_archive(pack_bin, "src_c", "gamma", &input_dir.join("c"));
    fs::write(input_dir.join("note.txt"), b"not an archive").unwrap();
    fs::write(input_dir.join("note.corrupt.json"), b"{\"sidecar\":true}").unwrap();
    let disguised = input_dir.join("bad.crushr");
    fs::write(&disguised, b"totally not an archive").unwrap();

    let output_dir = td.path().join("experiment");
    run_harness(lab_bin, salvage_bin, &input_dir, &output_dir, false);

    let manifest = read_json(&output_dir.join("experiment_manifest.json"));
    assert_eq!(manifest["run_count"], 3);

    let runs = read_json(&output_dir.join("summary.json"))["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["archive_path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(runs, vec!["a.crushr", "b.crs", "c"]);
}

#[test]
fn harness_resolves_salvage_without_path_dependency() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "src", "payload", &input_dir.join("sample.crushr"));

    let output_dir = td.path().join("experiment");
    let mut cmd = Command::new(lab_bin);
    cmd.arg(&input_dir)
        .arg("--output")
        .arg(&output_dir)
        .env_remove("CRUSHR_SALVAGE_BIN")
        .env("PATH", "");
    run(&mut cmd);

    let manifest = read_json(&output_dir.join("experiment_manifest.json"));
    assert_eq!(manifest["run_count"], 1);
}

#[test]
fn harness_reports_clear_error_when_salvage_bin_resolution_fails() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "src", "payload", &input_dir.join("sample.crushr"));

    let mut cmd = Command::new(lab_bin);
    cmd.arg(&input_dir)
        .arg("--output")
        .arg(td.path().join("experiment"))
        .env("CRUSHR_SALVAGE_BIN", td.path().join("missing-salvage"));

    let stderr = run_harness_expect_fail(&mut cmd);
    assert!(stderr.contains("CRUSHR_SALVAGE_BIN points to missing/non-file path"));
}

#[test]
fn harness_prefers_explicit_salvage_env_over_fallback_resolution() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "src", "payload", &input_dir.join("sample.crushr"));

    let mut cmd = Command::new(lab_bin);
    cmd.arg(&input_dir)
        .arg("--output")
        .arg(td.path().join("experiment"))
        .env("CRUSHR_SALVAGE_BIN", td.path().join("missing-salvage"))
        .env(
            "CARGO_BIN_EXE_crushr-salvage",
            env!("CARGO_BIN_EXE_crushr-salvage"),
        );

    let stderr = run_harness_expect_fail(&mut cmd);
    assert!(stderr.contains("CRUSHR_SALVAGE_BIN points to missing/non-file path"));
}
