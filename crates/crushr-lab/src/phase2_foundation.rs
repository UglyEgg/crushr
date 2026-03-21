// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::phase2_domain::{ArchiveFormat, Dataset};

pub fn run_phase2_foundation(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut artifact_dir = crate::cli::workspace_root()?.join("PHASE2_RESEARCH/foundation");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let root = crate::cli::workspace_root()?;
    let report = build_phase2_foundation(&root, &artifact_dir)?;
    validate_archive_coverage(&report)?;
    fs::write(
        artifact_dir.join("foundation_report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInventoryEntry {
    pub path: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInventory {
    pub dataset: Dataset,
    pub composition_rule: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub inventory_blake3: String,
    pub files: Vec<FileInventoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetProvenance {
    pub generator: String,
    pub deterministic: bool,
    pub dataset: Dataset,
    pub composition_rule: String,
    pub inventory_path: String,
    pub inventory_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionRecord {
    pub status: ExecutionStatus,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBuildRecord {
    pub dataset: Dataset,
    pub archive_format: ArchiveFormat,
    pub output_path: String,
    pub archive_file: String,
    pub archive_size: u64,
    pub archive_blake3: String,
    pub file_count: usize,
    pub dataset_name: String,
    pub format: String,
    pub build: CommandExecutionRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2FoundationReport {
    pub datasets: Vec<DatasetProvenance>,
    pub archive_builds: Vec<ArchiveBuildRecord>,
    pub deterministic_generation_confirmed: bool,
}

pub fn create_dataset_fixture(root: &Path, dataset: Dataset) -> Result<DatasetInventory> {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(root)?;

    match dataset {
        Dataset::Smallfiles => build_smallfiles(root)?,
        Dataset::Mixed => build_mixed(root)?,
        Dataset::Largefiles => build_largefiles(root)?,
    }

    deterministic_inventory(root, dataset)
}

pub fn write_inventory_and_provenance(
    workspace_root: &Path,
    dataset: Dataset,
    inventory: &DatasetInventory,
) -> Result<DatasetProvenance> {
    let dataset_dir = workspace_root
        .join("PHASE2_RESEARCH")
        .join("datasets")
        .join(dataset.slug());
    fs::create_dir_all(&dataset_dir)?;
    let inventory_path = dataset_dir.join("inventory.json");
    fs::write(&inventory_path, serde_json::to_vec_pretty(inventory)?)?;

    Ok(DatasetProvenance {
        generator: "crushr-lab phase2 fixture builder v1".to_string(),
        deterministic: true,
        dataset,
        composition_rule: dataset.composition_rule().to_string(),
        inventory_path: rel_path(workspace_root, &inventory_path),
        inventory_blake3: inventory.inventory_blake3.clone(),
    })
}

pub fn build_archives_for_dataset(
    workspace_root: &Path,
    artifact_root: &Path,
    dataset: Dataset,
    dataset_dir: &Path,
) -> Result<Vec<ArchiveBuildRecord>> {
    let baselines_root = workspace_root.join("PHASE2_RESEARCH/baselines");
    let observations_dir = artifact_root.join("observations");
    fs::create_dir_all(&baselines_root)?;
    fs::create_dir_all(&observations_dir)?;

    let files = collect_relative_files(dataset_dir)?;
    normalize_fixture_timestamps(dataset_dir, &files)?;
    let mut records = Vec::new();

    for archive_format in ArchiveFormat::ordered_locked_core() {
        let format_dir = baselines_root.join(archive_format.slug());
        fs::create_dir_all(&format_dir)?;
        let output_path = format_dir.join(archive_format.output_file_name(dataset));
        let build = match archive_format {
            ArchiveFormat::Crushr => run_crushr_pack(
                workspace_root,
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveFormat::Zip => run_zip_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveFormat::TarZstd => run_tar_zstd_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveFormat::TarGz => run_tar_gz_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveFormat::TarXz => run_tar_xz_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
        };

        records.push(ArchiveBuildRecord {
            dataset,
            archive_format: *archive_format,
            output_path: rel_path(workspace_root, &output_path),
            archive_file: output_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| archive_format.output_file_name(dataset)),
            archive_size: if matches!(build.status, ExecutionStatus::Success)
                && output_path.exists()
            {
                fs::metadata(&output_path)?.len()
            } else {
                0
            },
            archive_blake3: if matches!(build.status, ExecutionStatus::Success)
                && output_path.exists()
            {
                blake3::hash(&fs::read(&output_path)?).to_hex().to_string()
            } else {
                String::new()
            },
            file_count: files.len(),
            dataset_name: dataset.slug().to_string(),
            format: archive_format.slug().to_string(),
            build,
        });
    }

    Ok(records)
}

fn normalize_fixture_timestamps(dataset_dir: &Path, files: &[String]) -> Result<()> {
    if detect_tool(["touch"]).is_none() {
        return Ok(());
    }

    for file in files {
        let status = Command::new("touch")
            .arg("-t")
            .arg("198001010000.00")
            .arg(dataset_dir.join(file))
            .status()
            .with_context(|| format!("normalizing timestamp for {file}"))?;
        if !status.success() {
            bail!("failed to normalize timestamp for {file}");
        }
    }

    Ok(())
}

fn build_smallfiles(root: &Path) -> Result<()> {
    for dir in ["docs", "src", "logs", "cfg", "tmp", "notes"] {
        fs::create_dir_all(root.join(dir))?;
    }

    for i in 0..24_u8 {
        let folder = match i % 6 {
            0 => "docs",
            1 => "src",
            2 => "logs",
            3 => "cfg",
            4 => "tmp",
            _ => "notes",
        };
        let name = format!("file_{:02}.txt", i);
        let path = root.join(folder).join(name);
        let mut content = String::new();
        for line in 0..(usize::from(i) + 3) {
            content.push_str(&format!(
                "smallfiles deterministic payload file={} line={} seed=phase2\n",
                i, line
            ));
        }
        fs::write(path, content)?;
    }
    Ok(())
}

fn build_mixed(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("text"))?;
    fs::create_dir_all(root.join("bin"))?;
    fs::create_dir_all(root.join("meta"))?;

    for i in 0..12_u8 {
        let path = root.join("text").join(format!("doc_{:02}.txt", i));
        let content = format!(
            "mixed text payload idx={}\n{}\n",
            i,
            "lorem ipsum".repeat(usize::from(i) + 1)
        );
        fs::write(path, content)?;
    }

    for i in 0..4_u8 {
        let mut bytes = Vec::with_capacity(2048 + usize::from(i) * 257);
        for n in 0..bytes.capacity() {
            bytes.push(((n + usize::from(i) * 17) % 251) as u8);
        }
        fs::write(root.join("bin").join(format!("blob_{:02}.bin", i)), bytes)?;
    }

    fs::write(
        root.join("meta/config.json"),
        "{\"name\":\"mixed\",\"version\":1,\"stable\":true}\n",
    )?;
    fs::write(
        root.join("meta/summary.json"),
        "{\"rows\":2,\"format\":\"deterministic\"}\n",
    )?;
    fs::write(root.join("meta/table_a.csv"), "id,value\n1,alpha\n2,beta\n")?;
    fs::write(root.join("meta/table_b.csv"), "key,count\nx,10\ny,20\n")?;
    Ok(())
}

fn build_largefiles(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("payload"))?;

    let mut text = String::new();
    for i in 0..12_000_u32 {
        text.push_str(&format!(
            "largefiles text line={} token={}\n",
            i,
            i.wrapping_mul(31)
        ));
    }
    fs::write(root.join("payload/large_text.txt"), text)?;

    let mut bin_a = vec![0_u8; 196_608];
    for (idx, byte) in bin_a.iter_mut().enumerate() {
        *byte = (idx % 251) as u8;
    }
    fs::write(root.join("payload/large_blob_a.bin"), bin_a)?;

    let mut bin_b = vec![0_u8; 262_144];
    for (idx, byte) in bin_b.iter_mut().enumerate() {
        *byte = ((idx * 7) % 253) as u8;
    }
    fs::write(root.join("payload/large_blob_b.bin"), bin_b)?;

    Ok(())
}

fn deterministic_inventory(root: &Path, dataset: Dataset) -> Result<DatasetInventory> {
    let rel_files = collect_relative_files(root)?;
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    for rel in rel_files {
        let bytes = fs::read(root.join(&rel))?;
        total_bytes += bytes.len() as u64;
        files.push(FileInventoryEntry {
            path: rel,
            bytes: bytes.len() as u64,
            blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }

    let mut digest_feed = String::new();
    for file in &files {
        digest_feed.push_str(&format!("{}:{}:{}\n", file.path, file.bytes, file.blake3));
    }

    Ok(DatasetInventory {
        dataset,
        composition_rule: dataset.composition_rule().to_string(),
        file_count: files.len(),
        total_bytes,
        inventory_blake3: blake3::hash(digest_feed.as_bytes()).to_hex().to_string(),
        files,
    })
}

fn run_crushr_pack(
    workspace_root: &Path,
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: Dataset,
) -> Result<CommandExecutionRecord> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root)
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("crushr")
        .arg("--bin")
        .arg("crushr-pack")
        .arg("--");
    for file in files {
        cmd.arg(dataset_dir.join(file));
    }
    cmd.arg("-o").arg(output_path);

    execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_build", dataset.slug(), ArchiveFormat::Crushr.slug()),
        "cargo",
        cmd,
    )
}

fn run_zip_build(
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: Dataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["zip"]).is_none() {
        return Ok(skipped_record("zip", "zip executable not found in PATH"));
    }

    let mut cmd = Command::new("zip");
    cmd.current_dir(dataset_dir)
        .arg("-X")
        .arg("-q")
        .arg(output_path);
    for file in files {
        cmd.arg(file);
    }
    execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_build", dataset.slug(), ArchiveFormat::Zip.slug()),
        "zip",
        cmd,
    )
}

fn run_tar_zstd_build(
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: Dataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["tar"]).is_none() {
        return Ok(skipped_record("tar", "tar executable not found in PATH"));
    }
    if detect_tool(["zstd"]).is_none() {
        return Ok(skipped_record("zstd", "zstd executable not found in PATH"));
    }

    let tar_path = output_path.with_extension("tar");
    let mut tar_cmd = Command::new("tar");
    tar_cmd
        .current_dir(dataset_dir)
        .arg("--sort=name")
        .arg("--mtime=@0")
        .arg("--owner=0")
        .arg("--group=0")
        .arg("--numeric-owner")
        .arg("-cf")
        .arg(&tar_path);
    for file in files {
        tar_cmd.arg(file);
    }
    let tar_record = execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_tar", dataset.slug(), ArchiveFormat::TarZstd.slug()),
        "tar",
        tar_cmd,
    )?;
    if !matches!(tar_record.status, ExecutionStatus::Success) {
        return Ok(tar_record);
    }

    let mut zstd_cmd = Command::new("zstd");
    zstd_cmd
        .arg("-q")
        .arg("-f")
        .arg(&tar_path)
        .arg("-o")
        .arg(output_path);
    let zstd_record = execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_zstd", dataset.slug(), ArchiveFormat::TarZstd.slug()),
        "zstd",
        zstd_cmd,
    )?;

    if tar_path.exists() {
        fs::remove_file(tar_path)?;
    }

    Ok(zstd_record)
}

