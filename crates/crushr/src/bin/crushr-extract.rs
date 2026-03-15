use anyhow::{Context, Result};
use crushr_core::extraction::ExtractionOutcomeKind;
use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

#[path = "../strict_extract_impl.rs"]
mod strict_extract_impl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefusalExitPolicy {
    Success,
    PartialFailure,
}

impl RefusalExitPolicy {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "success" => Some(Self::Success),
            "partial-failure" => Some(Self::PartialFailure),
            _ => None,
        }
    }
}

const USAGE: &str =
    "usage: crushr-extract <archive> -o <out-dir> [--overwrite] [--refusal-exit <success|partial-failure>] [--json]";

#[derive(Debug)]
struct CliOptions {
    archive: PathBuf,
    out_dir: PathBuf,
    overwrite: bool,
    refusal_exit: RefusalExitPolicy,
    json: bool,
}

#[derive(Debug, Serialize)]
struct ExtractionErrorReport {
    overall_status: &'static str,
    error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractionErrorKind {
    Usage,
    Structural,
}

#[derive(Debug)]
struct ExtractionClassifiedError {
    kind: ExtractionErrorKind,
    error: anyhow::Error,
}

impl fmt::Display for ExtractionClassifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.error)
    }
}

impl std::error::Error for ExtractionClassifiedError {}

impl ExtractionClassifiedError {
    fn usage(error: anyhow::Error) -> Self {
        Self {
            kind: ExtractionErrorKind::Usage,
            error,
        }
    }

    fn structural(error: anyhow::Error) -> Self {
        Self {
            kind: ExtractionErrorKind::Structural,
            error,
        }
    }

    fn message(&self) -> String {
        format!("{:#}", self.error)
    }
}

#[derive(Debug)]
struct ClassifiedRun {
    outcome_kind: ExtractionOutcomeKind,
    report: crushr_core::extraction::ExtractionReport,
}

fn parse_cli_options() -> Result<CliOptions, ExtractionClassifiedError> {
    let mut archive = None;
    let mut out_dir = None;
    let mut overwrite = false;
    let mut refusal_exit = RefusalExitPolicy::Success;
    let mut json = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-o" || arg == "--output" {
            let value = args
                .next()
                .context(USAGE)
                .map_err(ExtractionClassifiedError::usage)?;
            out_dir = Some(PathBuf::from(value));
        } else if arg == "--overwrite" {
            overwrite = true;
        } else if arg == "--json" {
            json = true;
        } else if arg == "--refusal-exit" {
            let value = args
                .next()
                .context(USAGE)
                .map_err(ExtractionClassifiedError::usage)?;
            refusal_exit = RefusalExitPolicy::parse(&value).with_context(|| {
                format!(
                    "unsupported value for --refusal-exit: {value} (expected success|partial-failure)"
                )
            })
            .map_err(ExtractionClassifiedError::usage)?;
        } else if arg.starts_with('-') {
            return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                "unsupported flag: {arg}"
            )));
        } else if archive.is_none() {
            archive = Some(PathBuf::from(arg));
        } else {
            return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                "unexpected argument: {arg}"
            )));
        }
    }

    Ok(CliOptions {
        archive: archive
            .context(USAGE)
            .map_err(ExtractionClassifiedError::usage)?,
        out_dir: out_dir
            .context(USAGE)
            .map_err(ExtractionClassifiedError::usage)?,
        overwrite,
        refusal_exit,
        json,
    })
}

fn run(opts: &CliOptions) -> Result<ClassifiedRun> {
    let strict =
        strict_extract_impl::run_strict_extract(&strict_extract_impl::StrictExtractOptions {
            archive: opts.archive.clone(),
            out_dir: opts.out_dir.clone(),
            overwrite: opts.overwrite,
            selected_paths: None,
        })?;

    Ok(ClassifiedRun {
        outcome_kind: strict.outcome_kind,
        report: strict.report,
    })
}

fn exit_code_for_outcome(kind: ExtractionOutcomeKind, refusal_exit: RefusalExitPolicy) -> i32 {
    match (kind, refusal_exit) {
        (ExtractionOutcomeKind::PartialRefusal, RefusalExitPolicy::PartialFailure) => 3,
        _ => 0,
    }
}

fn exit_code_for_error(kind: ExtractionErrorKind) -> i32 {
    match kind {
        ExtractionErrorKind::Usage => 1,
        ExtractionErrorKind::Structural => 2,
    }
}

fn main() {
    let opts = match parse_cli_options() {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(exit_code_for_error(err.kind));
        }
    };

    match run(&opts) {
        Ok(classified) => {
            if opts.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&classified.report)
                        .expect("serialize extraction report")
                );
            }
            let code = exit_code_for_outcome(classified.outcome_kind, opts.refusal_exit);
            std::process::exit(code);
        }
        Err(err) => {
            let classified_err = ExtractionClassifiedError::structural(err);
            eprintln!("{classified_err}");
            let msg = classified_err.message();
            let code = exit_code_for_error(classified_err.kind);

            if opts.json {
                let json_err = ExtractionErrorReport {
                    overall_status: "error",
                    error: msg,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json_err)
                        .expect("serialize extraction error report")
                );
            }
            std::process::exit(code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        exit_code_for_error, exit_code_for_outcome, ExtractionErrorKind, ExtractionOutcomeKind,
        RefusalExitPolicy,
    };
    use crushr_core::extraction::{
        build_extraction_report, RefusalReason, RefusedFileReport, SafeFileReport,
    };

    #[test]
    fn typed_outcome_exit_mapping_is_stable() {
        assert_eq!(
            exit_code_for_outcome(ExtractionOutcomeKind::Success, RefusalExitPolicy::Success),
            0
        );
        assert_eq!(
            exit_code_for_outcome(
                ExtractionOutcomeKind::Success,
                RefusalExitPolicy::PartialFailure,
            ),
            0
        );
        assert_eq!(
            exit_code_for_outcome(
                ExtractionOutcomeKind::PartialRefusal,
                RefusalExitPolicy::Success,
            ),
            0
        );
        assert_eq!(
            exit_code_for_outcome(
                ExtractionOutcomeKind::PartialRefusal,
                RefusalExitPolicy::PartialFailure,
            ),
            3
        );
        assert_eq!(exit_code_for_error(ExtractionErrorKind::Usage), 1);
        assert_eq!(exit_code_for_error(ExtractionErrorKind::Structural), 2);
    }

    #[test]
    fn extraction_json_contract_is_assembled_by_core_helper() {
        let (kind, report) = build_extraction_report(
            vec![SafeFileReport {
                path: "b.txt".into(),
            }],
            vec![RefusedFileReport {
                path: "a.txt".into(),
                reason: RefusalReason::CorruptedRequiredBlocks,
            }],
        );

        assert_eq!(kind, ExtractionOutcomeKind::PartialRefusal);
        assert_eq!(report.overall_status, "partial_refusal");
        assert_eq!(report.safe_file_count, 1);
        assert_eq!(report.refused_file_count, 1);
    }
}
