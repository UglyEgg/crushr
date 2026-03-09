use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIRST_EXPERIMENT_ID: &str = "crushr_p0s12f0_first_e2e_byteflip";
const FIRST_EXPERIMENT_REL_DIR: &str = "docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip";
const FIRST_EXPERIMENT_FIXTURE: &[u8] = b"crushr experiment fixture\nline-2\nline-3\n";

#[derive(Debug, Serialize)]
struct CorruptionLog {
    model: String,
    seed: u64,
    input_len: u64,
    input_blake3: String,
    output_blake3: String,
    touched_offsets: Vec<u64>,
}

#[derive(Debug)]
struct CorruptArgs {
    input: PathBuf,
    output: PathBuf,
    model: String,
    seed: u64,
    offset: Option<u64>,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "corrupt" => run_corrupt(args.collect()),
        "run-first-experiment" => run_first_experiment(args.collect()),
        _ => {
            eprintln!(
                "usage:\n  crushr-lab corrupt <input> <output> [--model byteflip --seed <u64> --offset <u64>]\n  crushr-lab run-first-experiment [--artifact-dir <path>]"
            );
            std::process::exit(1);
        }
    }
}

fn run_corrupt(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let input = PathBuf::from(args.next().context("missing input")?);
    let output = PathBuf::from(args.next().context("missing output")?);

    let mut model = String::from("byteflip");
    let mut seed = 0_u64;
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
        seed,
        offset,
    };

    if parsed.model != "byteflip" {
        bail!("unsupported model: {}", parsed.model);
    }

    let mut bytes =
        fs::read(&parsed.input).with_context(|| format!("reading {}", parsed.input.display()))?;
    let chosen_offset = pick_offset(bytes.len(), parsed.seed, parsed.offset)?;
    if let Some(ix) = chosen_offset {
        bytes[ix] ^= 0x01;
    }
    fs::write(&parsed.output, &bytes)
        .with_context(|| format!("writing {}", parsed.output.display()))?;
    let log = CorruptionLog {
        model: parsed.model,
        seed: parsed.seed,
        input_len: bytes.len() as u64,
        input_blake3: blake3::hash(&fs::read(&parsed.input)?).to_hex().to_string(),
        output_blake3: blake3::hash(&bytes).to_hex().to_string(),
        touched_offsets: chosen_offset.map(|x| vec![x as u64]).unwrap_or_default(),
    };
    let log_path = parsed.output.with_extension("corrupt.json");
    fs::write(log_path, serde_json::to_vec_pretty(&log)?)?;
    Ok(())
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
        .arg("byteflip")
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
}