fn run_tar_gz_build(
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: Dataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["tar"]).is_none() {
        return Ok(skipped_record("tar", "tar executable not found in PATH"));
    }

    let mut cmd = Command::new("tar");
    cmd.current_dir(dataset_dir)
        .env("GZIP", "-n")
        .arg("--sort=name")
        .arg("--mtime=@0")
        .arg("--owner=0")
        .arg("--group=0")
        .arg("--numeric-owner")
        .arg("-czf")
        .arg(output_path);
    for file in files {
        cmd.arg(file);
    }
    execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_build", dataset.slug(), ArchiveFormat::TarGz.slug()),
        "tar",
        cmd,
    )
}

fn run_tar_xz_build(
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: Dataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["tar"]).is_none() {
        return Ok(skipped_record("tar", "tar executable not found in PATH"));
    }

    let mut cmd = Command::new("tar");
    cmd.current_dir(dataset_dir)
        .arg("--sort=name")
        .arg("--mtime=@0")
        .arg("--owner=0")
        .arg("--group=0")
        .arg("--numeric-owner")
        .arg("-cJf")
        .arg(output_path);
    for file in files {
        cmd.arg(file);
    }
    execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_build", dataset.slug(), ArchiveFormat::TarXz.slug()),
        "tar",
        cmd,
    )
}

