// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::phase2_domain::{
    ArchiveFormat, CorruptionType, Dataset, LOCKED_CORE_SEEDS, Magnitude,
    PHASE2_SCENARIO_ID_FORMAT, TargetClass,
};
use crate::phase2_manifest::{
    PHASE2_MANIFEST_SCHEMA_PATH, Phase2ExperimentManifest, validate_manifest_shape,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXPECTED_SCENARIO_COUNT: usize = 2700;
const PHASE2_LOCKS_PATH: &str = "PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md";
const DEFAULT_MANIFEST_PATH: &str = "PHASE2_RESEARCH/manifest/phase2_core_manifest.json";
const DEFAULT_AUDIT_ARTIFACT_DIR: &str = "PHASE2_RESEARCH/generated/audit";

#[derive(Debug, Clone, Serialize)]
pub struct PretrialAuditReport {
    pub pass: bool,
    pub summary: AuditSummary,
    pub failing_checks: Vec<String>,
    pub matrix: LockedMatrixSummary,
    pub checks: Vec<AuditCheck>,
    pub tools: Vec<ToolStatus>,
    pub output_roots: Vec<OutputRootStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditSummary {
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LockedMatrixSummary {
    pub datasets: Vec<String>,
    pub formats: Vec<String>,
    pub corruption_types: Vec<String>,
    pub targets: Vec<String>,
    pub magnitudes: Vec<String>,
    pub seeds: Vec<u64>,
    pub expected_scenario_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditCheck {
    pub category: AuditCategory,
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Manifest,
    Tooling,
    OutputRoot,
    SupportFile,
    DeterministicGeneration,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub format: ArchiveFormat,
    pub status: CheckStatus,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputRootStatus {
    pub path: String,
    pub status: CheckStatus,
    pub detail: String,
}

pub fn run_phase2_pretrial_audit_cmd(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let root = crate::cli::workspace_root()?;
    let mut manifest_path = root.join(DEFAULT_MANIFEST_PATH);
    let mut artifact_dir = root.join(DEFAULT_AUDIT_ARTIFACT_DIR);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                manifest_path = PathBuf::from(args.next().context("missing value for --manifest")?)
            }
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?)
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let report = run_phase2_pretrial_audit(&root, &manifest_path)?;
    fs::create_dir_all(&artifact_dir)?;
    let output = artifact_dir.join("phase2_pretrial_audit.json");
    fs::write(&output, serde_json::to_vec_pretty(&report)?)?;

    if !report.pass {
        bail!("pre-trial audit failed; see {}", output.to_string_lossy());
    }

    Ok(())
}

pub fn run_phase2_pretrial_audit(
    workspace_root: &Path,
    manifest_path: &Path,
) -> Result<PretrialAuditReport> {
    let matrix = locked_matrix_summary();
    let mut checks = Vec::new();

    let manifest_result = check_manifest(manifest_path);
    push_check_result(&mut checks, manifest_result);

    let tool_statuses = tool_statuses(workspace_root);
    for tool in &tool_statuses {
        checks.push(AuditCheck {
            category: AuditCategory::Tooling,
            name: format!("tool availability: {}", tool.format.slug()),
            status: tool.status,
            detail: tool.details.join("; "),
        });
    }

    let output_root_statuses = check_output_roots(workspace_root);
    for root in &output_root_statuses {
        checks.push(AuditCheck {
            category: AuditCategory::OutputRoot,
            name: format!("output root writable: {}", root.path),
            status: root.status,
            detail: root.detail.clone(),
        });
    }

    for relative in [PHASE2_LOCKS_PATH, PHASE2_MANIFEST_SCHEMA_PATH] {
        let path = workspace_root.join(relative);
        let exists = path.is_file();
        checks.push(AuditCheck {
            category: AuditCategory::SupportFile,
            name: format!("support file exists: {relative}"),
            status: if exists {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            detail: if exists {
                "found".to_string()
            } else {
                "missing required support file".to_string()
            },
        });
    }

    checks.push(AuditCheck {
        category: AuditCategory::DeterministicGeneration,
        name: "deterministic-generation prerequisite state exposure".to_string(),
        status: CheckStatus::Pass,
        detail: "no additional pre-trial deterministic-generation state is exposed by crushr-lab; baseline is validated via locked matrix + manifest checks".to_string(),
    });

    let mut failing_checks = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .map(|c| format!("{} ({:?})", c.name, c.category))
        .collect::<Vec<_>>();
    failing_checks.sort();
    let passed_checks = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count();
    let summary = AuditSummary {
        total_checks: checks.len(),
        passed_checks,
        failed_checks: checks.len() - passed_checks,
    };

    Ok(PretrialAuditReport {
        pass: failing_checks.is_empty(),
        summary,
        failing_checks,
        matrix,
        checks,
        tools: tool_statuses,
        output_roots: output_root_statuses,
    })
}

fn check_manifest(manifest_path: &Path) -> Result<()> {
    let raw = fs::read(manifest_path)
        .with_context(|| format!("reading manifest {}", manifest_path.display()))?;
    let manifest_value: Value = serde_json::from_slice(&raw)
        .with_context(|| format!("parsing manifest JSON {}", manifest_path.display()))?;
    validate_manifest_shape(&manifest_value)?;

    let manifest: Phase2ExperimentManifest = serde_json::from_value(manifest_value)
        .context("deserializing manifest into typed model")?;

    if manifest.scenario_id_format != PHASE2_SCENARIO_ID_FORMAT {
        bail!("scenario_id_format does not match locked Phase 2 format");
    }

    if manifest.scenarios.len() != EXPECTED_SCENARIO_COUNT {
        bail!("manifest scenario count is not 2700");
    }

    if manifest.datasets != Dataset::ordered_locked_core().to_vec() {
        bail!("manifest datasets do not match locked core matrix");
    }
    if manifest.formats != ArchiveFormat::ordered_locked_core().to_vec() {
        bail!("manifest formats do not match locked core matrix");
    }
    if manifest.corruption_types != CorruptionType::ordered_locked_core().to_vec() {
        bail!("manifest corruption types do not match locked core matrix");
    }
    if manifest.target_classes != TargetClass::ordered_locked_core().to_vec() {
        bail!("manifest target classes do not match locked core matrix");
    }
    if manifest.magnitudes != Magnitude::ordered_locked_core().to_vec() {
        bail!("manifest magnitudes do not match locked core matrix");
    }
    if manifest.seeds != LOCKED_CORE_SEEDS.to_vec() {
        bail!("manifest seeds do not match locked core matrix");
    }

    let mut ids = HashSet::new();
    for scenario in &manifest.scenarios {
        if !ids.insert(scenario.scenario_id.as_str()) {
            bail!("duplicate scenario_id detected: {}", scenario.scenario_id);
        }
    }

    Ok(())
}

fn push_check_result(checks: &mut Vec<AuditCheck>, check: Result<()>) {
    match check {
        Ok(()) => checks.push(AuditCheck {
            category: AuditCategory::Manifest,
            name: "manifest/schema/locked-matrix validation".to_string(),
            status: CheckStatus::Pass,
            detail:
                "manifest schema, scenario count, uniqueness, and locked matrix values are valid"
                    .to_string(),
        }),
        Err(err) => checks.push(AuditCheck {
            category: AuditCategory::Manifest,
            name: "manifest/schema/locked-matrix validation".to_string(),
            status: CheckStatus::Fail,
            detail: err.to_string(),
        }),
    }
}

fn tool_statuses(workspace_root: &Path) -> Vec<ToolStatus> {
    ArchiveFormat::ordered_locked_core()
        .iter()
        .copied()
        .map(|format| match format {
            ArchiveFormat::Crushr => {
                let status = check_command(
                    workspace_root,
                    "cargo",
                    &[
                        "run",
                        "-q",
                        "-p",
                        "crushr",
                        "--bin",
                        "crushr-info",
                        "--",
                        "--version",
                    ],
                );
                ToolStatus {
                    format,
                    status: status.0,
                    details: vec![status.1],
                }
            }
            ArchiveFormat::Zip => {
                let status = check_command(workspace_root, "zip", &["--help"]);
                ToolStatus {
                    format,
                    status: status.0,
                    details: vec![status.1],
                }
            }
            ArchiveFormat::TarZstd => {
                let tar = check_command(workspace_root, "tar", &["--help"]);
                let zstd = check_command(workspace_root, "zstd", &["--help"]);
                let status = if tar.0 == CheckStatus::Pass && zstd.0 == CheckStatus::Pass {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                };
                ToolStatus {
                    format,
                    status,
                    details: vec![format!("tar: {}", tar.1), format!("zstd: {}", zstd.1)],
                }
            }
            ArchiveFormat::TarGz | ArchiveFormat::TarXz => {
                let status = check_command(workspace_root, "tar", &["--help"]);
                ToolStatus {
                    format,
                    status: status.0,
                    details: vec![status.1],
                }
            }
        })
        .collect()
}

fn check_command(workspace_root: &Path, program: &str, args: &[&str]) -> (CheckStatus, String) {
    let mut cmd = Command::new(program);
    cmd.current_dir(workspace_root).args(args);
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                (CheckStatus::Pass, "available".to_string())
            } else {
                (
                    CheckStatus::Fail,
                    format!(
                        "present but returned non-zero status ({})",
                        output.status.code().unwrap_or(-1)
                    ),
                )
            }
        }
        Err(err) => (CheckStatus::Fail, format!("missing tool ({err})")),
    }
}

fn check_output_roots(workspace_root: &Path) -> Vec<OutputRootStatus> {
    [
        "PHASE2_RESEARCH/manifest",
        "PHASE2_RESEARCH/generated/foundation",
        "PHASE2_RESEARCH/generated/execution",
        "PHASE2_RESEARCH/generated/audit",
    ]
    .iter()
    .map(|relative| {
        let path = workspace_root.join(relative);
        match fs::create_dir_all(&path) {
            Ok(()) => {
                let probe = path.join(".write_probe");
                match fs::write(&probe, b"ok") {
                    Ok(()) => {
                        let _ = fs::remove_file(probe);
                        OutputRootStatus {
                            path: relative.to_string(),
                            status: CheckStatus::Pass,
                            detail: "writable".to_string(),
                        }
                    }
                    Err(err) => OutputRootStatus {
                        path: relative.to_string(),
                        status: CheckStatus::Fail,
                        detail: format!("unwritable output root ({err})"),
                    },
                }
            }
            Err(err) => OutputRootStatus {
                path: relative.to_string(),
                status: CheckStatus::Fail,
                detail: format!("failed to create output root ({err})"),
            },
        }
    })
    .collect()
}

fn locked_matrix_summary() -> LockedMatrixSummary {
    LockedMatrixSummary {
        datasets: Dataset::ordered_locked_core()
            .iter()
            .map(|v| v.slug().to_string())
            .collect(),
        formats: ArchiveFormat::ordered_locked_core()
            .iter()
            .map(|v| v.slug().to_string())
            .collect(),
        corruption_types: CorruptionType::ordered_locked_core()
            .iter()
            .map(|v| v.slug().to_string())
            .collect(),
        targets: TargetClass::ordered_locked_core()
            .iter()
            .map(|v| v.slug().to_string())
            .collect(),
        magnitudes: Magnitude::ordered_locked_core()
            .iter()
            .map(|v| v.slug().to_string())
            .collect(),
        seeds: LOCKED_CORE_SEEDS.to_vec(),
        expected_scenario_count: EXPECTED_SCENARIO_COUNT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase2_manifest::{PHASE2_MANIFEST_SCHEMA_ID, Phase2ExperimentManifest};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("crushr_lab_{label}_{ts}"))
    }

    fn write_manifest(path: &Path, mut manifest: Value) {
        manifest.as_object_mut().unwrap().insert(
            "$schema".to_string(),
            Value::String(PHASE2_MANIFEST_SCHEMA_ID.to_string()),
        );
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }

    #[test]
    fn audit_report_serializes_machine_readable_output() {
        let report = PretrialAuditReport {
            pass: true,
            summary: AuditSummary {
                total_checks: 1,
                passed_checks: 1,
                failed_checks: 0,
            },
            failing_checks: Vec::new(),
            matrix: locked_matrix_summary(),
            checks: vec![AuditCheck {
                category: AuditCategory::Manifest,
                name: "manifest".to_string(),
                status: CheckStatus::Pass,
                detail: "ok".to_string(),
            }],
            tools: Vec::new(),
            output_roots: Vec::new(),
        };

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["pass"], true);
        assert_eq!(json["summary"]["failed_checks"], 0);
        assert_eq!(json["checks"][0]["status"], "pass");
    }

    #[test]
    fn manifest_check_detects_wrong_scenario_count() {
        let workspace = temp_dir("audit_wrong_count");
        let manifest_path = workspace.join(DEFAULT_MANIFEST_PATH);
        let mut manifest_json =
            serde_json::to_value(Phase2ExperimentManifest::locked_core()).unwrap();
        manifest_json
            .as_object_mut()
            .unwrap()
            .get_mut("scenarios")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .pop();
        write_manifest(&manifest_path, manifest_json);

        let err = check_manifest(&manifest_path).unwrap_err();
        assert!(err.to_string().contains("2700"));

        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn manifest_check_detects_duplicate_scenario_ids() {
        let workspace = temp_dir("audit_dup_ids");
        let manifest_path = workspace.join(DEFAULT_MANIFEST_PATH);
        let mut manifest_json =
            serde_json::to_value(Phase2ExperimentManifest::locked_core()).unwrap();
        let scenarios = manifest_json
            .as_object_mut()
            .unwrap()
            .get_mut("scenarios")
            .unwrap()
            .as_array_mut()
            .unwrap();
        scenarios[1]["scenario_id"] = scenarios[0]["scenario_id"].clone();
        write_manifest(&manifest_path, manifest_json);

        let err = check_manifest(&manifest_path).unwrap_err();
        assert!(
            err.to_string().contains("ordering differs")
                || err.to_string().contains("duplicate scenario_id")
        );

        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn run_audit_happy_path_with_local_support_files() {
        let workspace = temp_dir("audit_happy");
        fs::create_dir_all(workspace.join("PHASE2_RESEARCH/methodology")).unwrap();
        fs::create_dir_all(workspace.join("schemas")).unwrap();
        fs::write(workspace.join(PHASE2_LOCKS_PATH), "# lock\n").unwrap();
        fs::write(
            workspace.join(PHASE2_MANIFEST_SCHEMA_PATH),
            "{\"$id\":\"test\"}",
        )
        .unwrap();

        let manifest_path = workspace.join(DEFAULT_MANIFEST_PATH);
        write_manifest(
            &manifest_path,
            serde_json::to_value(Phase2ExperimentManifest::locked_core()).unwrap(),
        );

        let report = run_phase2_pretrial_audit(&workspace, &manifest_path).unwrap();
        assert_eq!(report.matrix.expected_scenario_count, 2700);
        assert!(!report.checks.is_empty());

        fs::remove_dir_all(workspace).ok();
    }
}
