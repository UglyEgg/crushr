// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const HELP: &str = r#"crushr — integrity-first preservation archive suite

Usage:
  crushr <command> [args...]
  crushr --help

Canonical product commands:
  pack      create an archive (dispatches to crushr-pack)
  extract   strict extraction (dispatches to crushr-extract)
  verify    strict verification alias (dispatches to crushr-extract --verify)
  info      inspect archive metadata/reporting (dispatches to crushr-info)

Bounded non-primary commands:
  salvage   experimental salvage planner (dispatches to crushr-salvage)
  lab       research harness (dispatches to crushr-lab)

Examples:
  crushr pack ./input -o archive.crushr
  crushr verify archive.crushr
  crushr extract archive.crushr -o ./out
  crushr info archive.crushr --json

Legacy command note:
  legacy generic-compressor commands are no longer part of the primary surface.
  Use the focused tools above for current product behavior.
"#;

const LEGACY_DEMOTION_MESSAGE: &str = "legacy command is no longer part of the primary crushr surface; use pack/extract/verify/info (or salvage/lab for bounded research tools)";

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        print!("{HELP}");
        std::process::exit(1);
    };

    if cmd == "--help" || cmd == "-h" {
        print!("{HELP}");
        return Ok(());
    }
    if cmd == "--version" || cmd == "-V" {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let rest: Vec<String> = args.collect();
    match cmd.as_str() {
        "pack" => forward_to("crushr-pack", &rest),
        "extract" => forward_to("crushr-extract", &rest),
        "info" => forward_to("crushr-info", &rest),
        "salvage" => forward_to("crushr-salvage", &rest),
        "lab" => forward_to("crushr-lab", &rest),
        "verify" => {
            if rest.is_empty() {
                bail!("usage: crushr verify <archive> [--json] [--silent]");
            }
            let mut mapped = Vec::with_capacity(rest.len() + 1);
            mapped.push("--verify".to_string());
            mapped.extend(rest);
            forward_to("crushr-extract", &mapped)
        }
        "append" | "list" | "cat" | "dict-train" | "tune" | "completions" => {
            bail!("{LEGACY_DEMOTION_MESSAGE}")
        }
        _ => {
            eprintln!("unknown command: {cmd}\n");
            print!("{HELP}");
            std::process::exit(1);
        }
    }
}

fn forward_to(binary: &str, args: &[String]) -> Result<()> {
    let target = resolve_sibling_binary(binary)?;
    let status = Command::new(&target)
        .args(args)
        .status()
        .with_context(|| format!("spawn {}", target.display()))?;
    std::process::exit(status.code().unwrap_or(1));
}

fn resolve_sibling_binary(binary: &str) -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    let dir = exe
        .parent()
        .context("resolve executable directory")
        .map(Path::to_path_buf)?;

    let mut filename = binary.to_string();
    if cfg!(windows) {
        filename.push_str(".exe");
    }

    let candidate = dir.join(filename);
    if candidate.exists() {
        return Ok(candidate);
    }

    bail!(
        "required companion binary not found: {}",
        candidate.display()
    )
}
