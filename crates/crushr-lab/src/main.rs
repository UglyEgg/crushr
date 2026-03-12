use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod phase2_corruption;
mod phase2_foundation;
mod phase2_manifest;
mod phase2_runner;

use phase2_corruption::{apply_locked_corruption, CorruptionRequest};
use phase2_foundation::{build_phase2_foundation, validate_archive_coverage};
use phase2_manifest::{
    validate_manifest_shape, CorruptionType, Magnitude, Phase2ExperimentManifest, TargetClass,
    PHASE2_MANIFEST_SCHEMA_ID, PHASE2_MANIFEST_SCHEMA_PATH,
};
use phase2_runner::run_phase2_execution;

const FIRST_EXPERIMENT_ID: &str = "crushr_p0s12f0_first_e2e_byteflip";
const FIRST_EXPERIMENT_REL_DIR: &str = "docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip";
const FIRST_EXPERIMENT_FIXTURE: &[u8] = b"crushr experiment fixture\nline-2\nline-3\n";
const COMPARISON_SCAFFOLD_ID: &str = "crushr_p0s13f0_competitor_scaffold_byteflip";
const COMPARISON_SCAFFOLD_REL_DIR: &str =
    "docs/RESEARCH/artifacts/crushr_p0s13f0_competitor_scaffold_byteflip";
const COMPARISON_SEED: u64 = 2026;

#[derive(Debug, Serialize)]
struct CorruptionLog {
    model: String,
    source_archive: String,
    scenario_id: String,
    target: String,
    magnitude: String,
    seed: u64,
    input_len: u64,
    input_blake3: String,
    output_blake3: String,
    touched_offsets: Vec<u64>,
    concrete_mutation_details: serde_json::Value,
}

#[derive(Debug)]
struct CorruptArgs {
    input: PathBuf,
    output: PathBuf,
    model: String,
    target: String,
    magnitude: String,
    scenario_id: String,
    seed: u64,
    offset: Option<u64>,
}

#[derive(Debug, Serialize)]
struct CommandResultRecord {
    command: String,
    status: String,
    exit_code: i32,
    stdout_file: String,
    stderr_file: String,
}

#[derive(Debug, Serialize)]
struct ComparisonTargetRecord {
    archive_type: String,
    supported: bool,
    deferred_reason: Option<String>,
    build_tool: Option<String>,
    build: Option<CommandResultRecord>,
    observe_clean: Option<CommandResultRecord>,
    observe_corrupt: Option<CommandResultRecord>,
    corruption: Option<CorruptionLog>,
}

#[derive(Debug, Serialize)]
struct ComparisonManifest {
    experiment_id: String,
    fixture: String,
    corruption_model: String,
    seed: u64,
    artifact_layout: Vec<String>,
    targets: Vec<ComparisonTargetRecord>,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "corrupt" => run_corrupt(args.collect()),
        "write-phase2-manifest" => write_phase2_manifest(args.collect()),
        "run-first-experiment" => run_first_experiment(args.collect()),
        "run-competitor-scaffold" => run_competitor_scaffold(args.collect()),
        "build-phase2-foundation" => run_phase2_foundation(args.collect()),
        "run-phase2-execution" => run_phase2_execution_cmd(args.collect()),
        _ => {
            eprintln!(
                "usage:\n  crushr-lab corrupt <input> <output> [--model <bit_flip|byte_overwrite|zero_fill|truncation|tail_damage> --target <header|index|payload|tail> --magnitude <1B|256B|4KB> --seed <1337|2600|65535> --scenario-id <id> [--offset <u64>]]\n  crushr-lab write-phase2-manifest [--output <path>]\n  crushr-lab run-first-experiment [--artifact-dir <path>]\n  crushr-lab run-competitor-scaffold [--artifact-dir <path>]\n  crushr-lab build-phase2-foundation [--artifact-dir <path>]\n  crushr-lab run-phase2-execution [--manifest <path> --foundation-report <path> --artifact-dir <path>]"
            );
            std::process::exit(1);
        }
    }
}

