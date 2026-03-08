use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

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
    if cmd != "corrupt" {
        eprintln!("usage: crushr-lab corrupt <input> <output>");
        std::process::exit(1);
    }
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
