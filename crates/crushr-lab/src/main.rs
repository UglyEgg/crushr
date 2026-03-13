use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "corrupt" => run_corrupt(args.collect()),
        "write-phase2-manifest" => write_phase2_manifest(args.collect()),
        "build-phase2-foundation" => run_phase2_foundation(args.collect()),
        "run-phase2-execution" => run_phase2_execution_cmd(args.collect()),
        _ => {
            eprintln!(
                "usage:\n  crushr-lab corrupt <input> <output> [--model <bit_flip|byte_overwrite|zero_fill|truncation|tail_damage> --target <header|index|payload|tail> --magnitude <1B|256B|4KB> --seed <1337|2600|65535> --scenario-id <id> [--offset <u64>]]\n  crushr-lab write-phase2-manifest [--output <path>]\n  crushr-lab build-phase2-foundation [--artifact-dir <path>]\n  crushr-lab run-phase2-execution [--manifest <path> --foundation-report <path> --artifact-dir <path>]"
            );
            std::process::exit(1);
        }
    }
}

fn run_phase2_foundation(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let mut artifact_dir = workspace_root()?.join("PHASE2_RESEARCH/generated/foundation");
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
    let mut manifest_path = root.join("PHASE2_RESEARCH/manifests/phase2_core_manifest.json");
    let mut foundation_report_path =
        root.join("PHASE2_RESEARCH/generated/foundation/foundation_report.json");
    let mut artifact_dir = root.join("PHASE2_RESEARCH/generated/execution");

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
    let mut output = workspace_root()?.join("PHASE2_RESEARCH/manifests/phase2_core_manifest.json");
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

fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to derive workspace root")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_corruption_type_aliases_byteflip() {
        assert!(matches!(
            parse_corruption_type("byteflip").expect("alias should parse"),
            CorruptionType::BitFlip
        ));
    }

    #[test]
    fn parse_target_rejects_unknown_values() {
        let err = parse_target("unknown").expect_err("target should fail");
        assert!(err.to_string().contains("unsupported --target"));
    }
}
