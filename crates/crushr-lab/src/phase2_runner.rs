// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::phase2_corruption::{apply_locked_corruption, CorruptionRequest};
use crate::phase2_domain::{ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass};
use crate::phase2_foundation::{
    ArchiveBuildRecord, DatasetInventory, ExecutionStatus, FileInventoryEntry,
    Phase2FoundationReport,
};
use crate::phase2_manifest::{Phase2ExperimentManifest, Phase2Scenario};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_MANIFEST_PATH: &str = "PHASE2_RESEARCH/manifest/phase2_manifest.json";
const DEFAULT_FOUNDATION_REPORT_PATH: &str = "PHASE2_RESEARCH/foundation/foundation_report.json";
const DEFAULT_ARTIFACT_DIR: &str = "PHASE2_RESEARCH/trials";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultArtifacts {
    pub stdout_path: String,
    pub stderr_path: String,
    pub json_result_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationStatus {
    Completed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQuality {
    StdoutOnly,
    StdoutAndStderr,
    StructuredJsonResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolVersionStatus {
    Detected,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionObservation {
    pub status: ToolVersionStatus,
    pub version: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolVersionSummary {
    pub tool_kind: ArchiveFormat,
    pub executable: String,
    pub status: ToolVersionStatus,
    pub version: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonResultCounts {
    pub true_count: usize,
    pub false_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionReportToolVersions {
    pub by_tool: Vec<ToolVersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContextPaths {
    pub source_archive_path: String,
    pub corrupted_archive_path: String,
    pub corruption_log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAccounting {
    pub files_expected: u64,
    pub files_recovered: u64,
    pub files_missing: u64,
    pub bytes_expected: u64,
    pub bytes_recovered: u64,
    pub recovery_ratio_files: f64,
    pub recovery_ratio_bytes: f64,
}

impl Default for RecoveryAccounting {
    fn default() -> Self {
        Self {
            files_expected: 0,
            files_recovered: 0,
            files_missing: 0,
            bytes_expected: 0,
            bytes_recovered: 0,
            recovery_ratio_files: 0.0,
            recovery_ratio_bytes: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub scenario_id: String,
    pub dataset: Dataset,
    pub extraction_output_dir: String,
    pub missing_files: Vec<String>,
    pub present_files: Vec<String>,
    pub accounting: RecoveryAccounting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRunRecord {
    pub scenario_id: String,
    pub dataset: Dataset,
    pub format: ArchiveFormat,
    pub corruption_type: CorruptionType,
    pub target_class: TargetClass,
    pub magnitude: Magnitude,
    pub magnitude_bytes: u64,
    pub seed: u64,
    pub source_archive_path: String,
    pub corrupted_archive_path: String,
    pub tool_kind: ArchiveFormat,
    pub executable: String,
    pub argv: Vec<String>,
    pub cwd: Option<String>,
    pub exit_code: i32,
    pub stdout_path: String,
    pub stderr_path: String,
    pub json_result_path: Option<String>,
    pub has_json_result: bool,
    pub invocation_status: InvocationStatus,
    pub stage_classification: Option<String>,
    pub tool_version: ToolVersionObservation,
    pub result_artifacts: ResultArtifacts,
    pub result_completeness: EvidenceQuality,
    pub run_context_paths: RunContextPaths,
    #[serde(default)]
    pub extraction_output_dir: String,
    #[serde(default)]
    pub recovery_report_path: String,
    #[serde(default)]
    pub recovery_accounting: RecoveryAccounting,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletenessAudit {
    pub expected_runs: usize,
    pub actual_runs: usize,
    pub missing_runs: Vec<String>,
    pub duplicate_runs: Vec<String>,
    pub mismatched_scenario_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phase2ExecutionReport {
    pub expected_runs: usize,
    pub actual_runs: usize,
    pub records_path: String,
    pub completeness_path: String,
    pub scenario_count_by_format: BTreeMap<String, usize>,
    pub scenario_count_by_dataset: BTreeMap<String, usize>,
    pub exit_code_histogram: BTreeMap<String, usize>,
    pub has_json_result_counts: JsonResultCounts,
    pub tool_versions: ExecutionReportToolVersions,
    pub completeness_audit_passed: bool,
}

pub fn run_phase2_execution_cmd(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let root = crate::cli::workspace_root()?;
    let mut manifest_path = root.join(DEFAULT_MANIFEST_PATH);
    let mut foundation_report_path = root.join(DEFAULT_FOUNDATION_REPORT_PATH);
    let mut artifact_dir = root.join(DEFAULT_ARTIFACT_DIR);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                manifest_path = PathBuf::from(args.next().context("missing value for --manifest")?)
            }
            "--foundation-report" => {
                foundation_report_path = PathBuf::from(
                    args.next()
                        .context("missing value for --foundation-report")?,
                )
            }
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?)
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let manifest: Phase2ExperimentManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
        .with_context(|| format!("parsing manifest {}", manifest_path.display()))?;
    let foundation: Phase2FoundationReport =
        serde_json::from_slice(&fs::read(&foundation_report_path)?).with_context(|| {
            format!(
                "parsing foundation report {}",
                foundation_report_path.display()
            )
        })?;

    let report = run_phase2_execution(&root, &manifest, &foundation, &artifact_dir)?;
    fs::write(
        artifact_dir.join("execution_report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

pub fn run_phase2_execution(
    workspace_root: &Path,
    manifest: &Phase2ExperimentManifest,
    foundation: &Phase2FoundationReport,
    artifact_root: &Path,
) -> Result<Phase2ExecutionReport> {
    fs::create_dir_all(artifact_root)?;
    let raw_root = artifact_root.join("raw");
    fs::create_dir_all(&raw_root)?;

    let mut archive_index = HashMap::new();
    for build in &foundation.archive_builds {
        archive_index.insert((build.dataset, build.archive_format), build);
    }

    let inventory_index = load_inventory_index(workspace_root, foundation)?;

    let mut version_cache = HashMap::new();
    let mut records = Vec::with_capacity(manifest.scenarios.len());
    for scenario in &manifest.scenarios {
        let record = execute_scenario(
            workspace_root,
            artifact_root,
            &raw_root,
            scenario,
            &archive_index,
            &inventory_index,
            &mut version_cache,
        )?;
        records.push(record);
    }

    let audit = audit_completeness(manifest, &records);

    let records_path = artifact_root.join("raw_run_records.json");
    fs::write(&records_path, serde_json::to_vec_pretty(&records)?)?;

    let completeness_path = artifact_root.join("completeness_audit.json");
    fs::write(&completeness_path, serde_json::to_vec_pretty(&audit)?)?;

    let report = build_execution_report(
        manifest,
        &records,
        &audit,
        artifact_root,
        &records_path,
        &completeness_path,
    );

    if !report.completeness_audit_passed
        || !audit.duplicate_runs.is_empty()
        || !audit.mismatched_scenario_ids.is_empty()
    {
        bail!(
            "completeness audit failed; see {}",
            completeness_path.display()
        );
    }

    Ok(report)
}

fn build_execution_report(
    manifest: &Phase2ExperimentManifest,
    records: &[RawRunRecord],
    audit: &CompletenessAudit,
    artifact_root: &Path,
    records_path: &Path,
    completeness_path: &Path,
) -> Phase2ExecutionReport {
    let mut scenario_count_by_format = BTreeMap::new();
    let mut scenario_count_by_dataset = BTreeMap::new();
    let mut exit_code_histogram = BTreeMap::new();
    let mut has_json_true = 0usize;
    let mut tool_versions: BTreeMap<String, ToolVersionSummary> = BTreeMap::new();

    for record in records {
        *scenario_count_by_format
            .entry(record.format.slug().to_string())
            .or_insert(0) += 1;
        *scenario_count_by_dataset
            .entry(record.dataset.slug().to_string())
            .or_insert(0) += 1;
        *exit_code_histogram
            .entry(record.exit_code.to_string())
            .or_insert(0) += 1;
        if record.has_json_result {
            has_json_true += 1;
        }

        tool_versions
            .entry(record.tool_kind.slug().to_string())
            .or_insert_with(|| ToolVersionSummary {
                tool_kind: record.tool_kind,
                executable: record.executable.clone(),
                status: record.tool_version.status,
                version: record.tool_version.version.clone(),
                detail: record.tool_version.detail.clone(),
            });
    }

    Phase2ExecutionReport {
        expected_runs: manifest.scenarios.len(),
        actual_runs: records.len(),
        records_path: rel_path(artifact_root, records_path),
        completeness_path: rel_path(artifact_root, completeness_path),
        scenario_count_by_format,
        scenario_count_by_dataset,
        exit_code_histogram,
        has_json_result_counts: JsonResultCounts {
            true_count: has_json_true,
            false_count: records.len().saturating_sub(has_json_true),
        },
        tool_versions: ExecutionReportToolVersions {
            by_tool: tool_versions.into_values().collect(),
        },
        completeness_audit_passed: audit.missing_runs.is_empty()
            && audit.duplicate_runs.is_empty()
            && audit.mismatched_scenario_ids.is_empty(),
    }
}

pub fn audit_completeness(
    manifest: &Phase2ExperimentManifest,
    records: &[RawRunRecord],
) -> CompletenessAudit {
    let expected_ids: HashSet<&str> = manifest
        .scenarios
        .iter()
        .map(|s| s.scenario_id.as_str())
        .collect();

    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    let mut mismatched = Vec::new();

    for record in records {
        let id = record.scenario_id.as_str();
        if !seen.insert(id) {
            duplicates.push(record.scenario_id.clone());
        }
        if !expected_ids.contains(id) {
            mismatched.push(record.scenario_id.clone());
        }
    }

    let recorded_ids: HashSet<&str> = records.iter().map(|r| r.scenario_id.as_str()).collect();
    let mut missing = manifest
        .scenarios
        .iter()
        .filter_map(|s| {
            (!recorded_ids.contains(s.scenario_id.as_str())).then_some(s.scenario_id.clone())
        })
        .collect::<Vec<_>>();

    missing.sort();
    duplicates.sort();
    mismatched.sort();

    CompletenessAudit {
        expected_runs: manifest.scenarios.len(),
        actual_runs: records.len(),
        missing_runs: missing,
        duplicate_runs: duplicates,
        mismatched_scenario_ids: mismatched,
    }
}

fn execute_scenario(
    workspace_root: &Path,
    artifact_root: &Path,
    raw_root: &Path,
    scenario: &Phase2Scenario,
    archive_index: &HashMap<(Dataset, ArchiveFormat), &ArchiveBuildRecord>,
    inventory_index: &HashMap<Dataset, DatasetInventory>,
    version_cache: &mut HashMap<ArchiveFormat, ToolVersionObservation>,
) -> Result<RawRunRecord> {
    let build = archive_index
        .get(&(scenario.dataset, scenario.format))
        .with_context(|| {
            format!(
                "missing archive build for dataset={} format={}",
                scenario.dataset.slug(),
                scenario.format.slug()
            )
        })?;
    if !matches!(build.build.status, ExecutionStatus::Success) {
        bail!(
            "archive build for {} {} is not successful",
            scenario.dataset.slug(),
            scenario.format.slug()
        );
    }

    let format = scenario.format;
    let source_archive = resolve_source_archive_path(workspace_root, &build.output_path);
    let scenario_dir = scenario_artifact_dir(raw_root, &scenario.scenario_id);
    fs::create_dir_all(&scenario_dir)?;

    let corrupted_archive = scenario_dir.join(format!(
        "{}.corrupt",
        source_archive.file_name().unwrap().to_string_lossy()
    ));
    let source_bytes = fs::read(&source_archive)
        .with_context(|| format!("reading source archive {}", source_archive.display()))?;

    let (mutated_bytes, provenance) = apply_locked_corruption(
        &source_bytes,
        &CorruptionRequest {
            source_archive: rel_path(artifact_root, &source_archive),
            scenario_id: scenario.scenario_id.clone(),
            corruption_type: scenario.corruption_type,
            target: scenario.target_class,
            magnitude: scenario.magnitude,
            seed: scenario.seed,
            forced_offset: None,
        },
    )?;
    fs::write(&corrupted_archive, mutated_bytes)?;

    let corruption_log_path = scenario_dir.join("corruption_provenance.json");
    fs::write(
        &corruption_log_path,
        serde_json::to_vec_pretty(&provenance)?,
    )?;

    let extraction_dir = scenario_dir.join("extracted");
    fs::create_dir_all(&extraction_dir)?;
    let mut cmd = observe_command(workspace_root, format, &corrupted_archive, &extraction_dir)?;
    let executable = cmd.get_program().to_string_lossy().to_string();
    let argv = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let cwd = cmd
        .get_current_dir()
        .map(|path| path.to_string_lossy().to_string());
    let output = cmd
        .output()
        .with_context(|| format!("observing scenario {}", scenario.scenario_id))?;

    let stdout_path = scenario_dir.join("stdout.txt");
    let stderr_path = scenario_dir.join("stderr.txt");
    fs::write(&stdout_path, &output.stdout)?;
    fs::write(&stderr_path, &output.stderr)?;
    let exit_status_code = output.status.code().unwrap_or(-1);

    let json_result_path = maybe_write_json_result(&scenario_dir, &output.stdout)?;
    let recovery_report_path = scenario_dir.join("recovery_report.json");
    let recovery_report = build_recovery_report(
        workspace_root,
        scenario,
        &extraction_dir,
        inventory_index.get(&scenario.dataset).with_context(|| {
            format!("missing dataset inventory for {}", scenario.dataset.slug())
        })?,
    )?;
    fs::write(
        &recovery_report_path,
        serde_json::to_vec_pretty(&recovery_report)?,
    )?;
    let tool_version = if let Some(existing) = version_cache.get(&format) {
        existing.clone()
    } else {
        let fetched = detect_tool_version(workspace_root, format)?;
        version_cache.insert(format, fetched.clone());
        fetched
    };

    let stdout_rel = rel_path(artifact_root, &stdout_path);
    let stderr_rel = rel_path(artifact_root, &stderr_path);
    let json_rel = json_result_path.map(|p| rel_path(artifact_root, &p));
    let source_archive_rel = rel_path(artifact_root, &source_archive);
    let corrupted_archive_rel = rel_path(artifact_root, &corrupted_archive);
    let corruption_log_rel = rel_path(artifact_root, &corruption_log_path);
    let extraction_dir_rel = rel_path(artifact_root, &extraction_dir);
    let recovery_report_rel = rel_path(artifact_root, &recovery_report_path);
    let has_json_result = json_rel.is_some();
    let result_completeness = if has_json_result {
        EvidenceQuality::StructuredJsonResult
    } else if output.stderr.is_empty() {
        EvidenceQuality::StdoutOnly
    } else {
        EvidenceQuality::StdoutAndStderr
    };

    Ok(RawRunRecord {
        scenario_id: scenario.scenario_id.clone(),
        dataset: scenario.dataset,
        format,
        corruption_type: scenario.corruption_type,
        target_class: scenario.target_class,
        magnitude: scenario.magnitude,
        magnitude_bytes: scenario.magnitude_bytes,
        seed: scenario.seed,
        source_archive_path: source_archive_rel.clone(),
        corrupted_archive_path: corrupted_archive_rel.clone(),
        tool_kind: format,
        executable: executable.clone(),
        argv: argv.clone(),
        cwd: cwd.clone(),
        exit_code: exit_status_code,
        stdout_path: stdout_rel.clone(),
        stderr_path: stderr_rel.clone(),
        json_result_path: json_rel.clone(),
        has_json_result,
        invocation_status: InvocationStatus::Completed,
        stage_classification: None,
        tool_version,
        result_artifacts: ResultArtifacts {
            stdout_path: stdout_rel,
            stderr_path: stderr_rel,
            json_result_path: json_rel,
        },
        result_completeness,
        run_context_paths: RunContextPaths {
            source_archive_path: source_archive_rel,
            corrupted_archive_path: corrupted_archive_rel,
            corruption_log_path: corruption_log_rel,
        },
        extraction_output_dir: extraction_dir_rel,
        recovery_report_path: recovery_report_rel,
        recovery_accounting: recovery_report.accounting,
    })
}

fn load_inventory_index(
    workspace_root: &Path,
    foundation: &Phase2FoundationReport,
) -> Result<HashMap<Dataset, DatasetInventory>> {
    let mut index = HashMap::new();
    for dataset in &foundation.datasets {
        let inventory_path = workspace_root.join(&dataset.inventory_path);
        let inventory: DatasetInventory = serde_json::from_slice(
            &fs::read(&inventory_path)
                .with_context(|| format!("reading {}", inventory_path.display()))?,
        )
        .with_context(|| format!("parsing {}", inventory_path.display()))?;
        index.insert(inventory.dataset, inventory);
    }
    Ok(index)
}

fn build_recovery_report(
    workspace_root: &Path,
    scenario: &Phase2Scenario,
    extraction_dir: &Path,
    inventory: &DatasetInventory,
) -> Result<RecoveryReport> {
    let extracted = collect_extracted_file_sizes(extraction_dir)?;
    let mut missing_files = Vec::new();
    let mut present_files = Vec::new();
    let mut bytes_recovered = 0_u64;

    for expected in &inventory.files {
        if let Some(actual_bytes) = extracted.get(expected.path.as_str()) {
            present_files.push(expected.path.clone());
            bytes_recovered += recovered_bytes_for_file(expected, *actual_bytes);
        } else {
            missing_files.push(expected.path.clone());
        }
    }

    missing_files.sort();
    present_files.sort();
    let files_expected = inventory.file_count as u64;
    let files_recovered = present_files.len() as u64;
    let files_missing = files_expected.saturating_sub(files_recovered);
    let bytes_expected = inventory.total_bytes;

    let accounting = RecoveryAccounting {
        files_expected,
        files_recovered,
        files_missing,
        bytes_expected,
        bytes_recovered,
        recovery_ratio_files: ratio(files_recovered, files_expected),
        recovery_ratio_bytes: ratio(bytes_recovered, bytes_expected),
    };

    Ok(RecoveryReport {
        scenario_id: scenario.scenario_id.clone(),
        dataset: scenario.dataset,
        extraction_output_dir: rel_path(workspace_root, extraction_dir),
        missing_files,
        present_files,
        accounting,
    })
}

fn recovered_bytes_for_file(expected: &FileInventoryEntry, actual_bytes: u64) -> u64 {
    expected.bytes.min(actual_bytes)
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn collect_extracted_file_sizes(root: &Path) -> Result<HashMap<String, u64>> {
    let mut files = HashMap::new();
    if !root.exists() {
        return Ok(files);
    }
    collect_extracted_file_sizes_inner(root, root, &mut files)?;
    Ok(files)
}

fn collect_extracted_file_sizes_inner(
    base: &Path,
    current: &Path,
    files: &mut HashMap<String, u64>,
) -> Result<()> {
    for entry in fs::read_dir(current).with_context(|| format!("reading {}", current.display()))? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_extracted_file_sizes_inner(base, &path, files)?;
        } else if metadata.is_file() {
            let rel = path
                .strip_prefix(base)
                .with_context(|| format!("computing relative path for {}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(rel, metadata.len());
        }
    }
    Ok(())
}

fn maybe_write_json_result(scenario_dir: &Path, stdout: &[u8]) -> Result<Option<PathBuf>> {
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(stdout) else {
        return Ok(None);
    };
    let path = scenario_dir.join("result.json");
    fs::write(&path, serde_json::to_vec_pretty(&json)?)?;
    Ok(Some(path))
}

fn resolve_source_archive_path(workspace_root: &Path, archive_path: &str) -> PathBuf {
    let archive_path = PathBuf::from(archive_path);
    if archive_path.is_absolute() {
        archive_path
    } else {
        workspace_root.join(archive_path)
    }
}

fn scenario_artifact_dir(raw_root: &Path, scenario_id: &str) -> PathBuf {
    raw_root.join(scenario_id)
}

fn observe_command(
    workspace_root: &Path,
    format: ArchiveFormat,
    archive_path: &Path,
    extraction_dir: &Path,
) -> Result<Command> {
    match format {
        ArchiveFormat::Crushr => {
            let mut cmd = Command::new("cargo");
            cmd.current_dir(workspace_root)
                .arg("run")
                .arg("-q")
                .arg("-p")
                .arg("crushr")
                .arg("--bin")
                .arg("crushr-extract")
                .arg("--")
                .arg(archive_path)
                .arg("-o")
                .arg(extraction_dir)
                .arg("--overwrite")
                .arg("--json");
            Ok(cmd)
        }
        ArchiveFormat::Zip => {
            let mut cmd = Command::new("unzip");
            cmd.arg("-o")
                .arg(archive_path)
                .arg("-d")
                .arg(extraction_dir);
            Ok(cmd)
        }
        ArchiveFormat::TarZstd => {
            let mut cmd = Command::new("tar");
            cmd.arg("--use-compress-program=zstd")
                .arg("-xf")
                .arg(archive_path);
            cmd.arg("-C").arg(extraction_dir);
            Ok(cmd)
        }
        ArchiveFormat::TarGz => {
            let mut cmd = Command::new("tar");
            cmd.arg("-xzf")
                .arg(archive_path)
                .arg("-C")
                .arg(extraction_dir);
            Ok(cmd)
        }
        ArchiveFormat::TarXz => {
            let mut cmd = Command::new("tar");
            cmd.arg("-xJf")
                .arg(archive_path)
                .arg("-C")
                .arg(extraction_dir);
            Ok(cmd)
        }
    }
}

fn detect_tool_version(
    workspace_root: &Path,
    format: ArchiveFormat,
) -> Result<ToolVersionObservation> {
    let mut cmd = match format {
        ArchiveFormat::Crushr => {
            let mut c = Command::new("cargo");
            c.current_dir(workspace_root)
                .arg("run")
                .arg("-q")
                .arg("-p")
                .arg("crushr")
                .arg("--bin")
                .arg("crushr-info")
                .arg("--")
                .arg("--version");
            c
        }
        ArchiveFormat::Zip => {
            let mut c = Command::new("zip");
            c.arg("-v");
            c
        }
        ArchiveFormat::TarZstd => {
            let mut c = Command::new("zstd");
            c.arg("--version");
            c
        }
        ArchiveFormat::TarGz | ArchiveFormat::TarXz => return Ok(ToolVersionObservation {
            status: ToolVersionStatus::Unsupported,
            version: None,
            detail: Some("version is shared with tar executable; captured under tar+gz/tar+xz command observations".to_string()),
        }),
    };

    let out = cmd.output();
    match out {
        Ok(o) => {
            parse_tool_version_output(o.status.success(), &o.stdout, &o.stderr, o.status.code())
        }
        Err(e) => Ok(ToolVersionObservation {
            status: ToolVersionStatus::Unavailable,
            version: None,
            detail: Some(e.to_string()),
        }),
    }
}

fn parse_tool_version_output(
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
    status_code: Option<i32>,
) -> Result<ToolVersionObservation> {
    if !success {
        return Ok(ToolVersionObservation {
            status: ToolVersionStatus::Unsupported,
            version: None,
            detail: Some(format!(
                "version command exited with status {status_code:?}"
            )),
        });
    }

    let combined = if stdout.is_empty() {
        String::from_utf8_lossy(stderr).into_owned()
    } else {
        String::from_utf8_lossy(stdout).into_owned()
    };
    let version = combined
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if version.is_empty() {
        Ok(ToolVersionObservation {
            status: ToolVersionStatus::Unsupported,
            version: None,
            detail: Some("version command returned empty output".to_string()),
        })
    } else {
        Ok(ToolVersionObservation {
            status: ToolVersionStatus::Detected,
            version: Some(version),
            detail: None,
        })
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase2_manifest::Phase2ExperimentManifest;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn base_record(manifest: &Phase2ExperimentManifest) -> RawRunRecord {
        let first = &manifest.scenarios[0];
        RawRunRecord {
            scenario_id: first.scenario_id.clone(),
            dataset: Dataset::Smallfiles,
            format: ArchiveFormat::Crushr,
            corruption_type: first.corruption_type,
            target_class: first.target_class,
            magnitude: first.magnitude,
            magnitude_bytes: first.magnitude_bytes,
            seed: first.seed,
            source_archive_path: "archives/a".to_string(),
            corrupted_archive_path: "raw/x/corrupt".to_string(),
            tool_kind: ArchiveFormat::Crushr,
            executable: "cargo".to_string(),
            argv: vec!["run".to_string()],
            cwd: Some("/workspace/crushr".to_string()),
            exit_code: 0,
            stdout_path: "raw/x/stdout.txt".to_string(),
            stderr_path: "raw/x/stderr.txt".to_string(),
            json_result_path: None,
            has_json_result: false,
            invocation_status: InvocationStatus::Completed,
            stage_classification: None,
            tool_version: ToolVersionObservation {
                status: ToolVersionStatus::Detected,
                version: Some("tool 1.0".to_string()),
                detail: None,
            },
            result_artifacts: ResultArtifacts {
                stdout_path: "raw/x/stdout.txt".to_string(),
                stderr_path: "raw/x/stderr.txt".to_string(),
                json_result_path: None,
            },
            result_completeness: EvidenceQuality::StdoutAndStderr,
            run_context_paths: RunContextPaths {
                source_archive_path: "archives/a".to_string(),
                corrupted_archive_path: "raw/x/corrupt".to_string(),
                corruption_log_path: "raw/x/log".to_string(),
            },
            extraction_output_dir: "raw/x/extracted".to_string(),
            recovery_report_path: "raw/x/recovery_report.json".to_string(),
            recovery_accounting: RecoveryAccounting {
                files_expected: 10,
                files_recovered: 10,
                files_missing: 0,
                bytes_expected: 1000,
                bytes_recovered: 1000,
                recovery_ratio_files: 1.0,
                recovery_ratio_bytes: 1.0,
            },
        }
    }

    #[test]
    fn completeness_audit_detects_missing_duplicate_and_mismatch() {
        let manifest = Phase2ExperimentManifest::locked_core();
        let first = &manifest.scenarios[0];
        let second = &manifest.scenarios[1];
        let unknown = "p2-core-unknown-crushr-bit_flip-header-1B-999";

        let base = base_record(&manifest);

        let mut dup = base.clone();
        dup.scenario_id = first.scenario_id.clone();

        let mut mismatch = base.clone();
        mismatch.scenario_id = unknown.to_string();

        let mut second_record = base.clone();
        second_record.scenario_id = second.scenario_id.clone();

        let audit = audit_completeness(&manifest, &[base, dup, second_record, mismatch]);

        assert!(audit.duplicate_runs.contains(&first.scenario_id));
        assert!(audit.mismatched_scenario_ids.contains(&unknown.to_string()));
        assert_eq!(audit.actual_runs, 4);
        assert!(!audit.missing_runs.is_empty());
    }

    #[test]
    fn execution_report_includes_histograms_and_tool_summary() {
        let manifest = Phase2ExperimentManifest::locked_core();
        let mut records = vec![base_record(&manifest), base_record(&manifest)];
        records[1].scenario_id = manifest.scenarios[1].scenario_id.clone();
        records[1].dataset = Dataset::Mixed;
        records[1].format = ArchiveFormat::Zip;
        records[1].tool_kind = ArchiveFormat::Zip;
        records[1].executable = "zip".to_string();
        records[1].exit_code = 2;
        records[1].has_json_result = true;
        records[1].json_result_path = Some("raw/y/result.json".to_string());
        records[1].tool_version = ToolVersionObservation {
            status: ToolVersionStatus::Unsupported,
            version: None,
            detail: Some("unsupported".to_string()),
        };

        let audit = CompletenessAudit {
            expected_runs: 2,
            actual_runs: 2,
            missing_runs: Vec::new(),
            duplicate_runs: Vec::new(),
            mismatched_scenario_ids: Vec::new(),
        };
        let report = build_execution_report(
            &manifest,
            &records,
            &audit,
            Path::new("/tmp/trials"),
            Path::new("/tmp/trials/raw_run_records.json"),
            Path::new("/tmp/trials/completeness_audit.json"),
        );

        assert_eq!(report.has_json_result_counts.true_count, 1);
        assert_eq!(report.exit_code_histogram.get("0"), Some(&1));
        assert_eq!(report.exit_code_histogram.get("2"), Some(&1));
        assert_eq!(report.scenario_count_by_dataset.get("smallfiles"), Some(&1));
        assert_eq!(report.scenario_count_by_dataset.get("mixed"), Some(&1));
        assert_eq!(report.tool_versions.by_tool.len(), 2);
    }

    #[test]
    fn tool_version_probe_for_tar_variants_is_explicitly_unsupported() {
        let observed =
            detect_tool_version(Path::new("/workspace/crushr"), ArchiveFormat::TarGz).unwrap();
        assert_eq!(observed.status, ToolVersionStatus::Unsupported);
        assert!(observed.version.is_none());
    }

    #[test]
    fn parse_tool_version_output_detects_supported_tool_output() {
        let observed =
            parse_tool_version_output(true, b"zip 3.0\nCopyright", b"", Some(0)).unwrap();
        assert_eq!(observed.status, ToolVersionStatus::Detected);
        assert_eq!(observed.version.as_deref(), Some("zip 3.0"));
    }

    #[test]
    fn parse_tool_version_output_marks_unsupported_probe() {
        let observed =
            parse_tool_version_output(false, b"", b"unsupported flag: --version", Some(2)).unwrap();
        assert_eq!(observed.status, ToolVersionStatus::Unsupported);
        assert!(observed.version.is_none());
    }

    #[test]
    fn json_result_is_written_only_for_valid_json_stdout() {
        let tmp = std::env::temp_dir().join(format!(
            "crushr_lab_runner_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&tmp).unwrap();

        let valid = maybe_write_json_result(&tmp, br#"{"ok":true}"#).unwrap();
        assert!(valid.is_some());

        let invalid = maybe_write_json_result(&tmp, b"not-json").unwrap();
        assert!(invalid.is_none());

        let _ = fs::remove_dir_all(tmp);
    }

    #[test]
    fn relative_source_archive_paths_resolve_against_workspace_root() {
        let root = Path::new("/workspace/crushr");
        let resolved =
            resolve_source_archive_path(root, "PHASE2_RESEARCH/baselines/crushr/smallfiles.crs");

        assert_eq!(
            resolved,
            PathBuf::from("/workspace/crushr/PHASE2_RESEARCH/baselines/crushr/smallfiles.crs")
        );
    }

    #[test]
    fn absolute_source_archive_paths_are_used_as_is() {
        let root = Path::new("/workspace/crushr");
        let absolute = "/tmp/archive/smallfiles.crs";
        let resolved = resolve_source_archive_path(root, absolute);

        assert_eq!(resolved, PathBuf::from(absolute));
    }

    #[test]
    fn scenario_outputs_stay_under_artifact_dir() {
        let artifact_root = Path::new("/tmp/trials");
        let raw_root = artifact_root.join("raw");
        let scenario_dir = scenario_artifact_dir(&raw_root, "p2-core-smallfiles-crushr");
        let stdout_path = scenario_dir.join("stdout.txt");

        assert!(scenario_dir.starts_with(artifact_root));
        assert_eq!(
            rel_path(artifact_root, &stdout_path),
            "raw/p2-core-smallfiles-crushr/stdout.txt"
        );
    }

    #[test]
    fn execution_defaults_match_canonical_phase2_layout() {
        assert_eq!(
            DEFAULT_MANIFEST_PATH,
            "PHASE2_RESEARCH/manifest/phase2_manifest.json"
        );
        assert_eq!(
            DEFAULT_FOUNDATION_REPORT_PATH,
            "PHASE2_RESEARCH/foundation/foundation_report.json"
        );
        assert_eq!(DEFAULT_ARTIFACT_DIR, "PHASE2_RESEARCH/trials");
    }
}
