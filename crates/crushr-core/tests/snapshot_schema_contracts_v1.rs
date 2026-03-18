use crushr_core::impact::{enumerate_impact_v1, FileEntryV1, FileExtentV1};
use jsonschema::JSONSchema;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
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
            .args(["build", "-q", "-p", "crushr", "--bins"])
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

fn load_schema(path: &str) -> JSONSchema {
    let abs = workspace_root().join(path);
    let bytes = fs::read(abs).unwrap();
    let schema: Value = serde_json::from_slice(&bytes).unwrap();
    JSONSchema::compile(&schema).unwrap()
}

fn validate_instance<T: Serialize>(validator: &JSONSchema, instance: &T, context: &str) {
    let value = serde_json::to_value(instance).unwrap();
    let errs = match validator.validate(&value) {
        Ok(()) => return,
        Err(errors) => errors.map(|e| e.to_string()).collect::<Vec<_>>(),
    };
    panic!(
        "schema validation failed for {context}:\n{}\ninstance={value}",
        errs.join("\n")
    );
}

#[test]
fn crushr_info_json_validates_against_v1_schema() {
    ensure_bins_built();
    let schema = load_schema("schemas/crushr-info.v1.schema.json");

    let root = unique_dir("crushr-info-schema");
    fs::create_dir_all(&root).unwrap();

    let src = root.join("single.txt");
    fs::write(&src, b"hello schema\n").unwrap();
    let archive = root.join("single.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let out = run_bin("crushr-info", &[archive.to_str().unwrap(), "--json"]);
    assert_ok(&out);
    let instance: Value = serde_json::from_slice(&out.stdout).unwrap();
    validate_instance(&schema, &instance, "crushr-info");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn crushr_extract_verify_json_is_deterministic_and_strict() {
    ensure_bins_built();

    let root = unique_dir("crushr-extract-verify-schema");
    fs::create_dir_all(&root).unwrap();

    let src = root.join("single.txt");
    fs::write(&src, b"hello extract verify schema\n").unwrap();
    let archive = root.join("single.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let out = run_bin(
        "crushr-extract",
        &["--verify", archive.to_str().unwrap(), "--json"],
    );
    assert_ok(&out);
    let instance: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(instance["verification_status"], "verified");
    assert_eq!(instance["safe_for_strict_extraction"], true);
    assert!(instance["refusal_reasons"].as_array().unwrap().is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn impact_report_validates_against_v1_schema() {
    let schema = load_schema("schemas/crushr-impact.v1.schema.json");

    let files = vec![
        FileEntryV1 {
            file_id: 1,
            path: "a.txt".to_string(),
            extents: vec![FileExtentV1 {
                block_id: 1,
                offset_in_block: 0,
                len: 5,
            }],
        },
        FileEntryV1 {
            file_id: 2,
            path: "b.txt".to_string(),
            extents: vec![FileExtentV1 {
                block_id: 2,
                offset_in_block: 2,
                len: 3,
            }],
        },
    ];

    let report = enumerate_impact_v1(&BTreeSet::from([2u32]), &files);
    validate_instance(&schema, &report, "impact-report");
}
