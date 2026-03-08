use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct CorruptionLog {
    model: String,
    seed: u64,
    input_blake3: String,
    output_blake3: String,
    touched_offsets: Vec<u64>,
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
    let mut bytes = fs::read(&input).with_context(|| format!("reading {}", input.display()))?;
    if !bytes.is_empty() {
        bytes[0] ^= 0x01;
    }
    fs::write(&output, &bytes).with_context(|| format!("writing {}", output.display()))?;
    let log = CorruptionLog {
        model: "byteflip".into(),
        seed: 0,
        input_blake3: blake3::hash(&fs::read(&input)?).to_hex().to_string(),
        output_blake3: blake3::hash(&bytes).to_hex().to_string(),
        touched_offsets: if bytes.is_empty() { vec![] } else { vec![0] },
    };
    let log_path = output.with_extension("corrupt.json");
    fs::write(log_path, serde_json::to_vec_pretty(&log)?)?;
    Ok(())
}
