// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::cli_presentation::{CliPresenter, StatusWord, group_u64};
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

use crate::recover_extract_impl;
use crate::strict_extract_impl;

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

const USAGE: &str = "usage: crushr-extract <archive> -o <out-dir> [--all] [PATH ...] [--overwrite] [--recover] [--refusal-exit <success|partial-failure>] [--json] [--silent]\n       crushr-extract --verify <archive> [--json] [--silent]";

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
    all: bool,
    recover: bool,
    selected_paths: Vec<String>,
    refusal_exit: RefusalExitPolicy,
    json: bool,
    silent: bool,
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
    recover_summary: Option<recover_extract_impl::RecoverExtractRun>,
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

fn parse_cli_options(raw_args: Vec<String>) -> Result<CliOptions, ExtractionClassifiedError> {
    let mut mode = CliMode::Extract;
    let mut archive = None;
    let mut out_dir = None;
    let mut overwrite = false;
    let mut all = false;
    let mut recover = false;
    let mut selected_paths: Vec<String> = Vec::new();
    let mut refusal_exit = RefusalExitPolicy::Success;
    let mut json = false;
    let mut silent = false;

    let mut args = raw_args.into_iter();
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
        } else if arg == "--silent" {
            silent = true;
        } else if arg == "--all" {
            all = true;
        } else if arg == "--recover" {
            recover = true;
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
            selected_paths.push(arg);
        }
    }

    CliOptions {
        mode,
        archive: archive
            .context(USAGE)
            .map_err(ExtractionClassifiedError::usage)?,
        out_dir,
        overwrite,
        all,
        recover,
        selected_paths,
        refusal_exit,
        json,
        silent,
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
                if self.all && !self.selected_paths.is_empty() {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--all cannot be combined with explicit PATH arguments"
                    )));
                }
            }
            CliMode::Verify => {
                if self.recover {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--verify cannot be combined with --recover"
                    )));
                }
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
                if self.all || !self.selected_paths.is_empty() {
                    return Err(ExtractionClassifiedError::usage(anyhow::anyhow!(
                        "--verify does not accept --all or PATH arguments"
                    )));
                }
            }
        }

        Ok(self)
    }
}

fn run_extract<F>(opts: &CliOptions, mut recover_progress: F) -> Result<ClassifiedRun>
where
    F: FnMut(&'static str),
{
    if opts.recover {
        recover_progress("recovery analysis");
        let analysis = recover_extract_impl::run_recovery_analysis(&opts.archive)?;
        let _ = (
            analysis.canonical_complete,
            analysis.recoverable_named,
            analysis.recoverable_anonymous,
            analysis.unrecoverable,
        );
        let recovered = recover_extract_impl::run_recover_extract_with_progress(
            &recover_extract_impl::RecoverExtractOptions {
                archive: opts.archive.clone(),
                out_dir: opts.out_dir.clone().expect("validated output dir"),
                overwrite: opts.overwrite,
                selected_paths: if opts.all || opts.selected_paths.is_empty() {
                    None
                } else {
                    Some(opts.selected_paths.clone())
                },
            },
            &mut recover_progress,
        )?;
        return Ok(ClassifiedRun {
            outcome_kind: recovered.outcome_kind,
            report: recovered.report.clone(),
            recover_summary: Some(recovered),
        });
    }

    let strict =
        strict_extract_impl::run_strict_extract(&strict_extract_impl::StrictExtractOptions {
            archive: opts.archive.clone(),
            out_dir: opts.out_dir.clone().expect("validated output dir"),
            overwrite: opts.overwrite,
            selected_paths: if opts.all || opts.selected_paths.is_empty() {
                None
            } else {
                Some(opts.selected_paths.clone())
            },
            verify_only: false,
        })?;

    Ok(ClassifiedRun {
        outcome_kind: strict.outcome_kind,
        report: strict.report,
        recover_summary: None,
    })
}

fn run_verify(
    opts: &CliOptions,
    progress: Option<&CliPresenter>,
) -> Result<(VerificationModel, VerificationReportView)> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };
    if let Some(presenter) = progress {
        presenter.stage("archive open / header read", StatusWord::Ok);
    }

    let opened = open_archive_v1(&reader)?;
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    if let Some(presenter) = progress {
        presenter.stage("metadata/index scan", StatusWord::Scanning);
    }
    let verified_extent_count = blocks.len();
    let _ = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    if let Some(presenter) = progress {
        presenter.stage("payload verification", StatusWord::Running);
    }

    let strict =
        strict_extract_impl::run_strict_extract(&strict_extract_impl::StrictExtractOptions {
            archive: opts.archive.clone(),
            out_dir: std::env::temp_dir(),
            overwrite: false,
            selected_paths: None,
            verify_only: true,
        })?;
    if let Some(presenter) = progress {
        presenter.stage("manifest validation", StatusWord::Finalizing);
    }

    let model = VerificationModel::from_extraction_report(&strict.report, verified_extent_count);
    if let Some(presenter) = progress {
        presenter.stage("final result/report", StatusWord::Complete);
    }
    Ok((
        model.clone(),
        model.to_report_view(opts.archive.display().to_string()),
    ))
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

