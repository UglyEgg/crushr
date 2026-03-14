use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("run command");
    if !out.status.success() {
        panic!(
            "command failed: {:?}
stdout:
{}
stderr:
{}",
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

#[test]
fn harness_generates_run_layout_and_summary() {
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

    let manifest = read_json(&output_dir.join("experiment_manifest.json"));
    assert_eq!(manifest["run_count"], 2);
    let archive_list = manifest["archive_list"].as_array().unwrap();
    assert_eq!(archive_list.len(), 2);

    let runs_dir = output_dir.join("runs");
    for archive_id in archive_list {
        let run_dir = runs_dir.join(archive_id.as_str().unwrap());
        assert!(run_dir.join("salvage_plan.json").exists());
        assert!(run_dir.join("run_metadata.json").exists());
        assert!(!run_dir.join("exported_artifacts").exists());

        let metadata = read_json(&run_dir.join("run_metadata.json"));
        assert!(metadata["archive_path"].is_string());
        assert!(metadata["archive_fingerprint"].is_string());
        assert!(metadata["verified_block_count"].is_u64());
        assert_eq!(metadata["exported_artifact_count"], 0);
    }
}

#[test]
fn harness_manifest_order_is_deterministic() {
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

    let first = read_json(&out_one.join("experiment_manifest.json"));
    let second = read_json(&out_two.join("experiment_manifest.json"));
    assert_eq!(first["archive_list"], second["archive_list"]);
}

#[test]
fn harness_export_toggle_controls_artifact_directory() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let input_dir = td.path().join("archives");
    fs::create_dir_all(&input_dir).unwrap();

    build_archive(pack_bin, "x", "payload", &input_dir.join("only.crushr"));

    let out_disabled = td.path().join("disabled");
    run_harness(lab_bin, salvage_bin, &input_dir, &out_disabled, false);
    let disabled_manifest = read_json(&out_disabled.join("experiment_manifest.json"));
    let disabled_id = disabled_manifest["archive_list"][0].as_str().unwrap();
    let disabled_run = out_disabled.join("runs").join(disabled_id);
    assert!(!disabled_run.join("exported_artifacts").exists());
    assert_eq!(
        read_json(&disabled_run.join("run_metadata.json"))["exported_artifact_count"],
        0
    );

    let out_enabled = td.path().join("enabled");
    run_harness(lab_bin, salvage_bin, &input_dir, &out_enabled, true);
    let enabled_manifest = read_json(&out_enabled.join("experiment_manifest.json"));
    let enabled_id = enabled_manifest["archive_list"][0].as_str().unwrap();
    let enabled_run = out_enabled.join("runs").join(enabled_id);
    assert!(enabled_run.join("exported_artifacts").exists());
    assert!(
        read_json(&enabled_run.join("run_metadata.json"))["exported_artifact_count"]
            .as_u64()
            .unwrap()
            > 0
    );
}
