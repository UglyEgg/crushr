use jsonschema::JSONSchema;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_schema(path: &str) -> Value {
    let bytes = fs::read(workspace_root().join(path)).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn compile_schema(value: &Value) -> JSONSchema {
    JSONSchema::compile(value).unwrap()
}

fn assert_valid(schema: &JSONSchema, value: &Value, context: &str) {
    if let Err(errors) = schema.validate(value) {
        let rendered = errors
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        panic!("{context} failed schema validation:\n{rendered}\ninstance={value}");
    }
}

#[test]
fn raw_run_record_schema_accepts_rich_execution_record() {
    let schema = compile_schema(&load_schema(
        "schemas/crushr-lab-phase2-raw-run-records.v1.schema.json",
    ));

    let records = json!([
      {
        "scenario_id": "p2-core-smallfiles-crushr-bit_flip-header-1B-1337",
        "dataset": "smallfiles",
        "format": "crushr",
        "corruption_type": "bit_flip",
        "target_class": "header",
        "magnitude": "1B",
        "magnitude_bytes": 1,
        "seed": 1337,
        "source_archive_path": "../baselines/crushr/smallfiles.crs",
        "corrupted_archive_path": "raw/s1/smallfiles.crs.corrupt",
        "tool_kind": "crushr",
        "executable": "cargo",
        "argv": ["run", "-q", "-p", "crushr"],
        "cwd": "/workspace/crushr",
        "exit_code": 0,
        "stdout_path": "raw/s1/stdout.txt",
        "stderr_path": "raw/s1/stderr.txt",
        "json_result_path": "raw/s1/result.json",
        "has_json_result": true,
        "invocation_status": "completed",
        "stage_classification": null,
        "tool_version": {
          "status": "detected",
          "version": "crushr-info 0.2.2",
          "detail": null
        },
        "result_artifacts": {
          "stdout_path": "raw/s1/stdout.txt",
          "stderr_path": "raw/s1/stderr.txt",
          "json_result_path": "raw/s1/result.json"
        },
        "result_completeness": "structured_json_result",
        "run_context_paths": {
          "source_archive_path": "../baselines/crushr/smallfiles.crs",
          "corrupted_archive_path": "raw/s1/smallfiles.crs.corrupt",
          "corruption_log_path": "raw/s1/corruption_provenance.json"
        },
        "extraction_output_dir": "raw/s1/extracted",
        "recovery_report_path": "raw/s1/recovery_report.json",
        "recovery_accounting": {
          "files_expected": 24,
          "files_recovered": 24,
          "files_missing": 0,
          "bytes_expected": 1024,
          "bytes_recovered": 1024,
          "recovery_ratio_files": 1.0,
          "recovery_ratio_bytes": 1.0
        },
      }
    ]);

    assert_valid(&schema, &records, "phase2 raw run record");
}

#[test]
fn raw_run_record_schema_rejects_broken_tool_version_string() {
    let schema = compile_schema(&load_schema(
        "schemas/crushr-lab-phase2-raw-run-records.v1.schema.json",
    ));

    let invalid = json!([
      {
        "scenario_id": "p2-core-smallfiles-crushr-bit_flip-header-1B-1337",
        "dataset": "smallfiles",
        "format": "crushr",
        "corruption_type": "bit_flip",
        "target_class": "header",
        "magnitude": "1B",
        "magnitude_bytes": 1,
        "seed": 1337,
        "source_archive_path": "a",
        "corrupted_archive_path": "b",
        "tool_kind": "crushr",
        "executable": "cargo",
        "argv": [],
        "cwd": null,
        "exit_code": 2,
        "stdout_path": "o",
        "stderr_path": "e",
        "json_result_path": null,
        "has_json_result": false,
        "invocation_status": "completed",
        "stage_classification": null,
        "tool_version": "unsupported flag: --version",
        "result_artifacts": {"stdout_path": "o", "stderr_path": "e", "json_result_path": null},
        "result_completeness": "stdout_and_stderr",
        "run_context_paths": {"source_archive_path": "a", "corrupted_archive_path": "b", "corruption_log_path": "c"},
        "extraction_output_dir": "raw/s1/extracted",
        "recovery_report_path": "raw/s1/recovery_report.json",
        "recovery_accounting": {"files_expected": 1, "files_recovered": 0, "files_missing": 1, "bytes_expected": 10, "bytes_recovered": 0, "recovery_ratio_files": 0.0, "recovery_ratio_bytes": 0.0}
      }
    ]);

    assert!(schema.validate(&invalid).is_err());
}

#[test]
fn execution_report_schema_accepts_summary_shape() {
    let schema = compile_schema(&load_schema(
        "schemas/crushr-lab-phase2-execution-report.v1.schema.json",
    ));

    let report = json!({
      "expected_runs": 2700,
      "actual_runs": 2700,
      "records_path": "raw_run_records.json",
      "completeness_path": "completeness_audit.json",
      "scenario_count_by_format": {"crushr": 540, "zip": 540, "tar+zstd": 540, "tar+gz": 540, "tar+xz": 540},
      "scenario_count_by_dataset": {"smallfiles": 900, "mixed": 900, "largefiles": 900},
      "exit_code_histogram": {"0": 1234, "1": 1466},
      "has_json_result_counts": {"true_count": 540, "false_count": 2160},
      "tool_versions": {
        "by_tool": [
          {"tool_kind": "crushr", "executable": "cargo", "status": "detected", "version": "crushr-info 0.2.2", "detail": null},
          {"tool_kind": "tar+gz", "executable": "tar", "status": "unsupported", "version": null, "detail": "shared with tar"}
        ]
      },
      "completeness_audit_passed": true
    });

    assert_valid(&schema, &report, "phase2 execution report");
}

#[test]
fn execution_report_schema_rejects_missing_summary_sections() {
    let schema = compile_schema(&load_schema(
        "schemas/crushr-lab-phase2-execution-report.v1.schema.json",
    ));
    let invalid = json!({
      "expected_runs": 1,
      "actual_runs": 1,
      "records_path": "raw_run_records.json",
      "completeness_path": "completeness_audit.json"
    });
    assert!(schema.validate(&invalid).is_err());
}
