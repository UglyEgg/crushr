use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureDataset {
    Smallfiles,
    Mixed,
    Largefiles,
}

impl FixtureDataset {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[Self::Smallfiles, Self::Mixed, Self::Largefiles]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Smallfiles => "smallfiles",
            Self::Mixed => "mixed",
            Self::Largefiles => "largefiles",
        }
    }

    pub fn composition_rule(self) -> &'static str {
        match self {
            Self::Smallfiles => {
                "24 UTF-8 text files split across 6 folders; file i has exactly i+3 lines with deterministic sentence payload"
            }
            Self::Mixed => {
                "12 text files + 4 deterministic binary blobs + 2 JSON files + 2 CSV files in stable nested layout"
            }
            Self::Largefiles => {
                "3 larger files (2 binary, 1 text) with deterministic byte/line generation and stable sizes"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArchiveKind {
    #[serde(rename = "crushr")]
    Crushr,
    #[serde(rename = "tar+zstd")]
    TarZstd,
    #[serde(rename = "zip")]
    Zip,
    #[serde(rename = "7z/lzma")]
    SevenZLzma,
}

impl ArchiveKind {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[Self::Crushr, Self::TarZstd, Self::Zip, Self::SevenZLzma]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Crushr => "crushr",
            Self::TarZstd => "tar_zstd",
            Self::Zip => "zip",
            Self::SevenZLzma => "7z_lzma",
        }
    }

    fn output_file_name(self, dataset: FixtureDataset) -> String {
        match self {
            Self::Crushr => format!("{}.crs", dataset.slug()),
            Self::TarZstd => format!("{}.tar.zst", dataset.slug()),
            Self::Zip => format!("{}.zip", dataset.slug()),
            Self::SevenZLzma => format!("{}.7z", dataset.slug()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileInventoryEntry {
    pub path: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetInventory {
    pub dataset: FixtureDataset,
    pub composition_rule: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub inventory_blake3: String,
    pub files: Vec<FileInventoryEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetProvenance {
    pub generator: String,
    pub deterministic: bool,
    pub dataset: FixtureDataset,
    pub composition_rule: String,
    pub inventory_path: String,
    pub inventory_blake3: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandExecutionRecord {
    pub status: ExecutionStatus,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchiveBuildRecord {
    pub dataset: FixtureDataset,
    pub archive_kind: ArchiveKind,
    pub output_path: String,
    pub build: CommandExecutionRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phase2FoundationReport {
    pub datasets: Vec<DatasetProvenance>,
    pub archive_builds: Vec<ArchiveBuildRecord>,
}

pub fn create_dataset_fixture(root: &Path, dataset: FixtureDataset) -> Result<DatasetInventory> {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(root)?;

    match dataset {
        FixtureDataset::Smallfiles => build_smallfiles(root)?,
        FixtureDataset::Mixed => build_mixed(root)?,
        FixtureDataset::Largefiles => build_largefiles(root)?,
    }

    deterministic_inventory(root, dataset)
}

pub fn write_inventory_and_provenance(
    artifact_root: &Path,
    dataset: FixtureDataset,
    inventory: &DatasetInventory,
) -> Result<DatasetProvenance> {
    let dataset_dir = artifact_root.join("datasets").join(dataset.slug());
    fs::create_dir_all(&dataset_dir)?;
    let inventory_path = dataset_dir.join("inventory.json");
    fs::write(&inventory_path, serde_json::to_vec_pretty(inventory)?)?;

    Ok(DatasetProvenance {
        generator: "crushr-lab phase2 fixture builder v1".to_string(),
        deterministic: true,
        dataset,
        composition_rule: dataset.composition_rule().to_string(),
        inventory_path: rel_path(artifact_root, &inventory_path),
        inventory_blake3: inventory.inventory_blake3.clone(),
    })
}

pub fn build_archives_for_dataset(
    workspace_root: &Path,
    artifact_root: &Path,
    dataset: FixtureDataset,
    dataset_dir: &Path,
) -> Result<Vec<ArchiveBuildRecord>> {
    let archives_dir = artifact_root.join("archives");
    let observations_dir = artifact_root.join("observations");
    fs::create_dir_all(&archives_dir)?;
    fs::create_dir_all(&observations_dir)?;

    let files = collect_relative_files(dataset_dir)?;
    let mut records = Vec::new();

    for archive_kind in ArchiveKind::ordered_locked_core() {
        let output_path = archives_dir.join(archive_kind.output_file_name(dataset));
        let build = match archive_kind {
            ArchiveKind::Crushr => run_crushr_pack(
                workspace_root,
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveKind::Zip => run_zip_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveKind::TarZstd => run_tar_zstd_build(
                dataset_dir,
                &files,
                &output_path,
                artifact_root,
                &observations_dir,
                dataset,
            )?,
            ArchiveKind::SevenZLzma => run_7z_build(
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
            archive_kind: *archive_kind,
            output_path: rel_path(artifact_root, &output_path),
            build,
        });
    }

    Ok(records)
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

fn deterministic_inventory(root: &Path, dataset: FixtureDataset) -> Result<DatasetInventory> {
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
    dataset: FixtureDataset,
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
        format!("{}_{}_build", dataset.slug(), ArchiveKind::Crushr.slug()),
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
    dataset: FixtureDataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["zip"]).is_none() {
        return Ok(skipped_record("zip", "zip executable not found in PATH"));
    }

    let mut cmd = Command::new("zip");
    cmd.current_dir(dataset_dir).arg("-q").arg(output_path);
    for file in files {
        cmd.arg(file);
    }
    execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_build", dataset.slug(), ArchiveKind::Zip.slug()),
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
    dataset: FixtureDataset,
) -> Result<CommandExecutionRecord> {
    if detect_tool(["tar"]).is_none() {
        return Ok(skipped_record("tar", "tar executable not found in PATH"));
    }
    if detect_tool(["zstd"]).is_none() {
        return Ok(skipped_record("zstd", "zstd executable not found in PATH"));
    }

    let tar_path = output_path.with_extension("tar");
    let mut tar_cmd = Command::new("tar");
    tar_cmd.current_dir(dataset_dir).arg("-cf").arg(&tar_path);
    for file in files {
        tar_cmd.arg(file);
    }
    let tar_record = execute_command(
        artifact_root,
        observations_dir,
        format!("{}_{}_tar", dataset.slug(), ArchiveKind::TarZstd.slug()),
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
        format!("{}_{}_zstd", dataset.slug(), ArchiveKind::TarZstd.slug()),
        "zstd",
        zstd_cmd,
    )?;

    if tar_path.exists() {
        fs::remove_file(tar_path)?;
    }

    Ok(zstd_record)
}

fn run_7z_build(
    dataset_dir: &Path,
    files: &[String],
    output_path: &Path,
    artifact_root: &Path,
    observations_dir: &Path,
    dataset: FixtureDataset,
) -> Result<CommandExecutionRecord> {
    let program = detect_tool(["7z", "7za"]);
    let Some(tool) = program else {
        return Ok(skipped_record("7z", "7z/7za executable not found in PATH"));
    };

    let mut cmd = Command::new(&tool);
    cmd.current_dir(dataset_dir)
        .arg("a")
        .arg("-t7z")
        .arg("-mx=7")
        .arg(output_path);
    for file in files {
        cmd.arg(file);
    }
    execute_command(
        artifact_root,
        observations_dir,
        format!(
            "{}_{}_build",
            dataset.slug(),
            ArchiveKind::SevenZLzma.slug()
        ),
        &tool,
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
    let datasets_root = artifact_root.join("datasets");
    let mut dataset_records = Vec::new();
    let mut archive_records = Vec::new();

    for dataset in FixtureDataset::ordered_locked_core() {
        let dataset_dir = datasets_root.join(dataset.slug()).join("payload");
        let inventory = create_dataset_fixture(&dataset_dir, *dataset)?;
        let provenance = write_inventory_and_provenance(artifact_root, *dataset, &inventory)?;
        let inventory_hash_path = artifact_root
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

    Ok(Phase2FoundationReport {
        datasets: dataset_records,
        archive_builds: archive_records,
    })
}

pub fn validate_archive_coverage(report: &Phase2FoundationReport) -> Result<()> {
    let expected =
        FixtureDataset::ordered_locked_core().len() * ArchiveKind::ordered_locked_core().len();
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
        let inventory_a = create_dataset_fixture(&root_a, FixtureDataset::Mixed).expect("build a");
        let inventory_b = create_dataset_fixture(&root_b, FixtureDataset::Mixed).expect("build b");

        assert_eq!(inventory_a.inventory_blake3, inventory_b.inventory_blake3);
        assert_eq!(inventory_a.files, inventory_b.files);

        fs::remove_dir_all(root_a).ok();
        fs::remove_dir_all(root_b).ok();
    }

    #[test]
    fn inventory_digest_is_deterministic() {
        let root = temp_path("inventory");
        let inventory_1 = create_dataset_fixture(&root, FixtureDataset::Smallfiles).expect("build");
        let inventory_2 =
            deterministic_inventory(&root, FixtureDataset::Smallfiles).expect("inventory");

        assert_eq!(inventory_1.inventory_blake3, inventory_2.inventory_blake3);
        assert_eq!(inventory_1.files, inventory_2.files);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn archive_coverage_matches_locked_matrix() {
        let report = Phase2FoundationReport {
            datasets: Vec::new(),
            archive_builds: FixtureDataset::ordered_locked_core()
                .iter()
                .flat_map(|dataset| {
                    ArchiveKind::ordered_locked_core()
                        .iter()
                        .map(|kind| ArchiveBuildRecord {
                            dataset: *dataset,
                            archive_kind: *kind,
                            output_path: format!("archives/{}_{}.out", dataset.slug(), kind.slug()),
                            build: skipped_record("test", "not executed"),
                        })
                })
                .collect(),
        };

        validate_archive_coverage(&report).expect("coverage ok");
    }
}
