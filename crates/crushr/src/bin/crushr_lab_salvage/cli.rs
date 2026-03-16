use super::*;

pub(super) fn parse_cli_options() -> Result<CliOptions> {
    let mut args = std::env::args().skip(1);

    if let Some(first) = args.next() {
        if first == "--help" || first == "-h" || first == "help" {
            return Ok(CliOptions {
                mode: Mode::Help,
                export_fragments: false,
                limit: None,
                verbose: false,
            });
        }

        if first == "run-redundant-map-comparison" {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    "--help" | "-h" => {
                        return Ok(CliOptions {
                            mode: Mode::Help,
                            export_fragments: false,
                            limit: None,
                            verbose: false,
                        });
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: Mode::RunRedundantMapComparison {
                    comparison_dir: output_dir.context(USAGE)?,
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }
        if first == "run-experimental-resilience-comparison" {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    "--help" | "-h" => {
                        return Ok(CliOptions {
                            mode: Mode::Help,
                            export_fragments: false,
                            limit: None,
                            verbose: false,
                        });
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: Mode::RunExperimentalResilienceComparison {
                    comparison_dir: output_dir.context(USAGE)?,
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }

        if first == "run-file-identity-comparison"
            || first == "run-format04-comparison"
            || first == "run-format05-comparison"
            || first == "run-format06-comparison"
            || first == "run-format07-comparison"
            || first == "run-format08-placement-comparison"
            || first == "run-format09-comparison"
            || first == "run-format10-pruning-comparison"
            || first == "run-format11-extent-identity-comparison"
        {
            let mut output_dir = None;
            let mut verbose = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                    }
                    "--verbose" => {
                        verbose = true;
                    }
                    "--help" | "-h" => {
                        return Ok(CliOptions {
                            mode: Mode::Help,
                            export_fragments: false,
                            limit: None,
                            verbose: false,
                        });
                    }
                    _ => bail!("unsupported comparison argument: {arg}"),
                }
            }

            return Ok(CliOptions {
                mode: if first == "run-format04-comparison" {
                    Mode::RunFormat04Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format05-comparison" {
                    Mode::RunFormat05Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format06-comparison" {
                    Mode::RunFormat06Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format07-comparison" {
                    Mode::RunFormat07Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format08-placement-comparison" {
                    Mode::RunFormat08PlacementComparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format09-comparison" {
                    Mode::RunFormat09Comparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format10-pruning-comparison" {
                    Mode::RunFormat10PruningComparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else if first == "run-format11-extent-identity-comparison" {
                    Mode::RunFormat11ExtentIdentityComparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                } else {
                    Mode::RunFileIdentityComparison {
                        comparison_dir: output_dir.context(USAGE)?,
                    }
                },
                export_fragments: false,
                limit: None,
                verbose,
            });
        }

        let mut input_dir = None;
        let mut output_dir = None;
        let mut resummarize_dir = None;
        let mut export_fragments = false;
        let mut limit = None;
        let mut verbose = false;

        let mut pending = Some(first);
        loop {
            let arg = if let Some(value) = pending.take() {
                value
            } else if let Some(value) = args.next() {
                value
            } else {
                break;
            };

            match arg.as_str() {
                "--output" => {
                    output_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                }
                "--export-fragments" => {
                    export_fragments = true;
                }
                "--limit" => {
                    let value = args.next().context(USAGE)?;
                    limit = Some(
                        value
                            .parse::<usize>()
                            .with_context(|| format!("invalid --limit value: {value}"))?,
                    );
                }
                "--verbose" => {
                    verbose = true;
                }
                "--resummarize" => {
                    resummarize_dir = Some(PathBuf::from(args.next().context(USAGE)?));
                }
                "--help" | "-h" => {
                    return Ok(CliOptions {
                        mode: Mode::Help,
                        export_fragments: false,
                        limit: None,
                        verbose: false,
                    });
                }
                "run-redundant-map-comparison"
                | "run-experimental-resilience-comparison"
                | "run-file-identity-comparison"
                | "run-format04-comparison"
                | "run-format05-comparison"
                | "run-format06-comparison"
                | "run-format07-comparison"
                | "run-format08-placement-comparison"
                | "run-format09-comparison"
                | "run-format10-pruning-comparison"
                | "run-format11-extent-identity-comparison" => {
                    bail!("subcommand `{arg}` must be used as the first argument\n{USAGE}")
                }
                _ if arg.starts_with('-') => bail!("unsupported flag: {arg}"),
                _ if input_dir.is_none() => input_dir = Some(PathBuf::from(arg)),
                _ => bail!("unexpected argument: {arg}"),
            }
        }

        let mode = if let Some(experiment_dir) = resummarize_dir {
            if input_dir.is_some() || limit.is_some() || export_fragments || output_dir.is_some() {
                bail!("--resummarize cannot be combined with run flags");
            }
            Mode::Resummarize { experiment_dir }
        } else {
            Mode::RunExperiment {
                input_dir: input_dir.context(USAGE)?,
                experiment_dir: output_dir.context(USAGE)?,
            }
        };

        Ok(CliOptions {
            mode,
            export_fragments,
            limit,
            verbose,
        })
    } else {
        bail!(USAGE)
    }
}