fn run_phase2_foundation(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut artifact_dir = workspace_root()?.join("docs/RESEARCH/artifacts/phase2_foundation");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let root = workspace_root()?;
    let report = build_phase2_foundation(&root, &artifact_dir)?;
    validate_archive_coverage(&report)?;
    fs::write(
        artifact_dir.join("foundation_report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn run_phase2_execution_cmd(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let root = workspace_root()?;
    let mut manifest_path = root.join("docs/RESEARCH/artifacts/phase2_core_manifest.json");
    let mut foundation_report_path =
        root.join("docs/RESEARCH/artifacts/phase2_foundation/foundation_report.json");
    let mut artifact_dir = root.join("docs/RESEARCH/artifacts/phase2_execution");

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
    let foundation: phase2_foundation::Phase2FoundationReport =
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

fn write_phase2_manifest(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut output = workspace_root()?.join("docs/RESEARCH/artifacts/phase2_core_manifest.json");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = PathBuf::from(args.next().context("missing value for --output")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut manifest = serde_json::to_value(Phase2ExperimentManifest::locked_core())?;
    manifest
        .as_object_mut()
        .context("phase2 manifest value must be object")?
        .insert(
            "$schema".to_string(),
            serde_json::Value::String(PHASE2_MANIFEST_SCHEMA_ID.to_string()),
        );

    validate_manifest_shape(&manifest)?;
    fs::write(output, serde_json::to_vec_pretty(&manifest)?)?;
    eprintln!(
        "wrote locked Phase 2 manifest using schema {}",
        PHASE2_MANIFEST_SCHEMA_PATH
    );
    Ok(())
}

fn run_competitor_scaffold(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut artifact_dir = workspace_root()?.join(COMPARISON_SCAFFOLD_REL_DIR);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let fixture_dir = artifact_dir.join("fixture");
    let archives_dir = artifact_dir.join("archives");
    let corrupt_dir = artifact_dir.join("corrupt");
    let observations_dir = artifact_dir.join("observations");
    for dir in [&fixture_dir, &archives_dir, &corrupt_dir, &observations_dir] {
        fs::create_dir_all(dir)?;
    }

    fs::write(
        fixture_dir.join("alpha.txt"),
        b"alpha fixture line\nsecond line\n",
    )?;
    fs::write(
        fixture_dir.join("beta.txt"),
        b"beta fixture payload\nfor comparison\n",
    )?;

    let targets = vec![
        run_crushr_target(
            &artifact_dir,
            &fixture_dir,
            &archives_dir,
            &corrupt_dir,
            &observations_dir,
        )?,
        run_zip_target(
            &artifact_dir,
            &fixture_dir,
            &archives_dir,
            &corrupt_dir,
            &observations_dir,
        )?,
        run_tar_zstd_target(
            &artifact_dir,
            &fixture_dir,
            &archives_dir,
            &corrupt_dir,
            &observations_dir,
        )?,
        run_7z_target(),
    ];

    let manifest = ComparisonManifest {
        experiment_id: COMPARISON_SCAFFOLD_ID.to_string(),
        fixture: "fixture/alpha.txt + fixture/beta.txt".to_string(),
        corruption_model: "byteflip".to_string(),
        seed: COMPARISON_SEED,
        artifact_layout: vec![
            "fixture/".to_string(),
            "archives/".to_string(),
            "corrupt/".to_string(),
            "observations/".to_string(),
            "comparison_manifest.json".to_string(),
        ],
        targets,
    };
    fs::write(
        artifact_dir.join("comparison_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}

fn run_crushr_target(
    artifact_dir: &Path,
    fixture_dir: &Path,
    archives_dir: &Path,
    corrupt_dir: &Path,
    observations_dir: &Path,
) -> Result<ComparisonTargetRecord> {
    let clean = archives_dir.join("fixture.crs");
    let corrupt = corrupt_dir.join("fixture.crs.corrupt");
    let build = record_command(
        artifact_dir,
        observations_dir,
        "crushr_build",
        "cargo run -q -p crushr --bin crushr-pack -- fixture/*.txt -o archives/fixture.crs",
        command_for_crushr(
            "crushr-pack",
            &[
                fixture_dir.join("alpha.txt").as_path(),
                fixture_dir.join("beta.txt").as_path(),
                Path::new("-o"),
                clean.as_path(),
            ],
        )?,
    )?;
    if build.exit_code != 0 {
        return Ok(ComparisonTargetRecord {
            archive_type: "crushr".to_string(),
            supported: true,
            deferred_reason: Some("build failed in current environment".to_string()),
            build_tool: Some("crushr-pack".to_string()),
            build: Some(build),
            observe_clean: None,
            observe_corrupt: None,
            corruption: None,
        });
    }
    let offset = fs::metadata(&clean)?.len().saturating_sub(1);
    let corruption = apply_byteflip(&clean, &corrupt, COMPARISON_SEED, offset)?;
    let observe_clean = record_command(
        artifact_dir,
        observations_dir,
        "crushr_observe_clean",
        "cargo run -q -p crushr --bin crushr-info -- archives/fixture.crs --json",
        command_for_crushr("crushr-info", &[clean.as_path(), Path::new("--json")])?,
    )?;
    let observe_corrupt = record_command(
        artifact_dir,
        observations_dir,
        "crushr_observe_corrupt",
        "cargo run -q -p crushr --bin crushr-info -- corrupt/fixture.crs.corrupt --json",
        command_for_crushr("crushr-info", &[corrupt.as_path(), Path::new("--json")])?,
    )?;
    Ok(ComparisonTargetRecord {
        archive_type: "crushr".to_string(),
        supported: true,
        deferred_reason: None,
        build_tool: Some("crushr-pack".to_string()),
        build: Some(build),
        observe_clean: Some(observe_clean),
        observe_corrupt: Some(observe_corrupt),
        corruption: Some(corruption),
    })
}

fn run_zip_target(
    artifact_dir: &Path,
    fixture_dir: &Path,
    archives_dir: &Path,
    corrupt_dir: &Path,
    observations_dir: &Path,
) -> Result<ComparisonTargetRecord> {
    let Some(tool) = detect_tool(&["zip"]) else {
        return Ok(deferred_target("zip", "zip executable not found in PATH"));
    };
    let clean = archives_dir.join("fixture.zip");
    let corrupt = corrupt_dir.join("fixture.zip.corrupt");
    let mut build_cmd = Command::new(&tool);
    build_cmd
        .current_dir(fixture_dir)
        .arg("-q")
        .arg(clean.as_path())
        .arg("alpha.txt")
        .arg("beta.txt");
    let build = record_command(
        artifact_dir,
        observations_dir,
        "zip_build",
        "zip -q archives/fixture.zip alpha.txt beta.txt",
        build_cmd,
    )?;
    if build.exit_code != 0 {
        return Ok(build_failed_target("zip", "zip", build));
    }
    let offset = fs::metadata(&clean)?.len().saturating_sub(1);
    let corruption = apply_byteflip(&clean, &corrupt, COMPARISON_SEED, offset)?;
    let mut clean_cmd = Command::new(&tool);
    clean_cmd.arg("-T").arg(clean.as_path());
    let observe_clean = record_command(
        artifact_dir,
        observations_dir,
        "zip_observe_clean",
        "zip -T archives/fixture.zip",
        clean_cmd,
    )?;
    let mut corrupt_cmd = Command::new(&tool);
    corrupt_cmd.arg("-T").arg(corrupt.as_path());
    let observe_corrupt = record_command(
        artifact_dir,
        observations_dir,
        "zip_observe_corrupt",
        "zip -T corrupt/fixture.zip.corrupt",
        corrupt_cmd,
    )?;
    Ok(ComparisonTargetRecord {
        archive_type: "zip".to_string(),
        supported: true,
        deferred_reason: None,
        build_tool: Some("zip".to_string()),
        build: Some(build),
        observe_clean: Some(observe_clean),
        observe_corrupt: Some(observe_corrupt),
        corruption: Some(corruption),
    })
}

fn run_tar_zstd_target(
    artifact_dir: &Path,
    fixture_dir: &Path,
    archives_dir: &Path,
    corrupt_dir: &Path,
    observations_dir: &Path,
) -> Result<ComparisonTargetRecord> {
    let Some(_tar) = detect_tool(&["tar"]) else {
        return Ok(deferred_target(
            "tar+zstd",
            "tar executable not found in PATH",
        ));
    };
    let Some(_zstd) = detect_tool(&["zstd"]) else {
        return Ok(deferred_target(
            "tar+zstd",
            "zstd executable not found in PATH",
        ));
    };
    let clean = archives_dir.join("fixture.tar.zst");
    let corrupt = corrupt_dir.join("fixture.tar.zst.corrupt");
    let mut build_cmd = Command::new("sh");
    build_cmd.current_dir(fixture_dir).arg("-c").arg(format!(
        "tar -cf - alpha.txt beta.txt | zstd -q -o {}",
        clean.display()
    ));
    let build = record_command(
        artifact_dir,
        observations_dir,
        "tar_zstd_build",
        "tar -cf - alpha.txt beta.txt | zstd -q -o archives/fixture.tar.zst",
        build_cmd,
    )?;
    if build.exit_code != 0 {
        return Ok(build_failed_target("tar+zstd", "tar+zstd", build));
    }
    let offset = fs::metadata(&clean)?.len().saturating_sub(1);
    let corruption = apply_byteflip(&clean, &corrupt, COMPARISON_SEED, offset)?;
    let mut clean_cmd = Command::new("sh");
    clean_cmd
        .arg("-c")
        .arg(format!("zstd -dc {} | tar -tf -", clean.display()));
    let observe_clean = record_command(
        artifact_dir,
        observations_dir,
        "tar_zstd_observe_clean",
        "zstd -dc archives/fixture.tar.zst | tar -tf -",
        clean_cmd,
    )?;
    let mut corrupt_cmd = Command::new("sh");
    corrupt_cmd
        .arg("-c")
        .arg(format!("zstd -dc {} | tar -tf -", corrupt.display()));
    let observe_corrupt = record_command(
        artifact_dir,
        observations_dir,
        "tar_zstd_observe_corrupt",
        "zstd -dc corrupt/fixture.tar.zst.corrupt | tar -tf -",
        corrupt_cmd,
    )?;
    Ok(ComparisonTargetRecord {
        archive_type: "tar+zstd".to_string(),
        supported: true,
        deferred_reason: None,
        build_tool: Some("tar+zstd".to_string()),
        build: Some(build),
        observe_clean: Some(observe_clean),
        observe_corrupt: Some(observe_corrupt),
        corruption: Some(corruption),
    })
}

fn run_7z_target() -> ComparisonTargetRecord {
    if detect_tool(&["7z", "7za"]).is_some() {
        return deferred_target(
            "7z",
            "tool detected but 7z comparison execution is intentionally deferred in this scaffold",
        );
    }
    deferred_target("7z", "7z/7za executable not found in PATH")
}

fn deferred_target(archive_type: &str, reason: &str) -> ComparisonTargetRecord {
    ComparisonTargetRecord {
        archive_type: archive_type.to_string(),
        supported: false,
        deferred_reason: Some(reason.to_string()),
        build_tool: None,
        build: None,
        observe_clean: None,
        observe_corrupt: None,
        corruption: None,
    }
}

fn build_failed_target(
    archive_type: &str,
    tool: &str,
    build: CommandResultRecord,
) -> ComparisonTargetRecord {
    ComparisonTargetRecord {
        archive_type: archive_type.to_string(),
        supported: true,
        deferred_reason: Some("build command returned nonzero status".to_string()),
        build_tool: Some(tool.to_string()),
        build: Some(build),
        observe_clean: None,
        observe_corrupt: None,
        corruption: None,
    }
}

fn detect_tool(names: &[&str]) -> Option<String> {
    for name in names {
        let mut cmd = Command::new(name);
        cmd.arg("--help");
        if cmd.output().is_ok() {
            return Some((*name).to_string());
        }
    }
    None
}

fn record_command(
    artifact_dir: &Path,
    observations_dir: &Path,
    name: &str,
    command: &str,
    mut cmd: Command,
) -> Result<CommandResultRecord> {
    let out = cmd.output()?;
    let stdout_file = format!("observations/{name}.stdout.txt");
    let stderr_file = format!("observations/{name}.stderr.txt");
    fs::write(artifact_dir.join(&stdout_file), &out.stdout)?;
    fs::write(artifact_dir.join(&stderr_file), &out.stderr)?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(observations_dir.join("commands.log"))?;
    writeln!(f, "{name}: {command}")?;

    Ok(CommandResultRecord {
        command: command.to_string(),
        status: if out.status.success() {
            "success".to_string()
        } else {
            "failure".to_string()
        },
        exit_code: out.status.code().unwrap_or(-1),
        stdout_file,
        stderr_file,
    })
}

fn apply_byteflip(input: &Path, output: &Path, seed: u64, offset: u64) -> Result<CorruptionLog> {
    let mut bytes = fs::read(input)?;
    let chosen_offset = pick_offset(bytes.len(), seed, Some(offset))?;
    if let Some(ix) = chosen_offset {
        bytes[ix] ^= 0x01;
    }
    fs::write(output, &bytes)?;
    Ok(CorruptionLog {
        model: "byteflip".to_string(),
        source_archive: input.display().to_string(),
        scenario_id: "legacy-byteflip-explicit-offset".to_string(),
        target: "tail".to_string(),
        magnitude: "1B".to_string(),
        seed,
        input_len: bytes.len() as u64,
        input_blake3: blake3::hash(&fs::read(input)?).to_hex().to_string(),
        output_blake3: blake3::hash(&bytes).to_hex().to_string(),
        touched_offsets: chosen_offset.map(|x| vec![x as u64]).unwrap_or_default(),
        concrete_mutation_details: serde_json::json!([
            {
                "operation": "bit_flip",
                "offset": chosen_offset.map(|x| x as u64)
            }
        ]),
    })
}

fn run_corrupt(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let input = PathBuf::from(args.next().context("missing input")?);
    let output = PathBuf::from(args.next().context("missing output")?);

    let mut model = String::from("bit_flip");
    let mut target = String::from("payload");
    let mut magnitude = String::from("1B");
    let mut scenario_id = String::from("ad-hoc-scenario");
    let mut seed = 1337_u64;
    let mut offset = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let value = args.next().context("missing value for --seed")?;
                seed = value
                    .parse::<u64>()
                    .with_context(|| format!("invalid --seed value: {value}"))?;
            }
            "--model" => {
                model = args.next().context("missing value for --model")?;
            }
            "--target" => {
                target = args.next().context("missing value for --target")?;
            }
            "--magnitude" => {
                magnitude = args.next().context("missing value for --magnitude")?;
            }
            "--scenario-id" => {
                scenario_id = args.next().context("missing value for --scenario-id")?;
            }
            "--offset" => {
                let value = args.next().context("missing value for --offset")?;
                offset = Some(
                    value
                        .parse::<u64>()
                        .with_context(|| format!("invalid --offset value: {value}"))?,
                );
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let parsed = CorruptArgs {
        input,
        output,
        model,
        target,
        magnitude,
        scenario_id,
        seed,
        offset,
    };

    let corruption_type = parse_corruption_type(&parsed.model)?;
    let target = parse_target(&parsed.target)?;
    let magnitude = parse_magnitude(&parsed.magnitude)?;

    let input_bytes =
        fs::read(&parsed.input).with_context(|| format!("reading {}", parsed.input.display()))?;
    let (bytes, provenance) = apply_locked_corruption(
        &input_bytes,
        &CorruptionRequest {
            source_archive: parsed.input.display().to_string(),
            scenario_id: parsed.scenario_id.clone(),
            corruption_type,
            target,
            magnitude,
            seed: parsed.seed,
            forced_offset: parsed.offset,
        },
    )?;

    fs::write(&parsed.output, &bytes)
        .with_context(|| format!("writing {}", parsed.output.display()))?;

    let touched_offsets = provenance
        .concrete_mutation_details
        .iter()
        .filter_map(|d| d.offset)
        .collect();
    let log = CorruptionLog {
        model: parsed.model,
        source_archive: provenance.source_archive,
        scenario_id: provenance.scenario_id,
        target: parsed.target,
        magnitude: parsed.magnitude,
        seed: parsed.seed,
        input_len: input_bytes.len() as u64,
        input_blake3: blake3::hash(&fs::read(&parsed.input)?).to_hex().to_string(),
        output_blake3: blake3::hash(&bytes).to_hex().to_string(),
        touched_offsets,
        concrete_mutation_details: serde_json::to_value(provenance.concrete_mutation_details)?,
    };
    let log_path = parsed.output.with_extension("corrupt.json");
    fs::write(log_path, serde_json::to_vec_pretty(&log)?)?;
    Ok(())
}

fn parse_corruption_type(raw: &str) -> Result<CorruptionType> {
    match raw {
        "bit_flip" | "byteflip" => Ok(CorruptionType::BitFlip),
        "byte_overwrite" => Ok(CorruptionType::ByteOverwrite),
        "zero_fill" => Ok(CorruptionType::ZeroFill),
        "truncation" => Ok(CorruptionType::Truncation),
        "tail_damage" => Ok(CorruptionType::TailDamage),
        _ => bail!("unsupported model: {raw}"),
    }
}

fn parse_target(raw: &str) -> Result<TargetClass> {
    match raw {
        "header" => Ok(TargetClass::Header),
        "index" => Ok(TargetClass::Index),
        "payload" => Ok(TargetClass::Payload),
        "tail" => Ok(TargetClass::Tail),
        _ => bail!("unsupported --target: {raw}"),
    }
}

fn parse_magnitude(raw: &str) -> Result<Magnitude> {
    match raw {
        "1B" => Ok(Magnitude::OneByte),
        "256B" => Ok(Magnitude::TwoHundredFiftySixBytes),
        "4KB" => Ok(Magnitude::FourKilobytes),
        _ => bail!("unsupported --magnitude: {raw}"),
    }
}

fn run_first_experiment(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut artifact_dir = workspace_root()?.join(FIRST_EXPERIMENT_REL_DIR);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-dir" => {
                artifact_dir =
                    PathBuf::from(args.next().context("missing value for --artifact-dir")?);
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("creating artifact directory {}", artifact_dir.display()))?;

    let fixture = artifact_dir.join("fixture.txt");
    fs::write(&fixture, FIRST_EXPERIMENT_FIXTURE)?;

    let clean_archive = artifact_dir.join("clean.crs");
    let corrupt_archive = artifact_dir.join("corrupt.crs");

    run_expect_success(
        "pack",
        command_for_crushr(
            "crushr-pack",
            &[fixture.as_path(), Path::new("-o"), clean_archive.as_path()],
        )?,
    )?;

    let clean_info = run_expect_success(
        "clean_info",
        command_for_crushr(
            "crushr-info",
            &[clean_archive.as_path(), Path::new("--json")],
        )?,
    )?;
    fs::write(artifact_dir.join("clean.info.json"), &clean_info.stdout)?;

    let clean_fsck = run_expect_success(
        "clean_fsck",
        command_for_crushr(
            "crushr-fsck",
            &[clean_archive.as_path(), Path::new("--json")],
        )?,
    )?;
    fs::write(artifact_dir.join("clean.fsck.json"), &clean_fsck.stdout)?;
    let clean_fsck_json: serde_json::Value = serde_json::from_slice(&clean_fsck.stdout)?;
    if clean_fsck_json["payload"]["verify"]["status"] != "ok" {
        bail!("clean_fsck payload verify status is not ok");
    }

    let clean_len = fs::metadata(&clean_archive)?.len();
    let offset = clean_len
        .checked_sub(1)
        .context("clean archive has zero length")?;

    run_expect_success(
        "corrupt",
        command_for_lab_corrupt(&clean_archive, &corrupt_archive, 1337, offset)?,
    )?;

    let corrupt_log_path = corrupt_archive.with_extension("corrupt.json");
    let corruption_log: serde_json::Value = serde_json::from_slice(&fs::read(&corrupt_log_path)?)?;

    run_expect_failure(
        "corrupt_fsck",
        command_for_crushr(
            "crushr-fsck",
            &[corrupt_archive.as_path(), Path::new("--json")],
        )?,
        2,
        artifact_dir.join("corrupt.fsck.exit_code.txt"),
        artifact_dir.join("corrupt.fsck.stderr.txt"),
    )?;

    run_expect_failure(
        "corrupt_info",
        command_for_crushr(
            "crushr-info",
            &[corrupt_archive.as_path(), Path::new("--json")],
        )?,
        2,
        artifact_dir.join("corrupt.info.exit_code.txt"),
        artifact_dir.join("corrupt.info.stderr.txt"),
    )?;

    let manifest = serde_json::json!({
        "experiment_id": FIRST_EXPERIMENT_ID,
        "fixture": "single file fixture.txt with 3 short lines",
        "commands": {
            "pack": "cargo run -q -p crushr --bin crushr-pack -- fixture.txt -o clean.crs",
            "corrupt": "cargo run -q -p crushr-lab --bin crushr-lab -- corrupt clean.crs corrupt.crs --model byteflip --seed 1337 --offset <len-1>",
            "clean_info": "cargo run -q -p crushr --bin crushr-info -- clean.crs --json",
            "clean_fsck": "cargo run -q -p crushr --bin crushr-fsck -- clean.crs --json",
            "corrupt_fsck": "cargo run -q -p crushr --bin crushr-fsck -- corrupt.crs --json",
            "corrupt_info": "cargo run -q -p crushr --bin crushr-info -- corrupt.crs --json"
        },
        "seed": 1337,
        "model": "byteflip",
        "touched_offsets": corruption_log["touched_offsets"],
        "input_blake3": corruption_log["input_blake3"],
        "output_blake3": corruption_log["output_blake3"]
    });
    fs::write(
        artifact_dir.join("experiment_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to derive workspace root")
}

fn command_for_crushr(bin: &str, args: &[&Path]) -> Result<Command> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root()?)
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("crushr")
        .arg("--bin")
        .arg(bin)
        .arg("--");
    for arg in args {
        cmd.arg(arg);
    }
    Ok(cmd)
}

fn command_for_lab_corrupt(input: &Path, output: &Path, seed: u64, offset: u64) -> Result<Command> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root()?)
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("crushr-lab")
        .arg("--bin")
        .arg("crushr-lab")
        .arg("--")
        .arg("corrupt")
        .arg(input)
        .arg(output)
        .arg("--model")
        .arg("bit_flip")
        .arg("--target")
        .arg("tail")
        .arg("--magnitude")
        .arg("1B")
        .arg("--scenario-id")
        .arg("legacy-first-experiment-tail-byteflip")
        .arg("--seed")
        .arg(seed.to_string())
        .arg("--offset")
        .arg(offset.to_string());
    Ok(cmd)
}

fn run_expect_success(step: &str, mut cmd: Command) -> Result<Output> {
    let out = cmd
        .output()
        .with_context(|| format!("running step {step}"))?;
    if !out.status.success() {
        bail!(
            "step `{step}` failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(out)
}

fn run_expect_failure(
    step: &str,
    mut cmd: Command,
    expected_code: i32,
    exit_code_path: PathBuf,
    stderr_path: PathBuf,
) -> Result<()> {
    let out = cmd
        .output()
        .with_context(|| format!("running step {step}"))?;
    let code = out.status.code().unwrap_or(-1);
    fs::write(&exit_code_path, format!("{code}\n"))?;
    fs::write(&stderr_path, &out.stderr)?;
    if code != expected_code {
        bail!(
            "step `{step}` expected exit code {expected_code}, got {code}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn pick_offset(len: usize, seed: u64, offset: Option<u64>) -> Result<Option<usize>> {
    if len == 0 {
        return Ok(None);
    }

    if let Some(explicit) = offset {
        let idx = usize::try_from(explicit).context("--offset overflows usize")?;
        if idx >= len {
            bail!("--offset {explicit} is out of bounds for input length {len}");
        }
        return Ok(Some(idx));
    }

    Ok(Some((seed as usize) % len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_experiment_constants_match_artifact_contract() {
        assert_eq!(FIRST_EXPERIMENT_ID, "crushr_p0s12f0_first_e2e_byteflip");
        assert_eq!(
            FIRST_EXPERIMENT_REL_DIR,
            "docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip"
        );
        assert_eq!(
            FIRST_EXPERIMENT_FIXTURE,
            b"crushr experiment fixture\nline-2\nline-3\n"
        );
    }

    #[test]
    fn deferred_target_is_not_marked_success() {
        let target = deferred_target("7z", "not found");
        assert!(!target.supported);
        assert!(target.build.is_none());
        assert!(target.observe_clean.is_none());
        assert!(target.deferred_reason.is_some());
    }

    #[test]
    fn comparison_manifest_shape_is_deterministic() {
        let manifest = ComparisonManifest {
            experiment_id: COMPARISON_SCAFFOLD_ID.to_string(),
            fixture: "fixture/alpha.txt + fixture/beta.txt".to_string(),
            corruption_model: "byteflip".to_string(),
            seed: COMPARISON_SEED,
            artifact_layout: vec![
                "fixture/".to_string(),
                "archives/".to_string(),
                "corrupt/".to_string(),
                "observations/".to_string(),
                "comparison_manifest.json".to_string(),
            ],
            targets: vec![deferred_target("7z", "not found")],
        };
        let json = serde_json::to_value(&manifest).expect("serialize manifest");
        assert_eq!(json["experiment_id"], COMPARISON_SCAFFOLD_ID);
        assert_eq!(json["artifact_layout"][0], "fixture/");
        assert_eq!(json["targets"][0]["archive_type"], "7z");
        assert_eq!(json["targets"][0]["supported"], false);
    }

    #[test]
    fn build_failed_target_records_failure_metadata() {
        let record = CommandResultRecord {
            command: "zip -q".to_string(),
            status: "failure".to_string(),
            exit_code: 9,
            stdout_file: "observations/stdout.txt".to_string(),
            stderr_file: "observations/stderr.txt".to_string(),
        };
        let target = build_failed_target("zip", "zip", record);
        assert!(target.supported);
        assert_eq!(target.build_tool.as_deref(), Some("zip"));
        assert_eq!(target.build.as_ref().map(|r| r.exit_code), Some(9));
        assert!(target.observe_corrupt.is_none());
    }
}
