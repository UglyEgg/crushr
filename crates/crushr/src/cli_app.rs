// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Result, bail};
use clap::{Arg, Command, value_parser};
use clap_complete::{
    Generator,
    shells::{Bash, Fish, Zsh},
};
use clap_mangen::Man;
use crushr::cli_presentation::CliPresenter;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCommand {
    Pack,
    Extract,
    Verify,
    Info,
    About,
    Completion,
    Man,
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
            "completion" => Some(Self::Completion),
            "man" => Some(Self::Man),
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
        AppCommand::Completion => run_completion(rest)?,
        AppCommand::Man => run_man(rest)?,
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
        ("completion", "generate shell completion script"),
        ("man", "generate clap-derived man pages"),
    ] {
        presenter.kv(command, description);
    }

    presenter.section("Bounded non-primary commands");
    presenter.kv("lab", "research harness");
}

fn run_completion(args: Vec<String>) -> Result<i32> {
    let matches = completion_command()
        .try_get_matches_from(std::iter::once("crushr completion".to_string()).chain(args))?;
    let shell = matches
        .get_one::<String>("shell")
        .expect("shell arg is required");
    match shell.as_str() {
        "bash" => generate_completion(Bash),
        "zsh" => generate_completion(Zsh),
        "fish" => generate_completion(Fish),
        _ => unreachable!("clap enforces supported shells"),
    }
    Ok(0)
}

fn generate_completion<G: Generator>(generator: G) {
    let mut cmd = cli_spec_command();
    clap_complete::generate(generator, &mut cmd, "crushr", &mut io::stdout());
}

fn completion_command() -> Command {
    Command::new("crushr completion")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("shell")
                .value_parser(["bash", "zsh", "fish"])
                .required(true)
                .help("target shell"),
        )
}

fn run_man(args: Vec<String>) -> Result<i32> {
    let matches = man_command()
        .try_get_matches_from(std::iter::once("crushr man".to_string()).chain(args))?;
    let out_dir = matches
        .get_one::<PathBuf>("out-dir")
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&out_dir)?;

    for (page_name, command) in man_page_commands() {
        write_man_page(&out_dir, &page_name, command)?;
    }

    Ok(0)
}

fn man_command() -> Command {
    Command::new("crushr man")
        .disable_help_subcommand(true)
        .arg(
            Arg::new("out-dir")
                .long("out-dir")
                .value_name("path")
                .value_parser(value_parser!(PathBuf))
                .help("output directory for generated man pages"),
        )
}

fn man_page_commands() -> Vec<(String, Command)> {
    let spec = cli_spec_command();
    let mut pages = Vec::with_capacity(7);
    pages.push(("crushr".to_string(), spec.clone()));

    for subcommand_name in ["info", "extract", "verify", "pack", "about", "completion"] {
        let sub = spec
            .find_subcommand(subcommand_name)
            .expect("subcommand exists in canonical CLI definition");
        let page_name = format!("crushr-{subcommand_name}");
        pages.push((
            page_name.clone(),
            sub.clone().bin_name(format!("crushr {subcommand_name}")),
        ));
    }

    pages
}

fn write_man_page(out_dir: &Path, page_name: &str, command: Command) -> Result<()> {
    let mut rendered = Vec::new();
    Man::new(command).render(&mut rendered)?;
    fs::write(out_dir.join(format!("{page_name}.1")), rendered)?;
    Ok(())
}

fn cli_spec_command() -> Command {
    Command::new("crushr")
        .subcommand(Command::new("pack"))
        .subcommand(
            Command::new("extract")
                .arg(Arg::new("archive"))
                .arg(Arg::new("paths").num_args(0..))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("out-dir"),
                )
                .arg(Arg::new("all").long("all"))
                .arg(Arg::new("overwrite").long("overwrite"))
                .arg(Arg::new("recover").long("recover"))
                .arg(
                    Arg::new("refusal-exit")
                        .long("refusal-exit")
                        .value_parser(["success", "partial-failure"]),
                )
                .arg(Arg::new("json").long("json"))
                .arg(Arg::new("silent").long("silent")),
        )
        .subcommand(
            Command::new("verify")
                .arg(Arg::new("archive"))
                .arg(Arg::new("json").long("json"))
                .arg(Arg::new("silent").long("silent")),
        )
        .subcommand(
            Command::new("info")
                .arg(Arg::new("archive"))
                .arg(Arg::new("list").long("list"))
                .arg(Arg::new("flat").long("flat"))
                .arg(Arg::new("entry").long("entry"))
                .arg(Arg::new("find").long("find"))
                .arg(
                    Arg::new("find-mode")
                        .long("find-mode")
                        .value_parser(["substring"]),
                )
                .arg(
                    Arg::new("find-limit")
                        .long("find-limit")
                        .value_parser(value_parser!(u32)),
                )
                .arg(Arg::new("propagation").long("propagation"))
                .arg(Arg::new("json").long("json")),
        )
        .subcommand(Command::new("about"))
        .subcommand(
            Command::new("completion").arg(
                Arg::new("shell")
                    .value_parser(["bash", "zsh", "fish"])
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("man").arg(
                Arg::new("out-dir")
                    .long("out-dir")
                    .value_name("path")
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .subcommand(Command::new("lab"))
}