pub fn dispatch(args: Vec<String>) -> i32 {
    let early_args = args.clone();
    if matches!(
        early_args.first().map(String::as_str),
        Some("--help" | "-h")
    ) {
        println!("{USAGE}");
        return 0;
    }
    if matches!(
        early_args.first().map(String::as_str),
        Some("--version" | "-V")
    ) {
        println!("{}", crate::product_version());
        return 0;
    }

    let opts = match parse_cli_options(args) {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{err}");
            return exit_code_for_error(err.kind);
        }
    };

    match opts.mode {
        CliMode::Extract => {
            let presenter =
                CliPresenter::new("crushr-extract", "extract", opts.silent && !opts.json);
            if opts.recover && !opts.json && !opts.silent {
                presenter.header();
                presenter.section("Progress");
            }
            match run_extract(&opts, |stage| {
                if opts.recover && !opts.json && !opts.silent {
                    let status = match stage {
                        "archive open" => StatusWord::Ok,
                        "metadata scan" => StatusWord::Scanning,
                        "canonical extraction" => StatusWord::Running,
                        "recovery analysis" => StatusWord::Running,
                        "recovery extraction" => StatusWord::Finalizing,
                        "manifest/report finalization" => StatusWord::Complete,
                        _ => StatusWord::Running,
                    };
                    presenter.stage(stage, status);
                }
            }) {
                Ok(classified) => {
                    if opts.json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&classified.report)
                                .expect("serialize extraction report")
                        );
                    } else {
                        if !opts.recover || opts.silent {
                            presenter.header();
                        }
                        presenter.section("Archive");
                        presenter.kv("archive", opts.archive.display());
                        if let Some(out_dir) = &opts.out_dir {
                            presenter.kv("output dir", out_dir.display());
                        }
                        if opts.recover {
                            presenter.kv("mode", "recover");
                        } else {
                            presenter.kv("mode", "strict");
                        }
                        presenter.section("Result");
                        if opts.recover {
                            let recovered_run = classified
                                .recover_summary
                                .as_ref()
                                .expect("recover summary present in recover mode");
                            presenter.kv(
                                "canonical files",
                                group_u64(recovered_run.canonical_count as u64),
                            );
                            presenter.kv(
                                "recovered_named",
                                group_u64(recovered_run.recovered_named_count as u64),
                            );
                            presenter.kv(
                                "recovered_anonymous",
                                group_u64(recovered_run.recovered_anonymous_count as u64),
                            );
                            presenter.kv(
                                "unrecoverable",
                                group_u64(recovered_run.unrecoverable_count as u64),
                            );
                            presenter.section("Extraction status");
                            presenter.kv("canonical extraction", recovered_run.canonical_trust);
                            presenter.kv("recovery extraction", recovered_run.recovery_trust);
                            let non_canonical_count = recovered_run.recovered_named_count
                                + recovered_run.recovered_anonymous_count;
                            if non_canonical_count > 0 || recovered_run.unrecoverable_count > 0 {
                                presenter.section("Notes");
                                presenter.item(
                                    StatusWord::Ok,
                                    "recovered output is non-canonical and kept under recovery paths",
                                );
                                presenter.kv("manifest", recovered_run.manifest_path.display());
                            }
                        } else {
                            presenter.kv(
                                "safe files",
                                group_u64(classified.report.safe_file_count as u64),
                            );
                            presenter.kv(
                                "refused files",
                                group_u64(classified.report.refused_file_count as u64),
                            );
                        }
                        presenter.outcome(
                            if classified.outcome_kind == ExtractionOutcomeKind::Success {
                                StatusWord::Complete
                            } else {
                                StatusWord::Partial
                            },
                            if opts.recover {
                                "recovery extraction completed"
                            } else {
                                "strict extraction completed"
                            },
                        );
                        presenter.silent_summary(
                            if classified.outcome_kind == ExtractionOutcomeKind::Success {
                                StatusWord::Complete
                            } else {
                                StatusWord::Partial
                            },
                            &[
                                ("archive", opts.archive.display().to_string()),
                                ("safe_files", classified.report.safe_file_count.to_string()),
                                (
                                    "refused_files",
                                    classified.report.refused_file_count.to_string(),
                                ),
                            ],
                        );
                    }
                    exit_code_for_outcome(classified.outcome_kind, opts.refusal_exit)
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
                    code
                }
            }
        }
        CliMode::Verify => {
            let presenter =
                CliPresenter::new("crushr-extract", "verify", opts.silent && !opts.json);
            let progress = if opts.json || opts.silent {
                None
            } else {
                Some(&presenter)
            };
            if progress.is_some() {
                presenter.header();
                presenter.section("Progress");
            }
            match run_verify(&opts, progress) {
                Ok((model, report)) => {
                    if opts.json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).expect("serialize verify report")
                        );
                    } else {
                        presenter.section("Archive");
                        presenter.kv("archive", &report.archive_path);
                        if report.safe_for_strict_extraction {
                            presenter.section("Verification");
                            presenter.kv(
                                "verified extents",
                                group_u64(report.verified_extent_count as u64),
                            );
                            presenter
                                .kv("failed checks", group_u64(report.failed_check_count as u64));
                            presenter.kv(
                                "identity resolution",
                                format!("{:?}", model.failure_domains.identity_resolution)
                                    .to_lowercase(),
                            );
                            presenter.kv(
                                "dictionary resolution",
                                format!("{:?}", model.failure_domains.dictionary_resolution)
                                    .to_lowercase(),
                            );
                            presenter.section("Result");
                            presenter.outcome(StatusWord::Verified, "safe for strict extraction");
                        } else {
                            presenter.section("Failure domain");
                            presenter.kv("component", "strict verification");
                            presenter.kv("reason", "verification checks failed");
                            presenter.kv("expected", "archive passes all strict checks");
                            presenter.kv("received", "failed checks detected");
                            presenter
                                .kv("failed checks", group_u64(report.failed_check_count as u64));
                            if let Some(first_reason) = report.refusal_reasons.first() {
                                presenter.kv("first refusal", first_reason);
                            }
                            presenter.section("Result");
                            presenter
                                .outcome(StatusWord::Refused, "not safe for strict extraction");
                        }
                        presenter.silent_summary(
                            if report.safe_for_strict_extraction {
                                StatusWord::Verified
                            } else {
                                StatusWord::Refused
                            },
                            &[
                                ("archive", report.archive_path.clone()),
                                ("failed_checks", report.failed_check_count.to_string()),
                            ],
                        );
                    }
                    if report.safe_for_strict_extraction {
                        0
                    } else {
                        2
                    }
                }
                Err(err) => {
                    let classified_err = ExtractionClassifiedError::structural(err);
                    if opts.json {
                        eprintln!("{classified_err}");
                    } else {
                        if progress.is_none() {
                            presenter.header();
                        }
                        presenter.section("Archive");
                        presenter.kv("archive", opts.archive.display());
                        presenter.section("Failure domain");
                        presenter.kv("component", "archive structure");
                        presenter.kv("reason", "failed to parse strict verification inputs");
                        presenter.kv("expected", "valid FTR4 footer, tail frame, and index");
                        presenter.kv("received", "invalid or unreadable archive structure");
                        presenter.section("Result");
                        presenter.outcome(StatusWord::Refused, "not safe for strict extraction");
                        presenter.silent_summary(
                            StatusWord::Refused,
                            &[
                                ("archive", opts.archive.display().to_string()),
                                ("failed_checks", "1".to_string()),
                            ],
                        );
                    }
                    exit_code_for_error(classified_err.kind)
                }
            }
        }
    }
}

pub fn dispatch_from_env() -> i32 {
    dispatch(std::env::args().skip(1).collect())
}

#[cfg(test)]
mod tests {
    use super::{
        ExtractionErrorKind, ExtractionOutcomeKind, RefusalExitPolicy, exit_code_for_error,
        exit_code_for_outcome,
    };
    use crushr_core::extraction::{
        RefusalReason, RefusedFileReport, SafeFileReport, build_extraction_report,
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
