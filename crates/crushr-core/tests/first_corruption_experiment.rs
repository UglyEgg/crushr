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
                "crushr-extract",
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

    let clean_verify = run_bin(
        "crushr-extract",
        &["--verify", clean_archive.to_str().unwrap(), "--json"],
    );
    assert_ok(&clean_verify);
    let clean_verify_json: Value = serde_json::from_slice(&clean_verify.stdout).unwrap();
    assert_eq!(clean_verify_json["verification_status"], "verified");

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

    let corrupted_verify = run_bin(
        "crushr-extract",
        &["--verify", corrupted_archive.to_str().unwrap(), "--json"],
    );
    assert!(!corrupted_verify.status.success());

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
