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
}
