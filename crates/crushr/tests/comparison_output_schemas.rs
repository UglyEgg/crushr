// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("run command");
    if !out.status.success() {
        panic!(
            "command failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn assert_required_fields(value: &Value, schema: &Value, path: &str) {
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required {
            let field = key.as_str().unwrap();
            assert!(
                value.get(field).is_some(),
                "missing required field {}.{}",
                path,
                field
            );
        }
    }

    let schema_type = schema.get("type").and_then(Value::as_str);
    match schema_type {
        Some("object") => {
            if let Some(props) = schema.get("properties").and_then(Value::as_object) {
                for (key, sub_schema) in props {
                    if let Some(v) = value.get(key) {
                        assert_required_fields(v, sub_schema, &format!("{}.{}", path, key));
                    }
                }
            }
        }
        Some("array") => {
            if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
                for (idx, item) in arr.iter().enumerate() {
                    assert_required_fields(item, items, &format!("{}[{}]", path, idx));
                }
            }
        }
        _ => {}
    }
}

fn assert_summary_matches_schema(summary_path: &Path, schema_file: &str) {
    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(schema_file);
    let schema: Value = serde_json::from_slice(&fs::read(schema_path).unwrap()).unwrap();

    assert_required_fields(&summary, &schema, "$root");

    let expected_version = schema["properties"]["schema_version"]["const"]
        .as_str()
        .unwrap();
    assert_eq!(
        summary["schema_version"].as_str().unwrap(),
        expected_version
    );

    let canonical = serde_json::to_string_pretty(&summary).unwrap();
    let reparsed: Value = serde_json::from_str(&canonical).unwrap();
    assert_eq!(canonical, serde_json::to_string_pretty(&reparsed).unwrap());
}

#[test]
fn format12_summary_artifact_is_schema_backed() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    run(Command::new(lab_bin)
        .arg("run-format12-inline-path-comparison")
        .arg("--output")
        .arg(td.path()));
    assert_summary_matches_schema(
        &td.path().join("format12_comparison_summary.json"),
        "crushr-lab-salvage-format12-inline-path-comparison.v1.schema.json",
    );
}

#[test]
fn format13_summary_artifact_is_schema_backed() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    run(Command::new(lab_bin)
        .arg("run-format13-comparison")
        .arg("--output")
        .arg(td.path()));
    assert_summary_matches_schema(
        &td.path().join("format13_comparison_summary.json"),
        "crushr-lab-salvage-format13-comparison.v1.schema.json",
    );
}

#[test]
fn format14a_summary_artifact_is_schema_backed() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    run(Command::new(lab_bin)
        .arg("run-format14a-dictionary-resilience-comparison")
        .arg("--output")
        .arg(td.path()));
    assert_summary_matches_schema(
        &td.path()
            .join("format14a_dictionary_resilience_summary.json"),
        "crushr-lab-salvage-format14a-dictionary-resilience.v1.schema.json",
    );
}

#[test]
fn format15_summary_artifact_is_schema_backed() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    run(Command::new(lab_bin)
        .arg("run-format15-comparison")
        .arg("--output")
        .arg(td.path()));
    assert_summary_matches_schema(
        &td.path().join("format15_comparison_summary.json"),
        "crushr-lab-salvage-format15.v1.schema.json",
    );
}
