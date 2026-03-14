use crate::phase2_corruption::{apply_locked_corruption, CorruptionRequest};
use crate::phase2_domain::{ArchiveFormat, CorruptionType, Dataset, TargetClass};
use crate::phase2_foundation::{ArchiveBuildRecord, ExecutionStatus, Phase2FoundationReport};
use crate::phase2_manifest::{Phase2ExperimentManifest, Phase2Scenario};
use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_MANIFEST_PATH: &str = "PHASE2_RESEARCH/manifest/phase2_manifest.json";
const DEFAULT_FOUNDATION_REPORT_PATH: &str = "PHASE2_RESEARCH/foundation/foundation_report.json";
const DEFAULT_ARTIFACT_DIR: &str = "PHASE2_RESEARCH/trials";

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionMetadata {
    pub invocation: InvocationMetadata,
    pub source_archive_path: String,
    pub corrupted_archive_path: String,
    pub corruption_log_path: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvocationMetadata {
    pub tool_kind: ArchiveFormat,
    pub executable: String,
    pub argv: Vec<String>,
    pub cwd: Option<String>,
    pub exit_status_code: i32,
    pub stdout_artifact_path: String,
    pub stderr_artifact_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawRunRecord {
    pub scenario_id: String,
    pub dataset: Dataset,
    pub format: ArchiveFormat,
    pub corruption_type: CorruptionType,
    pub target_class: TargetClass,
    pub magnitude_bytes: u64,
    pub seed: u64,
    pub exit_code: i32,
    pub stdout_path: String,
    pub stderr_path: String,
    pub json_result_path: Option<String>,
    pub tool_version: String,
    pub execution_metadata: ExecutionMetadata,
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

    let mut version_cache = HashMap::new();
    let mut records = Vec::with_capacity(manifest.scenarios.len());
    for scenario in &manifest.scenarios {
        let record = execute_scenario(
            workspace_root,
            artifact_root,
            &raw_root,
            scenario,
            &archive_index,
            &mut version_cache,
        )?;
        records.push(record);
    }

    let audit = audit_completeness(manifest, &records);

    let records_path = artifact_root.join("raw_run_records.json");
    fs::write(&records_path, serde_json::to_vec_pretty(&records)?)?;

    let completeness_path = artifact_root.join("completeness_audit.json");
    fs::write(&completeness_path, serde_json::to_vec_pretty(&audit)?)?;

    if !audit.missing_runs.is_empty()
        || !audit.duplicate_runs.is_empty()
        || !audit.mismatched_scenario_ids.is_empty()
    {
        bail!(
            "completeness audit failed; see {}",
            completeness_path.display()
        );
    }

    Ok(Phase2ExecutionReport {
        expected_runs: manifest.scenarios.len(),
        actual_runs: records.len(),
        records_path: rel_path(artifact_root, &records_path),
        completeness_path: rel_path(artifact_root, &completeness_path),
    })
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
    version_cache: &mut HashMap<ArchiveFormat, String>,
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

    let mut cmd = observe_command(workspace_root, format, &corrupted_archive)?;
    let executable = cmd.get_program().to_string_lossy().to_string();
    let argv = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let cwd = cmd
        .get_current_dir()
        .map(|path| path.to_string_lossy().to_string());
    let started_unix_ms = now_ms()?;
    let output = cmd
        .output()
        .with_context(|| format!("observing scenario {}", scenario.scenario_id))?;
    let finished_unix_ms = now_ms()?;

    let stdout_path = scenario_dir.join("stdout.txt");
    let stderr_path = scenario_dir.join("stderr.txt");
    fs::write(&stdout_path, &output.stdout)?;
    fs::write(&stderr_path, &output.stderr)?;
    let exit_status_code = output.status.code().unwrap_or(-1);

    let json_result_path = maybe_write_json_result(&scenario_dir, &output.stdout)?;
    let tool_version = if let Some(existing) = version_cache.get(&format) {
        existing.clone()
    } else {
        let fetched = detect_tool_version(workspace_root, format)?;
        version_cache.insert(format, fetched.clone());
        fetched
    };

    Ok(RawRunRecord {
        scenario_id: scenario.scenario_id.clone(),
        dataset: scenario.dataset,
        format,
        corruption_type: scenario.corruption_type,
        target_class: scenario.target_class,
        magnitude_bytes: scenario.magnitude_bytes,
        seed: scenario.seed,
        exit_code: exit_status_code,
        stdout_path: rel_path(artifact_root, &stdout_path),
        stderr_path: rel_path(artifact_root, &stderr_path),
        json_result_path: json_result_path.map(|p| rel_path(artifact_root, &p)),
        tool_version,
        execution_metadata: ExecutionMetadata {
            invocation: InvocationMetadata {
                tool_kind: format,
                executable,
                argv,
                cwd,
                exit_status_code,
                stdout_artifact_path: rel_path(artifact_root, &stdout_path),
                stderr_artifact_path: rel_path(artifact_root, &stderr_path),
            },
            source_archive_path: rel_path(artifact_root, &source_archive),
            corrupted_archive_path: rel_path(artifact_root, &corrupted_archive),
            corruption_log_path: rel_path(artifact_root, &corruption_log_path),
            started_unix_ms,
            finished_unix_ms,
        },
    })
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
                .arg("crushr-info")
                .arg("--")
                .arg(archive_path)
                .arg("--json");
            Ok(cmd)
        }
        ArchiveFormat::Zip => {
            let mut cmd = Command::new("zip");
            cmd.arg("-T").arg(archive_path);
            Ok(cmd)
        }
        ArchiveFormat::TarZstd => {
            let mut cmd = Command::new("tar");
            cmd.arg("--use-compress-program=zstd")
                .arg("-tf")
                .arg(archive_path);
            Ok(cmd)
        }
        ArchiveFormat::TarGz => {
            let mut cmd = Command::new("tar");
            cmd.arg("-tzf").arg(archive_path);
            Ok(cmd)
        }
        ArchiveFormat::TarXz => {
            let mut cmd = Command::new("tar");
            cmd.arg("-tJf").arg(archive_path);
            Ok(cmd)
        }
    }
}