fn execute_command(
    artifact_root: &Path,
    observations_dir: &Path,
    label: String,
    program: &str,
    mut cmd: Command,
) -> Result<CommandExecutionRecord> {
    let out = cmd.output().with_context(|| format!("running {label}"))?;
    let stdout_rel = format!("observations/{label}.stdout.txt");
    let stderr_rel = format!("observations/{label}.stderr.txt");
    fs::write(artifact_root.join(&stdout_rel), &out.stdout)?;
    fs::write(artifact_root.join(&stderr_rel), &out.stderr)?;

    let args = cmd
        .get_args()
        .map(os_to_string)
        .collect::<Result<Vec<_>>>()?;

    let status = if out.status.success() {
        ExecutionStatus::Success
    } else {
        ExecutionStatus::Failure
    };

    let note = if out.status.success() {
        None
    } else {
        Some("command returned non-zero status".to_string())
    };

    fs::write(
        observations_dir.join(format!("{label}.command.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "program": program,
            "args": args,
            "exit_code": out.status.code()
        }))?,
    )?;

    Ok(CommandExecutionRecord {
        status,
        program: program.to_string(),
        args,
        exit_code: out.status.code(),
        stdout_path: Some(stdout_rel),
        stderr_path: Some(stderr_rel),
        note,
    })
}

