// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Context, Result};
use crushr_core::extraction::ExtractionOutcomeKind;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verification_model::{VerificationModel, VerificationReportView},
    verify::{scan_blocks_v1, verify_block_payloads_v1},
};
use serde::Serialize;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

#[path = "../extraction_path.rs"]
mod extraction_path;
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

const USAGE: &str = "usage: crushr-extract <archive> -o <out-dir> [--overwrite] [--refusal-exit <success|partial-failure>] [--json]\n       crushr-extract --verify <archive> [--json]";

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliMode {
    Extract,
    Verify,
}

#[derive(Debug)]
struct CliOptions {
    mode: CliMode,
    archive: PathBuf,
    out_dir: Option<PathBuf>,
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

#[derive(Debug)]
struct FileReader {
    file: File,
}

impl ReadAt for FileReader {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        use std::os::unix::fs::FileExt;
        Ok(self.file.read_at(buf, offset)?)
    }
}

impl Len for FileReader {
    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

fn parse_cli_options() -> Result<CliOptions, ExtractionClassifiedError> {
    let mut mode = CliMode::Extract;
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
        } else if arg == "--verify" {
            mode = CliMode::Verify;
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

    CliOptions {
        mode,
        archive: archive
            .context(USAGE)
            .map_err(ExtractionClassifiedError::usage)?,
        out_dir,
        overwrite,
        refusal_exit,
        json,
    }
    .validate()
}

impl CliOptions {
    fn validate(self) -> Result<Self, ExtractionClassifiedError> {
        match self.mode {
            CliMode::Extract => {
                if self.out_dir.is_none() {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(USAGE)));
                }
            }
            CliMode::Verify => {
                if self.out_dir.is_some() {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--verify cannot be combined with -o/--output"
                    )));
                }
                if self.overwrite {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--verify cannot be combined with --overwrite"
                    )));
                }
                if self.refusal_exit != RefusalExitPolicy::Success {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--verify cannot be combined with --refusal-exit"
                    )));
                }
            }
        }

        Ok(self)
    }
}

fn run_extract(opts: &CliOptions) -> Result<ClassifiedRun> {
    let strict =
        strict_extract_impl::run_strict_extract(&strict_extract_impl::StrictExtractOptions {
            archive: opts.archive.clone(),
            out_dir: opts.out_dir.clone().expect("validated output dir"),
            overwrite: opts.overwrite,
            selected_paths: None,
        })?;

    Ok(ClassifiedRun {
        outcome_kind: strict.outcome_kind,
        report: strict.report,
    })
}

fn run_verify(opts: &CliOptions) -> Result<VerificationReportView> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };
    let opened = open_archive_v1(&reader)?;
    let verified_extent_count =
        scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?.len();
    let _ = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;

    let verify_dir = create_verify_tempdir()?;
    let strict =
        strict_extract_impl::run_strict_extract(&strict_extract_impl::StrictExtractOptions {
            archive: opts.archive.clone(),
            out_dir: verify_dir.clone(),
            overwrite: false,
            selected_paths: None,
        })?;
    let _ = std::fs::remove_dir_all(&verify_dir);

    let model = VerificationModel::from_extraction_report(&strict.report, verified_extent_count);
    Ok(model.to_report_view(opts.archive.display().to_string()))
}

fn create_verify_tempdir() -> Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "crushr-extract-verify-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("read current time")?
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
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

    match opts.mode {
        CliMode::Extract => match run_extract(&opts) {
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
        },
        CliMode::Verify => match run_verify(&opts) {
            Ok(report) => {
                if opts.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).expect("serialize verify report")
                    );
                } else if report.safe_for_strict_extraction {
                    println!(
                        "verified: archive is safe for strict extraction ({})",
                        report.archive_path
                    );
                } else {
                    println!(
                        "refused: archive is not safe for strict extraction ({})",
                        report.archive_path
                    );
                    for reason in report.refusal_reasons {
                        println!("- {reason}");
                    }
                }
                std::process::exit(if report.safe_for_strict_extraction {
                    0
                } else {
                    2
                });
            }
            Err(err) => {
                let classified_err = ExtractionClassifiedError::structural(err);
                eprintln!("{classified_err}");
                std::process::exit(exit_code_for_error(classified_err.kind));
            }
        },
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
    use crushr_core::verification_model::VerificationModel;

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

    #[test]
    fn verify_output_is_derived_from_canonical_model() {
        let (_kind, extraction) = build_extraction_report(
            vec![SafeFileReport {
                path: "b.txt".into(),
            }],
            vec![RefusedFileReport {
                path: "a.txt".into(),
                reason: RefusalReason::CorruptedRequiredBlocks,
            }],
        );
        let model = VerificationModel::from_extraction_report(&extraction, 3);
        let report = model.to_report_view("archive.crs".to_string());
        assert_eq!(report.archive_path, "archive.crs");
        assert_eq!(report.verification_status, "refused");
        assert_eq!(
            report.refusal_reasons,
            vec!["corrupted_required_blocks: a.txt".to_string()]
        );
        assert_eq!(report.verified_extent_count, 3);
    }
}
