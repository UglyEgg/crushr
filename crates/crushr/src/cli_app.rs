// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Result, bail};
use crushr::cli_presentation::CliPresenter;

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
        print_help();
        return Ok(1);
    };

    if first == "--help" || first == "-h" {
        print_help();
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
        AppCommand::Lab => crushr::commands::lab::dispatch(rest)?,
    };

    Ok(code)
}

fn print_help() {
    let presenter = CliPresenter::new("crushr", "help", false);
    presenter.header();
    presenter.section("Usage");
    presenter.kv("command", "crushr <command> [args...]");
    presenter.kv("help", "crushr --help");

    presenter.section("Canonical product commands");
    for (command, description) in [
        ("pack", "create an archive"),
        ("extract", "strict extraction"),
        ("verify", "strict verification alias (extract --verify)"),
        ("info", "inspect archive metadata/reporting"),
        ("about", "product identity and build metadata"),
    ] {
        presenter.kv(command, description);
    }

    presenter.section("Bounded non-primary commands");
    for (command, description) in [
        ("salvage", "experimental salvage planner"),
        ("lab", "research harness"),
    ] {
        presenter.kv(command, description);
    }
}
