use jsonschema::JSONSchema;
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

fn load_extract_result_schema() -> Value {
    let path = workspace_root().join("schemas/crushr-extract-result.v1.schema.json");
    let bytes = fs::read(path).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn compile_schema(schema: &Value) -> JSONSchema {
    JSONSchema::compile(schema).expect("compile schema")
}

fn assert_valid_against_schema(validator: &JSONSchema, instance: &Value, context: &str) {
    if let Err(errors) = validator.validate(instance) {
        let rendered = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
        panic!("{context} failed schema validation:\n{rendered}\ninstance={instance}");
    }
}
fn as_object<'a>(value: &'a Value, context: &str) -> &'a serde_json::Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object for {context}"))
}

fn as_array<'a>(value: &'a Value, context: &str) -> &'a [Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected array for {context}"))
}

fn assert_sorted_paths(arr: &[Value], context: &str) {
    let mut paths = Vec::new();
    for item in arr {
        let obj = as_object(item, context);
        paths.push(
            obj.get("path")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("missing path for {context}")),
        );
    }

    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted, "{context} paths must be sorted");
}

fn validate_success_or_partial(schema: &Value, instance: &Value) {
    let root = as_object(schema, "schema-root");
    let defs = as_object(root.get("$defs").unwrap(), "$defs");
    let refused_reason = defs["refused_file"]["properties"]["reason"]["const"]
        .as_str()
        .unwrap();
    let obj = as_object(instance, "extract-result-success-partial");
    let status = obj["overall_status"].as_str().unwrap();
    assert!(matches!(status, "success" | "partial_refusal"));

    assert_eq!(obj["maximal_safe_set_computed"], Value::Bool(true));

    let safe_files = as_array(&obj["safe_files"], "safe_files");
    let refused_files = as_array(&obj["refused_files"], "refused_files");

    assert_eq!(
        obj["safe_file_count"].as_u64().unwrap(),
        safe_files.len() as u64
    );
    assert_eq!(
        obj["refused_file_count"].as_u64().unwrap(),
        refused_files.len() as u64
    );

    assert_sorted_paths(safe_files, "safe_files");
    assert_sorted_paths(refused_files, "refused_files");

    for item in refused_files {
        let refused = as_object(item, "refused_file");
        assert_eq!(
            refused.get("reason").and_then(Value::as_str).unwrap(),
            refused_reason
        );
    }

    let allowed = BTreeSet::from([
        "overall_status",
        "maximal_safe_set_computed",
        "safe_files",
        "refused_files",
        "safe_file_count",
        "refused_file_count",
    ]);

    let actual = obj.keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        actual, allowed,
        "unexpected fields in success/partial report"
    );
}

fn validate_error(instance: &Value) {
    let obj = as_object(instance, "extract-result-error");
    assert_eq!(obj["overall_status"].as_str().unwrap(), "error");
    assert!(obj["error"].as_str().is_some());

    let actual = obj.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let allowed = BTreeSet::from(["overall_status", "error"]);
    assert_eq!(actual, allowed, "unexpected fields in error report");
}

fn validate_extract_result_against_schema(schema: &Value, instance: &Value) {
    let status = instance["overall_status"].as_str().unwrap_or_default();
    if status == "error" {
        validate_error(instance);
    } else {
        validate_success_or_partial(schema, instance);
    }
}

#[test]
fn extract_result_json_conforms_to_v1_schema_for_all_envelopes() {
    ensure_bins_built();
    let schema = load_extract_result_schema();
    let validator = compile_schema(&schema);

    let root = unique_dir("crushr-extract-schema-v1");
    fs::create_dir_all(&root).unwrap();

    // strict success
    let single = root.join("single.txt");
    fs::write(&single, b"schema strict success\n").unwrap();
    let single_archive = root.join("single.crs");
    let packed = run_bin(
        "crushr-pack",
        &[
            single.to_str().unwrap(),
            "-o",
            single_archive.to_str().unwrap(),
        ],
    );
    assert_ok(&packed);

    let strict_out = root.join("out-strict");
    let strict_extract = run_bin(
        "crushr-extract",
        &[
            single_archive.to_str().unwrap(),
            "-o",
            strict_out.to_str().unwrap(),
            "--json",
        ],
    );
    assert_ok(&strict_extract);
    let strict_json: Value = serde_json::from_slice(&strict_extract.stdout).unwrap();
    assert_valid_against_schema(&validator, &strict_json, "strict extract result");
    validate_extract_result_against_schema(&schema, &strict_json);

    // structural error envelope
    let mut invalid_bytes = fs::read(&single_archive).unwrap();
    let last = invalid_bytes.len() - 1;
    invalid_bytes[last] ^= 0x01;
    let broken = root.join("single-broken.crs");
    fs::write(&broken, invalid_bytes).unwrap();

    let err_out = root.join("out-error");
    let errored = run_bin(
        "crushr-extract",
        &[
            broken.to_str().unwrap(),
            "-o",
            err_out.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(errored.status.code(), Some(2));
    let error_json: Value = serde_json::from_slice(&errored.stdout).unwrap();
    assert_valid_against_schema(&validator, &error_json, "error extract result");
    validate_extract_result_against_schema(&schema, &error_json);

    let _ = fs::remove_dir_all(&root);
}