fn detect_tool_version(workspace_root: &Path, format: ArchiveFormat) -> Result<String> {
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
            c.arg("--version");
            c
        }
        ArchiveFormat::TarZstd => {
            let mut c = Command::new("zstd");
            c.arg("--version");
            c
        }
        ArchiveFormat::TarGz | ArchiveFormat::TarXz => {
            let mut c = Command::new("tar");
            c.arg("--version");
            c
        }
    };

    let out = cmd.output();
    match out {
        Ok(o) => {
            let combined = if o.stdout.is_empty() {
                String::from_utf8_lossy(&o.stderr).into_owned()
            } else {
                String::from_utf8_lossy(&o.stdout).into_owned()
            };
            Ok(combined
                .lines()
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string())
        }
        Err(e) => Ok(format!("unavailable: {e}")),
    }
}

fn now_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!(e))?
        .as_millis())
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

    #[test]
    fn completeness_audit_detects_missing_duplicate_and_mismatch() {
        let manifest = Phase2ExperimentManifest::locked_core();
        let first = &manifest.scenarios[0];
        let second = &manifest.scenarios[1];
        let unknown = "p2-core-unknown-crushr-bit_flip-header-1B-999";

        let base = RawRunRecord {
            scenario_id: first.scenario_id.clone(),
            dataset: Dataset::Smallfiles,
            format: ArchiveFormat::Crushr,
            corruption_type: first.corruption_type,
            target_class: first.target_class,
            magnitude_bytes: first.magnitude_bytes,
            seed: first.seed,
            exit_code: 0,
            stdout_path: "raw/x/stdout.txt".to_string(),
            stderr_path: "raw/x/stderr.txt".to_string(),
            json_result_path: None,
            tool_version: "tool 1.0".to_string(),
            execution_metadata: ExecutionMetadata {
                invocation: InvocationMetadata {
                    tool_kind: ArchiveFormat::Crushr,
                    executable: "cargo".to_string(),
                    argv: vec!["run".to_string()],
                    cwd: Some("/workspace/crushr".to_string()),
                    exit_status_code: 0,
                    stdout_artifact_path: "raw/x/stdout.txt".to_string(),
                    stderr_artifact_path: "raw/x/stderr.txt".to_string(),
                },
                source_archive_path: "archives/a".to_string(),
                corrupted_archive_path: "raw/x/corrupt".to_string(),
                corruption_log_path: "raw/x/log".to_string(),
                started_unix_ms: 1,
                finished_unix_ms: 2,
            },
        };

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
