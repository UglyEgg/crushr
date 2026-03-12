use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionOutcomeKind {
    Success,
    PartialRefusal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    CorruptedRequiredBlocks,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefusedFileReport {
    pub path: String,
    pub reason: RefusalReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeFileReport {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub overall_status: &'static str,
    pub maximal_safe_set_computed: bool,
    pub safe_files: Vec<SafeFileReport>,
    pub refused_files: Vec<RefusedFileReport>,
    pub safe_file_count: usize,
    pub refused_file_count: usize,
}

pub fn classify_refusal_paths(
    candidate_paths: impl IntoIterator<Item = String>,
    corrupted_blocks: &BTreeSet<u32>,
    required_blocks_for_path: impl Fn(&str) -> Vec<u32>,
) -> (Vec<SafeFileReport>, Vec<RefusedFileReport>) {
    let mut safe_files = Vec::new();
    let mut refused_files = Vec::new();

    for path in candidate_paths {
        let requires_corrupted_block = required_blocks_for_path(&path)
            .into_iter()
            .any(|block_id| corrupted_blocks.contains(&block_id));

        if requires_corrupted_block {
            refused_files.push(RefusedFileReport {
                path,
                reason: RefusalReason::CorruptedRequiredBlocks,
            });
        } else {
            safe_files.push(SafeFileReport { path });
        }
    }

    (safe_files, refused_files)
}

pub fn build_extraction_report(
    safe_files: Vec<SafeFileReport>,
    refused_files: Vec<RefusedFileReport>,
) -> (ExtractionOutcomeKind, ExtractionReport) {
    let outcome_kind = if refused_files.is_empty() {
        ExtractionOutcomeKind::Success
    } else {
        ExtractionOutcomeKind::PartialRefusal
    };

    let safe_file_count = safe_files.len();
    let refused_file_count = refused_files.len();

    (
        outcome_kind,
        ExtractionReport {
            overall_status: match outcome_kind {
                ExtractionOutcomeKind::Success => "success",
                ExtractionOutcomeKind::PartialRefusal => "partial_refusal",
            },
            maximal_safe_set_computed: true,
            safe_files,
            refused_files,
            safe_file_count,
            refused_file_count,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusal_classification_and_report_are_deterministic() {
        let candidates = vec!["a.txt".to_string(), "b.txt".to_string()];
        let corrupted_blocks = BTreeSet::from([2u32]);

        let (safe_files, refused_files) =
            classify_refusal_paths(candidates, &corrupted_blocks, |p| match p {
                "a.txt" => vec![2],
                "b.txt" => vec![1],
                _ => vec![],
            });

        assert_eq!(
            safe_files,
            vec![SafeFileReport {
                path: "b.txt".into()
            }]
        );
        assert_eq!(
            refused_files,
            vec![RefusedFileReport {
                path: "a.txt".into(),
                reason: RefusalReason::CorruptedRequiredBlocks,
            }]
        );

        let (kind, report) = build_extraction_report(safe_files, refused_files);
        assert_eq!(kind, ExtractionOutcomeKind::PartialRefusal);
        assert_eq!(report.overall_status, "partial_refusal");
        assert!(report.maximal_safe_set_computed);
        assert_eq!(report.safe_file_count, 1);
        assert_eq!(report.refused_file_count, 1);
    }
}
