use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

static BUILD_ONCE: Once = Once::new();

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn unique_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

fn ensure_bins_built() {
    BUILD_ONCE.call_once(|| {
        let out = Command::new("cargo")
            .current_dir(workspace_root())
            .args([
                "build",
                "-q",
                "-p",
                "crushr",
                "--bin",
                "crushr-pack",
                "--bin",
                "crushr-info",
                "--bin",
                "crushr-fsck",
                "-p",
                "crushr-lab",
                "--bin",
                "crushr-lab",
            ])
            .output()
            .expect("run cargo build");
        assert!(
            out.status.success(),
            "cargo build failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    });
}

fn run_bin(bin: &str, args: &[&str]) -> std::process::Output {
    let bin_path = workspace_root().join(format!("target/debug/{bin}"));
    Command::new(bin_path).args(args).output().unwrap()
}

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn first_e2e_corruption_experiment_loop() {
    ensure_bins_built();

    let root = unique_dir("crushr-first-exp");
    fs::create_dir_all(&root).unwrap();

    let fixture = root.join("fixture.txt");
    fs::write(&fixture, b"crushr experiment fixture\nline-2\nline-3\n").unwrap();

    let clean_archive = root.join("clean.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[
            fixture.to_str().unwrap(),
            "-o",
            clean_archive.to_str().unwrap(),
        ],
    ));

    let clean_info = run_bin("crushr-info", &[clean_archive.to_str().unwrap(), "--json"]);
    assert_ok(&clean_info);
    let clean_info_json: Value = serde_json::from_slice(&clean_info.stdout).unwrap();
    assert_eq!(clean_info_json["tool"], "crushr-info");

    let clean_fsck = run_bin("crushr-fsck", &[clean_archive.to_str().unwrap(), "--json"]);
    assert_ok(&clean_fsck);
    let clean_fsck_json: Value = serde_json::from_slice(&clean_fsck.stdout).unwrap();
    assert_eq!(clean_fsck_json["payload"]["verify"]["status"], "ok");

    let corrupted_archive = root.join("corrupt.crs");
    let clean_len = fs::metadata(&clean_archive).unwrap().len();
    let offset = (clean_len - 1).to_string();
    let seed = "1337";

    assert_ok(&run_bin(
        "crushr-lab",
        &[
            "corrupt",
            clean_archive.to_str().unwrap(),
            corrupted_archive.to_str().unwrap(),
            "--model",
            "byteflip",
            "--seed",
            seed,
            "--offset",
            &offset,
        ],
    ));

    let corruption_log: Value = serde_json::from_slice(
        &fs::read(corrupted_archive.with_extension("corrupt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(corruption_log["model"], "byteflip");
    assert_eq!(corruption_log["seed"], 1337);
    assert_eq!(corruption_log["touched_offsets"][0], clean_len - 1);

    let corrupted_fsck = run_bin(
        "crushr-fsck",
        &[corrupted_archive.to_str().unwrap(), "--json"],
    );
    assert!(!corrupted_fsck.status.success());

    let corrupted_info = run_bin(
        "crushr-info",
        &[corrupted_archive.to_str().unwrap(), "--json"],
    );
    assert!(!corrupted_info.status.success());

    let rerun_archive = root.join("corrupt-rerun.crs");
    assert_ok(&run_bin(
        "crushr-lab",
        &[
            "corrupt",
            clean_archive.to_str().unwrap(),
            rerun_archive.to_str().unwrap(),
            "--model",
            "byteflip",
            "--seed",
            seed,
            "--offset",
            &offset,
        ],
    ));

    let rerun_log: Value =
        serde_json::from_slice(&fs::read(rerun_archive.with_extension("corrupt.json")).unwrap())
            .unwrap();
    assert_eq!(corruption_log, rerun_log);
    assert_eq!(
        fs::read(&corrupted_archive).unwrap(),
        fs::read(&rerun_archive).unwrap()
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn first_experiment_runner_writes_expected_artifacts() {
    ensure_bins_built();

    let root = unique_dir("crushr-first-exp-runner");
    fs::create_dir_all(&root).unwrap();

    let out = run_bin(
        "crushr-lab",
        &[
            "run-first-experiment",
            "--artifact-dir",
            root.to_str().unwrap(),
        ],
    );
    assert_ok(&out);

    let expected = [
        "fixture.txt",
        "clean.crs",
        "clean.info.json",
        "clean.fsck.json",
        "corrupt.crs",
        "corrupt.corrupt.json",
        "corrupt.fsck.exit_code.txt",
        "corrupt.fsck.stderr.txt",
        "corrupt.info.exit_code.txt",
        "corrupt.info.stderr.txt",
        "experiment_manifest.json",
    ];

    for name in expected {
        assert!(root.join(name).exists(), "missing artifact {name}");
    }

    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("experiment_manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["experiment_id"],
        "crushr_p0s12f0_first_e2e_byteflip"
    );

    let clean_fsck: Value =
        serde_json::from_slice(&fs::read(root.join("clean.fsck.json")).unwrap()).unwrap();
    assert_eq!(clean_fsck["payload"]["verify"]["status"], "ok");

    assert_eq!(
        fs::read_to_string(root.join("corrupt.fsck.exit_code.txt")).unwrap(),
        "2\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("corrupt.info.exit_code.txt")).unwrap(),
        "2\n"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn recorded_results_reference_experiment_artifact() {
    let manifest_path = workspace_root()
        .join("docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/experiment_manifest.json");
    let results_path = workspace_root().join("docs/RESEARCH/RESULTS.md");

    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let results = fs::read_to_string(&results_path).unwrap();

    let experiment_id = manifest["experiment_id"].as_str().unwrap();
    let seed = manifest["seed"].as_u64().unwrap();
    assert!(results.contains(experiment_id));
    assert!(results.contains(&format!("seed `{seed}`")));
    assert!(results.contains("docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip"));
}
