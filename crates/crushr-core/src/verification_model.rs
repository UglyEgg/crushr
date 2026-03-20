// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction::{ExtractionReport, RefusalReason};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationVerdict {
    Verified,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityResolutionStatus {
    Verified,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryResolutionStatus {
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifySummary {
    pub verdict: VerificationVerdict,
    pub safe_for_strict_extraction: bool,
    pub verified_extent_count: usize,
    pub failed_check_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileResolution {
    pub path: String,
    pub verdict: VerificationVerdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason: Option<RefusalReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureDomains {
    pub identity_resolution: IdentityResolutionStatus,
    pub dictionary_resolution: DictionaryResolutionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationModel {
    pub summary: VerifySummary,
    pub files: Vec<FileResolution>,
    pub failure_domains: FailureDomains,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReportView {
    pub archive_path: String,
    pub verification_status: &'static str,
    pub safe_for_strict_extraction: bool,
    pub refusal_reasons: Vec<String>,
    pub verified_extent_count: usize,
    pub failed_check_count: usize,
}

impl VerificationModel {
    pub fn from_extraction_report(
        report: &ExtractionReport,
        verified_extent_count: usize,
    ) -> VerificationModel {
        let mut files = Vec::with_capacity(report.safe_files.len() + report.refused_files.len());
        files.extend(report.safe_files.iter().map(|entry| FileResolution {
            path: entry.path.clone(),
            verdict: VerificationVerdict::Verified,
            refusal_reason: None,
        }));
        files.extend(report.refused_files.iter().map(|entry| FileResolution {
            path: entry.path.clone(),
            verdict: VerificationVerdict::Refused,
            refusal_reason: Some(entry.reason),
        }));
        files.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.verdict.cmp(&b.verdict)));

        let safe = report.refused_files.is_empty();
        VerificationModel {
            summary: VerifySummary {
                verdict: if safe {
                    VerificationVerdict::Verified
                } else {
                    VerificationVerdict::Refused
                },
                safe_for_strict_extraction: safe,
                verified_extent_count,
                failed_check_count: report.refused_file_count,
            },
            files,
            failure_domains: FailureDomains {
                identity_resolution: if safe {
                    IdentityResolutionStatus::Verified
                } else {
                    IdentityResolutionStatus::Partial
                },
                dictionary_resolution: DictionaryResolutionStatus::Verified,
            },
        }
    }

    pub fn to_report_view(&self, archive_path: String) -> VerificationReportView {
        let refusal_reasons = self
            .files
            .iter()
            .filter_map(|file| {
                let reason = file.refusal_reason?;
                Some(format!("{}: {}", refusal_reason_slug(reason), file.path))
            })
            .collect::<Vec<_>>();
        VerificationReportView {
            archive_path,
            verification_status: match self.summary.verdict {
                VerificationVerdict::Verified => "verified",
                VerificationVerdict::Refused => "refused",
            },
            safe_for_strict_extraction: self.summary.safe_for_strict_extraction,
            refusal_reasons,
            verified_extent_count: self.summary.verified_extent_count,
            failed_check_count: self.summary.failed_check_count,
        }
    }
}

fn refusal_reason_slug(reason: RefusalReason) -> &'static str {
    match reason {
        RefusalReason::CorruptedRequiredBlocks => "corrupted_required_blocks",
    }
}

impl Ord for VerificationVerdict {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use VerificationVerdict::{Refused, Verified};
        match (self, other) {
            (Verified, Verified) | (Refused, Refused) => std::cmp::Ordering::Equal,
            (Verified, Refused) => std::cmp::Ordering::Less,
            (Refused, Verified) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for VerificationVerdict {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::{build_extraction_report, RefusedFileReport, SafeFileReport};

    #[test]
    fn canonical_verification_model_is_deterministic() {
        let (_kind, report) = build_extraction_report(
            vec![SafeFileReport {
                path: "b.txt".to_string(),
            }],
            vec![RefusedFileReport {
                path: "a.txt".to_string(),
                reason: RefusalReason::CorruptedRequiredBlocks,
            }],
        );
        let model = VerificationModel::from_extraction_report(&report, 2);
        assert_eq!(model.summary.verdict, VerificationVerdict::Refused);
        assert_eq!(model.summary.verified_extent_count, 2);
        assert_eq!(model.files[0].path, "a.txt");
        assert_eq!(model.files[1].path, "b.txt");
        assert_eq!(
            model.failure_domains.identity_resolution,
            IdentityResolutionStatus::Partial
        );
    }

    #[test]
    fn report_view_is_derived_from_canonical_verification_model() {
        let (_kind, report) = build_extraction_report(
            vec![SafeFileReport {
                path: "b.txt".to_string(),
            }],
            vec![RefusedFileReport {
                path: "a.txt".to_string(),
                reason: RefusalReason::CorruptedRequiredBlocks,
            }],
        );
        let model = VerificationModel::from_extraction_report(&report, 3);
        let view = model.to_report_view("archive.crs".to_string());
        assert_eq!(view.archive_path, "archive.crs");
        assert_eq!(view.verification_status, "refused");
        assert_eq!(
            view.refusal_reasons,
            vec!["corrupted_required_blocks: a.txt".to_string()]
        );
        assert_eq!(view.verified_extent_count, 3);
        assert_eq!(view.failed_check_count, 1);
    }
}