fn skipped_record(program: &str, reason: &str) -> CommandExecutionRecord {
    CommandExecutionRecord {
        status: ExecutionStatus::Skipped,
        program: program.to_string(),
        args: Vec::new(),
        exit_code: None,
        stdout_path: None,
        stderr_path: None,
        note: Some(reason.to_string()),
    }
}

fn detect_tool<const N: usize>(names: [&str; N]) -> Option<String> {
    for name in names {
        let mut cmd = Command::new(name);
        cmd.arg("--help");
        if cmd.output().is_ok() {
            return Some(name.to_string());
        }
    }
    None
}

fn collect_relative_files(root: &Path) -> Result<Vec<String>> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                stack.push(path);
            } else if entry.file_type()?.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .context("failed to strip fixture root")?;
                files.push(path_to_slash(rel));
            }
        }
    }
    files.sort();
    Ok(files)
}

fn path_to_slash(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn os_to_string(value: &OsStr) -> Result<String> {
    value
        .to_str()
        .map(ToOwned::to_owned)
        .context("command argument contains non-utf8")
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

pub fn build_phase2_foundation(
    workspace_root: &Path,
    artifact_root: &Path,
) -> Result<Phase2FoundationReport> {
    fs::create_dir_all(artifact_root)?;
    fs::create_dir_all(workspace_root.join("PHASE2_RESEARCH"))?;
    let datasets_root = workspace_root.join("PHASE2_RESEARCH/datasets");
    let mut dataset_records = Vec::new();
    let mut archive_records = Vec::new();

    for dataset in Dataset::ordered_locked_core() {
        let dataset_dir = datasets_root.join(dataset.slug()).join("payload");
        let inventory = create_dataset_fixture(&dataset_dir, *dataset)?;
        let provenance = write_inventory_and_provenance(workspace_root, *dataset, &inventory)?;
        let inventory_hash_path = workspace_root
            .join("PHASE2_RESEARCH")
            .join("datasets")
            .join(dataset.slug())
            .join("inventory.blake3.txt");
        fs::write(
            &inventory_hash_path,
            format!("{}\n", inventory.inventory_blake3),
        )?;
        dataset_records.push(provenance);

        archive_records.extend(build_archives_for_dataset(
            workspace_root,
            artifact_root,
            *dataset,
            &dataset_dir,
        )?);
    }

    let deterministic_generation_confirmed = archive_records.iter().all(|record| {
        matches!(record.build.status, ExecutionStatus::Success)
            && !record.archive_blake3.is_empty()
            && record.archive_size > 0
    });

    Ok(Phase2FoundationReport {
        datasets: dataset_records,
        archive_builds: archive_records,
        deterministic_generation_confirmed,
    })
}

pub fn validate_archive_coverage(report: &Phase2FoundationReport) -> Result<()> {
    let expected =
        Dataset::ordered_locked_core().len() * ArchiveFormat::ordered_locked_core().len();
    if report.archive_builds.len() != expected {
        bail!(
            "archive build coverage mismatch: expected {expected}, got {}",
            report.archive_builds.len()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("crushr_lab_{label}_{ts}"))
    }

    #[test]
    fn dataset_generation_is_reproducible() {
        let root_a = temp_path("dataset_a");
        let root_b = temp_path("dataset_b");
        let inventory_a = create_dataset_fixture(&root_a, Dataset::Mixed).expect("build a");
        let inventory_b = create_dataset_fixture(&root_b, Dataset::Mixed).expect("build b");

        assert_eq!(inventory_a.inventory_blake3, inventory_b.inventory_blake3);
        assert_eq!(inventory_a.files, inventory_b.files);

        fs::remove_dir_all(root_a).ok();
        fs::remove_dir_all(root_b).ok();
    }

    #[test]
    fn inventory_digest_is_deterministic() {
        let root = temp_path("inventory");
        let inventory_1 = create_dataset_fixture(&root, Dataset::Smallfiles).expect("build");
        let inventory_2 = deterministic_inventory(&root, Dataset::Smallfiles).expect("inventory");

        assert_eq!(inventory_1.inventory_blake3, inventory_2.inventory_blake3);
        assert_eq!(inventory_1.files, inventory_2.files);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn archive_coverage_matches_locked_matrix() {
        let report = Phase2FoundationReport {
            datasets: Vec::new(),
            archive_builds: Dataset::ordered_locked_core()
                .iter()
                .flat_map(|dataset| {
                    ArchiveFormat::ordered_locked_core()
                        .iter()
                        .map(|kind| ArchiveBuildRecord {
                            dataset: *dataset,
                            archive_format: *kind,
                            output_path: format!(
                                "PHASE2_RESEARCH/baselines/{}/{}_{}.out",
                                kind.slug(),
                                dataset.slug(),
                                kind.slug()
                            ),
                            archive_file: format!("{}_{}.out", dataset.slug(), kind.slug()),
                            archive_size: 1,
                            archive_blake3: "abc".to_string(),
                            file_count: 1,
                            dataset_name: dataset.slug().to_string(),
                            format: kind.slug().to_string(),
                            build: skipped_record("test", "not executed"),
                        })
                })
                .collect(),
            deterministic_generation_confirmed: true,
        };

        validate_archive_coverage(&report).expect("coverage ok");
    }
}
