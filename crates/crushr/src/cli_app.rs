// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Result, bail};

const HELP: &str = r#"crushr — integrity-first preservation archive suite

Usage:
  crushr <command> [args...]
  crushr --help

Canonical product commands:
  pack      create an archive
  extract   strict extraction
  verify    strict verification alias (extract --verify)
  info      inspect archive metadata/reporting
  about     product identity and build metadata

Bounded non-primary commands:
  salvage   experimental salvage planner
  lab       research harness
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCommand {
    Pack,
    Extract,
    Verify,
    Info,
    About,
    Salvage,
    Lab,
}

impl AppCommand {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "pack" => Some(Self::Pack),
            "extract" => Some(Self::Extract),
            "verify" => Some(Self::Verify),
            "info" => Some(Self::Info),
            "about" => Some(Self::About),
            "salvage" => Some(Self::Salvage),
            "lab" => Some(Self::Lab),
            _ => None,
        }
    }
}

pub fn run_env() -> i32 {
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            2
        }
    }
}

fn run(args: Vec<String>) -> Result<i32> {
    let Some(first) = args.first() else {
        print!("{HELP}");
        return Ok(1);
    };

    if first == "--help" || first == "-h" {
        print!("{HELP}");
        return Ok(0);
    }
    if first == "--version" || first == "-V" {
        println!("{}", crushr::product_version());
        return Ok(0);
    }

    let cmd =
        AppCommand::parse(first).ok_or_else(|| anyhow::anyhow!("unknown command: {first}"))?;
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();

    let code = match cmd {
        AppCommand::Pack => crushr::commands::pack::dispatch(rest),
        AppCommand::Extract => crushr::commands::extract::dispatch(rest),
        AppCommand::Verify => {
            if rest.is_empty() {
                bail!("usage: crushr verify <archive> [--json] [--silent]");
            }
            let mut mapped = Vec::with_capacity(rest.len() + 1);
            mapped.push("--verify".to_string());
            mapped.extend(rest);
            crushr::commands::extract::dispatch(mapped)
        }
        AppCommand::Info => crushr::commands::info::dispatch(rest),
        AppCommand::About => {
            if !rest.is_empty() {
                bail!("usage: crushr about");
            }
            print!(
                "{}",
                crushr::about::render_about(&crushr::about::BuildMetadata::from_env())
            );
            0
        }
        AppCommand::Salvage => crushr::commands::salvage::dispatch(rest),
        AppCommand::Lab => crushr_lab::dispatch(rest)?,
    };

    Ok(code)
}
