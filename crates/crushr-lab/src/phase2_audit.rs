use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::phase2_domain::{
    ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass, LOCKED_CORE_SEEDS,
};
use crate::phase2_manifest::{
    enumerate_locked_core_scenarios, validate_manifest_shape, Phase2ExperimentManifest,
    PHASE2_MANIFEST_SCHEMA_ID,
};

const EXPECTED_SCENARIO_COUNT: usize = 2700;

pub fn run_phase2_pretrial_audit(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let workspace_root = crate::cli::workspace_root()?;
    let mut manifest_path =
        workspace_root.join("PHASE2_RESEARCH/manifests/phase2_core_manifest.json");
    let mut output_path = workspace_root.join("PHASE2_RESEARCH/outputs/pretrial_audit.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                manifest_path = PathBuf::from(args.next().context("missing value for --manifest")?);
            }
            "--output" => {
                output_path = PathBuf::from(args.next().context("missing value for --output")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let report = build_pretrial_audit_report(&workspace_root, &manifest_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_vec_pretty(&report)?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditFailureCode {
    MissingTool,
    InvalidManifest,
    UnwritableOutputRoot,
    MissingSupportFile,
    Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCheck {
    pub name: String,
    pub ok: bool,
    pub failure_code: Option<AuditFailureCode>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub format: ArchiveFormat,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputRootStatus {
    pub path: String,
    pub writable: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedMatrixSummary {
    pub datasets: Vec<Dataset>,
    pub formats: Vec<ArchiveFormat>,
    pub corruption_types: Vec<CorruptionType>,
    pub target_classes: Vec<TargetClass>,
    pub magnitudes: Vec<Magnitude>,
    pub seeds: Vec<u64>,
    pub expected_scenarios: usize,
    pub observed_manifest_scenarios: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialReadinessSummary {
    pub ok: bool,
    pub total_checks: usize,
    pub failing_checks: usize,
    pub missing_tools: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PretrialAuditReport {
    pub audit_kind: String,
    pub pass: bool,
    pub checks: Vec<AuditCheck>,
    pub failing_checks: Vec<String>,
    pub tools: Vec<ToolStatus>,
    pub output_roots: Vec<OutputRootStatus>,
    pub locked_matrix: LockedMatrixSummary,
    pub readiness_summary: TrialReadinessSummary,
}

pub fn build_pretrial_audit_report(
    workspace_root: &Path,
    manifest_path: &Path,
) -> Result<PretrialAuditReport> {
    let manifest_value = read_manifest_value(manifest_path)?;
    let observed_manifest_scenarios = manifest_value
        .get("scenarios")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);

    let mut checks = Vec::new();
    checks.push(check_manifest_schema_and_shape(&manifest_value));
    checks.push(check_locked_matrix_and_ids(&manifest_value));
    checks.extend(check_required_files(workspace_root));

    let output_roots = check_output_roots(workspace_root);
    checks.extend(output_roots.iter().map(|status| AuditCheck {
        name: format!("output_root_writable:{}", status.path),
        ok: status.writable,
        failure_code: (!status.writable).then_some(AuditFailureCode::UnwritableOutputRoot),
        detail: status.detail.clone(),
    }));

    let tools = check_tools(workspace_root);
    checks.extend(tools.iter().map(|status| AuditCheck {
        name: format!("tool_available:{}", status.format.slug()),
        ok: status.ok,
        failure_code: (!status.ok).then_some(AuditFailureCode::MissingTool),
        detail: status.detail.clone(),
    }));

    let failing_checks = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.name.clone())
        .collect::<Vec<_>>();

    let pass = failing_checks.is_empty();
    let readiness_summary = TrialReadinessSummary {
        ok: pass,
        total_checks: checks.len(),
        failing_checks: failing_checks.len(),
        missing_tools: tools.iter().filter(|tool| !tool.ok).count(),
    };

    Ok(PretrialAuditReport {
        audit_kind: "phase2_pretrial_readiness_v1".to_string(),
        pass,
        checks,
        failing_checks,
        tools,
        output_roots,
        locked_matrix: LockedMatrixSummary {
            datasets: Dataset::ordered_locked_core().to_vec(),
            formats: ArchiveFormat::ordered_locked_core().to_vec(),
            corruption_types: CorruptionType::ordered_locked_core().to_vec(),
            target_classes: TargetClass::ordered_locked_core().to_vec(),
            magnitudes: Magnitude::ordered_locked_core().to_vec(),
            seeds: LOCKED_CORE_SEEDS.to_vec(),
            expected_scenarios: EXPECTED_SCENARIO_COUNT,
            observed_manifest_scenarios,
        },
        readiness_summary,
    })
}

fn read_manifest_value(manifest_path: &Path) -> Result<Value> {
    let bytes = fs::read(manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
    let value = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse manifest {}", manifest_path.display()))?;
    Ok(value)
}

fn check_manifest_schema_and_shape(manifest: &Value) -> AuditCheck {
    let result = (|| -> Result<()> {
        let schema = manifest
            .get("$schema")
            .and_then(Value::as_str)
            .context("manifest missing string $schema")?;
        if schema != PHASE2_MANIFEST_SCHEMA_ID {
            bail!("manifest $schema mismatch: expected {PHASE2_MANIFEST_SCHEMA_ID}, got {schema}");
        }
        validate_manifest_shape(manifest)?;
        Ok(())
    })();

    match result {
        Ok(()) => AuditCheck {
            name: "manifest_schema_and_shape".to_string(),
            ok: true,
            failure_code: None,
            detail: "manifest schema id and shape validation passed".to_string(),
        },
        Err(err) => AuditCheck {
            name: "manifest_schema_and_shape".to_string(),
            ok: false,
            failure_code: Some(AuditFailureCode::InvalidManifest),
            detail: err.to_string(),
        },
    }
}

fn check_locked_matrix_and_ids(manifest: &Value) -> AuditCheck {
    let result = (|| -> Result<()> {
        let parsed: Phase2ExperimentManifest = serde_json::from_value(manifest.clone())
            .context("manifest does not deserialize into Phase2ExperimentManifest")?;

        if parsed.scenarios.len() != EXPECTED_SCENARIO_COUNT {
            bail!(
                "manifest scenario count mismatch: expected {EXPECTED_SCENARIO_COUNT}, got {}",
                parsed.scenarios.len()
            );
        }

        if parsed.formats != ArchiveFormat::ordered_locked_core().to_vec() {
            bail!("manifest format ordering or locked comparator set differs from PHASE2 lock");
        }

        if parsed.ordering
            != [
                "dataset",
                "format",
                "corruption_type",
                "target_class",
                "magnitude",
                "seed",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        {
            bail!("manifest ordering differs from locked core ordering");
        }

        let expected_ids = enumerate_locked_core_scenarios()
            .into_iter()
            .map(|scenario| scenario.scenario_id)
            .collect::<Vec<_>>();

        let actual_ids = parsed
            .scenarios
            .iter()
            .map(|scenario| scenario.scenario_id.clone())
            .collect::<Vec<_>>();

        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();
        for id in &actual_ids {
            if !seen.insert(id.clone()) {
                duplicates.push(id.clone());
            }
        }
        if !duplicates.is_empty() {
            duplicates.sort();
            duplicates.dedup();
            bail!(
                "manifest has duplicate scenario IDs: {}",
                duplicates.join(",")
            );
        }

        if actual_ids != expected_ids {
            let expected_set = expected_ids.iter().cloned().collect::<HashSet<_>>();
            let actual_set = actual_ids.iter().cloned().collect::<HashSet<_>>();
            let mut missing = expected_set
                .difference(&actual_set)
                .cloned()
                .collect::<Vec<_>>();
            let mut unexpected = actual_set
                .difference(&expected_set)
                .cloned()
                .collect::<Vec<_>>();
            missing.sort();
            unexpected.sort();
            bail!(
                "manifest IDs differ from locked matrix (missing={}, unexpected={})",
                missing.len(),
                unexpected.len()
            );
        }

        Ok(())
    })();

    match result {
        Ok(()) => AuditCheck {
            name: "locked_matrix_scenarios".to_string(),
            ok: true,
            failure_code: None,
            detail: "manifest scenario IDs/count/ordering and comparator set match lock"
                .to_string(),
        },
        Err(err) => AuditCheck {
            name: "locked_matrix_scenarios".to_string(),
            ok: false,
            failure_code: Some(AuditFailureCode::InvalidManifest),
            detail: err.to_string(),
        },
    }
}

fn check_required_files(workspace_root: &Path) -> Vec<AuditCheck> {
    [
        "PHASE2_RESEARCH/README.md",
        "PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md",
        "schemas/crushr-lab-experiment-manifest.phase2.v1.schema.json",
        "schemas/crushr-info.v1.schema.json",
        "schemas/crushr-fsck.v1.schema.json",
        "schemas/crushr-impact.v1.schema.json",
        "schemas/crushr-extract-result.v1.schema.json",
        "schemas/crushr-propagation-graph.v1.schema.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = workspace_root.join(relative);
        let ok = path.is_file();
        AuditCheck {
            name: format!("required_file:{relative}"),
            ok,
            failure_code: (!ok).then_some(AuditFailureCode::MissingSupportFile),
            detail: if ok {
                "present".to_string()
            } else {
                format!("missing required file {}", path.display())
            },
        }
    })
    .collect()
}

fn check_output_roots(workspace_root: &Path) -> Vec<OutputRootStatus> {
    [
        "PHASE2_RESEARCH/manifests",
        "PHASE2_RESEARCH/generated/foundation",
        "PHASE2_RESEARCH/generated/execution",
        "PHASE2_RESEARCH/normalized",
        "PHASE2_RESEARCH/summaries",
        "PHASE2_RESEARCH/whitepaper_support",
        "PHASE2_RESEARCH/outputs",
    ]
    .into_iter()
    .map(|relative| {
        let path = workspace_root.join(relative);
        let status = ensure_path_writable(&path);
        OutputRootStatus {
            path: relative.to_string(),
            writable: status.is_ok(),
            detail: status
                .map(|_| "writable".to_string())
                .unwrap_or_else(|err| err.to_string()),
        }
    })
    .collect()
}

fn ensure_path_writable(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create output root {}", path.display()))?;
    let probe = path.join(".audit_write_probe.tmp");
    fs::write(&probe, b"phase2 audit write check")
        .with_context(|| format!("failed to write output root {}", path.display()))?;
    fs::remove_file(&probe)
        .with_context(|| format!("failed to remove output root probe {}", probe.display()))?;
    Ok(())
}

fn check_tools(workspace_root: &Path) -> Vec<ToolStatus> {
    vec![
        tool_status_from_command(
            ArchiveFormat::Crushr,
            {
                let mut cmd = Command::new("cargo");
                cmd.current_dir(workspace_root)
                    .arg("run")
                    .arg("-q")
                    .arg("-p")
                    .arg("crushr")
                    .arg("--bin")
                    .arg("crushr-info")
                    .arg("--")
                    .arg("--version");
                cmd
            },
            "cargo run -q -p crushr --bin crushr-info -- --version",
        ),
        tool_status_from_command(
            ArchiveFormat::Zip,
            {
                let mut cmd = Command::new("zip");
                cmd.arg("--version");
                cmd
            },
            "zip --version",
        ),
        tool_status_pair(
            ArchiveFormat::TarZstd,
            check_single_command(
                {
                    let mut cmd = Command::new("tar");
                    cmd.arg("--version");
                    cmd
                },
                "tar --version",
            ),
            check_single_command(
                {
                    let mut cmd = Command::new("zstd");
                    cmd.arg("--version");
                    cmd
                },
                "zstd --version",
            ),
        ),
        tool_status_from_command(
            ArchiveFormat::TarGz,
            {
                let mut cmd = Command::new("tar");
                cmd.arg("--version");
                cmd
            },
            "tar --version",
        ),
        tool_status_from_command(
            ArchiveFormat::TarXz,
            {
                let mut cmd = Command::new("tar");
                cmd.arg("--version");
                cmd
            },
            "tar --version",
        ),
    ]
}

fn tool_status_pair(
    format: ArchiveFormat,
    first: (bool, String),
    second: (bool, String),
) -> ToolStatus {
    let ok = first.0 && second.0;
    let detail = format!("{}; {}", first.1, second.1);
    ToolStatus { format, ok, detail }
}

fn tool_status_from_command(format: ArchiveFormat, cmd: Command, description: &str) -> ToolStatus {
    let (ok, detail) = check_single_command(cmd, description);
    ToolStatus { format, ok, detail }
}

fn check_single_command(mut cmd: Command, description: &str) -> (bool, String) {
    match cmd.output() {
        Ok(out) => {
            if out.status.success() {
                (true, format!("{description}: available"))
            } else {
                (
                    false,
                    format!(
                        "{description}: non-zero exit status {}",
                        out.status.code().unwrap_or(-1)
                    ),
                )
            }
        }
        Err(err) => (false, format!("{description}: unavailable ({err})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase2_manifest::Phase2ExperimentManifest;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("crushr_lab_{label}_{ts}"))
    }

    fn write_manifest(path: &Path, manifest: Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }

    fn locked_manifest_value() -> Value {
        let mut manifest = serde_json::to_value(Phase2ExperimentManifest::locked_core()).unwrap();
        manifest.as_object_mut().unwrap().insert(
            "$schema".to_string(),
            Value::String(PHASE2_MANIFEST_SCHEMA_ID.to_string()),
        );
        manifest
    }

    fn create_workspace_with_required_files(root: &Path) {
        for relative in [
            "PHASE2_RESEARCH/README.md",
            "PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md",
            crate::phase2_manifest::PHASE2_MANIFEST_SCHEMA_PATH,
            "schemas/crushr-info.v1.schema.json",
            "schemas/crushr-fsck.v1.schema.json",
            "schemas/crushr-impact.v1.schema.json",
            "schemas/crushr-extract-result.v1.schema.json",
            "schemas/crushr-propagation-graph.v1.schema.json",
        ] {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, b"{}\n").unwrap();
        }
    }

    #[test]
    fn audit_report_serializes_and_includes_readiness_summary() {
        let root = temp_path("audit_serialization");
        create_workspace_with_required_files(&root);
        let manifest_path = root.join("PHASE2_RESEARCH/manifests/phase2_core_manifest.json");
        write_manifest(&manifest_path, locked_manifest_value());

        let report = build_pretrial_audit_report(&root, &manifest_path).expect("audit report");
        let serialized = serde_json::to_value(&report).expect("serialize report");

        assert_eq!(serialized["audit_kind"], "phase2_pretrial_readiness_v1");
        assert!(serialized["readiness_summary"].is_object());
        assert_eq!(serialized["locked_matrix"]["expected_scenarios"], 2700);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn locked_matrix_check_rejects_duplicate_scenario_ids() {
        let mut manifest = locked_manifest_value();
        let scenarios = manifest
            .get_mut("scenarios")
            .and_then(Value::as_array_mut)
            .expect("scenarios");
        let duplicate_id = scenarios[0]["scenario_id"].as_str().unwrap().to_string();
        scenarios[1]["scenario_id"] = Value::String(duplicate_id);

        let check = check_locked_matrix_and_ids(&manifest);
        assert!(!check.ok);
        assert_eq!(check.failure_code, Some(AuditFailureCode::InvalidManifest));
        assert!(check.detail.contains("duplicate scenario IDs"));
    }

    #[test]
    fn missing_required_file_is_reported() {
        let root = temp_path("audit_missing_file");
        create_workspace_with_required_files(&root);
        fs::remove_file(root.join("PHASE2_RESEARCH/README.md")).unwrap();

        let checks = check_required_files(&root);
        let missing = checks
            .iter()
            .find(|check| check.name == "required_file:PHASE2_RESEARCH/README.md")
            .expect("readme check exists");

        assert!(!missing.ok);
        assert_eq!(
            missing.failure_code,
            Some(AuditFailureCode::MissingSupportFile)
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn output_root_probe_reports_unwritable_root() {
        let file_path = temp_path("audit_not_dir");
        fs::write(&file_path, b"not a dir").unwrap();

        let status = ensure_path_writable(&file_path);
        assert!(status.is_err());

        fs::remove_file(file_path).ok();
    }
}
