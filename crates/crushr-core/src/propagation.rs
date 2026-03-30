// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PROPAGATION_REPORT_VERSION: u32 = 1;
pub const FORMAT_FAMILY_MINIMAL_V1: &str = "minimal-v1";

pub const STRUCTURE_FTR4: &str = "structure:ftr4";
pub const STRUCTURE_TAIL_FRAME: &str = "structure:tail_frame";
pub const STRUCTURE_IDX3: &str = "structure:idx3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationNodeKind {
    Footer,
    TailFrame,
    Index,
    Block,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationDependencyReason {
    RequiresFooterReachability,
    RequiresTailFrame,
    RequiresIndex,
    RequiresMetadataMapping,
    RequiresBlockPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyLinkKind {
    Direct,
    Propagated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationImpactReason {
    CorruptedRequiredStructure,
    CorruptedRequiredBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivatedImpactKind {
    BlocksCanonicalExtraction,
    CausesMetadataDegraded,
    LeavesUnrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryTrustClass {
    Canonical,
    MetadataDegraded,
    RecoveredNamed,
    RecoveredAnonymous,
    Unrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropagationNode {
    pub id: String,
    pub kind: PropagationNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropagationEdge {
    pub from: String,
    pub to: String,
    pub reason: PropagationDependencyReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileImpactCause {
    pub cause_node: String,
    pub reason: PropagationImpactReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileImpactV1 {
    pub file_path: String,
    pub required_nodes: Vec<String>,
    pub hypothetical_impacts_if_corrupted: Vec<FileImpactCause>,
    pub actual_impacts_from_current_corruption: Vec<FileImpactCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedCorruptionV1 {
    pub structure_nodes: Vec<String>,
    pub blocks: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredStructureV1 {
    pub structure_node: String,
    pub reason: PropagationDependencyReason,
    pub required_for_all_entries: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryDependencyV1 {
    pub node: String,
    pub link_kind: DependencyLinkKind,
    pub reason: PropagationDependencyReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivatedImpactV1 {
    pub cause_node: String,
    pub reason: PropagationImpactReason,
    pub impact_kind: ActivatedImpactKind,
    pub affected_entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryImpactV1 {
    pub file_path: String,
    pub dependencies: Vec<EntryDependencyV1>,
    pub activated_causes: Vec<FileImpactCause>,
    pub canonical_blocked: bool,
    pub canonical_blocked_reasons: Vec<PropagationImpactReason>,
    pub supported_trust_classes: Vec<EntryTrustClass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropagationReportV1 {
    pub report_version: u32,
    pub format_family: String,
    pub report_kind: String,
    pub corrupted_structure_nodes: Vec<String>,
    pub corrupted_blocks: Vec<u32>,
    pub nodes: Vec<PropagationNode>,
    pub edges: Vec<PropagationEdge>,
    pub per_file_impacts: Vec<FileImpactV1>,
    pub detected_corruption: DetectedCorruptionV1,
    pub required_structures: Vec<RequiredStructureV1>,
    pub activated_impacts: Vec<ActivatedImpactV1>,
    pub entry_impacts: Vec<EntryImpactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDependencyV1 {
    pub file_path: String,
    pub required_blocks: Vec<u32>,
}

pub fn build_structural_failure_report_v1(corrupted_structures: &[&str]) -> PropagationReportV1 {
    let corrupted_structures = corrupted_structures
        .iter()
        .map(|v| (*v).to_string())
        .collect::<BTreeSet<_>>();
    build_propagation_report_v1(&[], &corrupted_structures, &BTreeSet::new())
}

pub fn build_propagation_report_v1(
    files: &[FileDependencyV1],
    corrupted_structure_nodes: &BTreeSet<String>,
    corrupted_blocks: &BTreeSet<u32>,
) -> PropagationReportV1 {
    let mut normalized_files = files.to_vec();
    normalized_files.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    for f in &mut normalized_files {
        f.required_blocks.sort_unstable();
        f.required_blocks.dedup();
    }

    let mut block_ids = BTreeSet::new();
    for file in &normalized_files {
        for block_id in &file.required_blocks {
            block_ids.insert(*block_id);
        }
    }

    let mut nodes = vec![
        PropagationNode {
            id: STRUCTURE_FTR4.to_string(),
            kind: PropagationNodeKind::Footer,
        },
        PropagationNode {
            id: STRUCTURE_TAIL_FRAME.to_string(),
            kind: PropagationNodeKind::TailFrame,
        },
        PropagationNode {
            id: STRUCTURE_IDX3.to_string(),
            kind: PropagationNodeKind::Index,
        },
    ];

    for block_id in &block_ids {
        nodes.push(PropagationNode {
            id: block_node_id(*block_id),
            kind: PropagationNodeKind::Block,
        });
    }

    for file in &normalized_files {
        nodes.push(PropagationNode {
            id: file_node_id(&file.file_path),
            kind: PropagationNodeKind::File,
        });
    }

    let mut edges = vec![
        PropagationEdge {
            from: STRUCTURE_FTR4.to_string(),
            to: STRUCTURE_TAIL_FRAME.to_string(),
            reason: PropagationDependencyReason::RequiresTailFrame,
        },
        PropagationEdge {
            from: STRUCTURE_TAIL_FRAME.to_string(),
            to: STRUCTURE_IDX3.to_string(),
            reason: PropagationDependencyReason::RequiresIndex,
        },
    ];

    for file in &normalized_files {
        let file_node = file_node_id(&file.file_path);
        edges.push(PropagationEdge {
            from: STRUCTURE_IDX3.to_string(),
            to: file_node.clone(),
            reason: PropagationDependencyReason::RequiresMetadataMapping,
        });
        for block_id in &file.required_blocks {
            edges.push(PropagationEdge {
                from: block_node_id(*block_id),
                to: file_node.clone(),
                reason: PropagationDependencyReason::RequiresBlockPayload,
            });
        }
    }

    edges.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then(a.to.cmp(&b.to))
            .then(reason_rank(&a.reason).cmp(&reason_rank(&b.reason)))
    });

    let required_structures = vec![
        RequiredStructureV1 {
            structure_node: STRUCTURE_FTR4.to_string(),
            reason: PropagationDependencyReason::RequiresFooterReachability,
            required_for_all_entries: true,
        },
        RequiredStructureV1 {
            structure_node: STRUCTURE_TAIL_FRAME.to_string(),
            reason: PropagationDependencyReason::RequiresTailFrame,
            required_for_all_entries: true,
        },
        RequiredStructureV1 {
            structure_node: STRUCTURE_IDX3.to_string(),
            reason: PropagationDependencyReason::RequiresIndex,
            required_for_all_entries: true,
        },
    ];

    let mut per_file_impacts = Vec::with_capacity(normalized_files.len());
    let mut entry_impacts = Vec::with_capacity(normalized_files.len());
    for file in normalized_files {
        let mut required_nodes = vec![
            STRUCTURE_FTR4.to_string(),
            STRUCTURE_TAIL_FRAME.to_string(),
            STRUCTURE_IDX3.to_string(),
        ];
        for block_id in &file.required_blocks {
            required_nodes.push(block_node_id(*block_id));
        }

        let mut hypothetical = vec![
            FileImpactCause {
                cause_node: STRUCTURE_FTR4.to_string(),
                reason: PropagationImpactReason::CorruptedRequiredStructure,
            },
            FileImpactCause {
                cause_node: STRUCTURE_TAIL_FRAME.to_string(),
                reason: PropagationImpactReason::CorruptedRequiredStructure,
            },
            FileImpactCause {
                cause_node: STRUCTURE_IDX3.to_string(),
                reason: PropagationImpactReason::CorruptedRequiredStructure,
            },
        ];
        for block_id in &file.required_blocks {
            hypothetical.push(FileImpactCause {
                cause_node: block_node_id(*block_id),
                reason: PropagationImpactReason::CorruptedRequiredBlock,
            });
        }

        let mut actual = Vec::new();
        for node in [STRUCTURE_FTR4, STRUCTURE_TAIL_FRAME, STRUCTURE_IDX3] {
            if corrupted_structure_nodes.contains(node) {
                actual.push(FileImpactCause {
                    cause_node: node.to_string(),
                    reason: PropagationImpactReason::CorruptedRequiredStructure,
                });
            }
        }
        for block_id in &file.required_blocks {
            if corrupted_blocks.contains(block_id) {
                actual.push(FileImpactCause {
                    cause_node: block_node_id(*block_id),
                    reason: PropagationImpactReason::CorruptedRequiredBlock,
                });
            }
        }

        per_file_impacts.push(FileImpactV1 {
            file_path: file.file_path.clone(),
            required_nodes,
            hypothetical_impacts_if_corrupted: hypothetical,
            actual_impacts_from_current_corruption: actual.clone(),
        });

        let mut dependencies = vec![
            EntryDependencyV1 {
                node: STRUCTURE_IDX3.to_string(),
                link_kind: DependencyLinkKind::Direct,
                reason: PropagationDependencyReason::RequiresMetadataMapping,
            },
            EntryDependencyV1 {
                node: STRUCTURE_TAIL_FRAME.to_string(),
                link_kind: DependencyLinkKind::Propagated,
                reason: PropagationDependencyReason::RequiresTailFrame,
            },
            EntryDependencyV1 {
                node: STRUCTURE_FTR4.to_string(),
                link_kind: DependencyLinkKind::Propagated,
                reason: PropagationDependencyReason::RequiresFooterReachability,
            },
        ];
        for block_id in &file.required_blocks {
            dependencies.push(EntryDependencyV1 {
                node: block_node_id(*block_id),
                link_kind: DependencyLinkKind::Direct,
                reason: PropagationDependencyReason::RequiresBlockPayload,
            });
        }

        let canonical_blocked = !actual.is_empty();
        let canonical_blocked_reasons = actual.iter().map(|c| c.reason.clone()).collect::<Vec<_>>();
        let supported_trust_classes = if !canonical_blocked {
            vec![EntryTrustClass::Canonical]
        } else if actual
            .iter()
            .any(|cause| cause.reason == PropagationImpactReason::CorruptedRequiredBlock)
        {
            vec![EntryTrustClass::Unrecoverable]
        } else if actual
            .iter()
            .any(|cause| cause.cause_node == STRUCTURE_IDX3)
        {
            vec![EntryTrustClass::RecoveredAnonymous]
        } else {
            vec![
                EntryTrustClass::RecoveredNamed,
                EntryTrustClass::MetadataDegraded,
            ]
        };

        entry_impacts.push(EntryImpactV1 {
            file_path: file.file_path,
            dependencies,
            activated_causes: actual,
            canonical_blocked,
            canonical_blocked_reasons,
            supported_trust_classes,
        });
    }

    let mut activated_impacts = Vec::new();
    for structure in [STRUCTURE_FTR4, STRUCTURE_TAIL_FRAME, STRUCTURE_IDX3] {
        if corrupted_structure_nodes.contains(structure) {
            let affected_entries = entry_impacts
                .iter()
                .filter(|entry| {
                    entry
                        .activated_causes
                        .iter()
                        .any(|cause| cause.cause_node == structure)
                })
                .map(|entry| entry.file_path.clone())
                .collect::<Vec<_>>();
            let impact_kind = if structure == STRUCTURE_IDX3 {
                ActivatedImpactKind::CausesMetadataDegraded
            } else {
                ActivatedImpactKind::BlocksCanonicalExtraction
            };
            activated_impacts.push(ActivatedImpactV1 {
                cause_node: structure.to_string(),
                reason: PropagationImpactReason::CorruptedRequiredStructure,
                impact_kind,
                affected_entries,
            });
        }
    }
    for block_id in corrupted_blocks {
        let cause_node = block_node_id(*block_id);
        let affected_entries = entry_impacts
            .iter()
            .filter(|entry| {
                entry
                    .activated_causes
                    .iter()
                    .any(|cause| cause.cause_node == cause_node)
            })
            .map(|entry| entry.file_path.clone())
            .collect::<Vec<_>>();
        if !affected_entries.is_empty() {
            activated_impacts.push(ActivatedImpactV1 {
                cause_node,
                reason: PropagationImpactReason::CorruptedRequiredBlock,
                impact_kind: ActivatedImpactKind::LeavesUnrecoverable,
                affected_entries,
            });
        }
    }
    activated_impacts.sort_by(|a, b| a.cause_node.cmp(&b.cause_node));

    PropagationReportV1 {
        report_version: PROPAGATION_REPORT_VERSION,
        format_family: FORMAT_FAMILY_MINIMAL_V1.to_string(),
        report_kind: "corruption_propagation_graph".to_string(),
        corrupted_structure_nodes: corrupted_structure_nodes.iter().cloned().collect(),
        corrupted_blocks: corrupted_blocks.iter().copied().collect(),
        nodes,
        edges,
        per_file_impacts,
        detected_corruption: DetectedCorruptionV1 {
            structure_nodes: corrupted_structure_nodes.iter().cloned().collect(),
            blocks: corrupted_blocks.iter().copied().collect(),
        },
        required_structures,
        activated_impacts,
        entry_impacts,
    }
}

fn reason_rank(value: &PropagationDependencyReason) -> u8 {
    match value {
        PropagationDependencyReason::RequiresFooterReachability => 0,
        PropagationDependencyReason::RequiresTailFrame => 1,
        PropagationDependencyReason::RequiresIndex => 2,
        PropagationDependencyReason::RequiresMetadataMapping => 3,
        PropagationDependencyReason::RequiresBlockPayload => 4,
    }
}

pub fn file_node_id(path: &str) -> String {
    format!("file:{path}")
}

pub fn block_node_id(block_id: u32) -> String {
    format!("block:{block_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_failure_helper_maps_to_required_structure_impacts() {
        let report = build_structural_failure_report_v1(&[
            STRUCTURE_FTR4,
            STRUCTURE_TAIL_FRAME,
            STRUCTURE_IDX3,
        ]);

        assert_eq!(
            report.corrupted_structure_nodes,
            vec![
                STRUCTURE_FTR4.to_string(),
                STRUCTURE_IDX3.to_string(),
                STRUCTURE_TAIL_FRAME.to_string(),
            ]
        );
        assert!(report.nodes.iter().any(|n| n.id == STRUCTURE_FTR4));
        assert_eq!(report.detected_corruption.structure_nodes.len(), 3);
    }

    #[test]
    fn report_is_deterministic_and_ordered() {
        let files = vec![
            FileDependencyV1 {
                file_path: "b.txt".to_string(),
                required_blocks: vec![3],
            },
            FileDependencyV1 {
                file_path: "a.txt".to_string(),
                required_blocks: vec![2, 1, 1],
            },
        ];
        let report = build_propagation_report_v1(&files, &BTreeSet::new(), &BTreeSet::from([2]));

        assert_eq!(report.nodes[0].id, STRUCTURE_FTR4);
        assert_eq!(report.nodes[3].id, "block:1");
        assert_eq!(report.nodes[6].id, "file:a.txt");
        assert_eq!(report.per_file_impacts[0].file_path, "a.txt");
        assert_eq!(report.per_file_impacts[0].required_nodes[3], "block:1");
        assert_eq!(
            report.per_file_impacts[0].actual_impacts_from_current_corruption,
            vec![FileImpactCause {
                cause_node: "block:2".to_string(),
                reason: PropagationImpactReason::CorruptedRequiredBlock,
            }]
        );
        assert_eq!(
            report.entry_impacts[0].supported_trust_classes,
            vec![EntryTrustClass::Unrecoverable]
        );
    }

    #[test]
    fn shared_structure_corruption_impacts_all_files() {
        let files = vec![
            FileDependencyV1 {
                file_path: "x".to_string(),
                required_blocks: vec![1],
            },
            FileDependencyV1 {
                file_path: "y".to_string(),
                required_blocks: vec![2],
            },
        ];
        let report = build_propagation_report_v1(
            &files,
            &BTreeSet::from([STRUCTURE_IDX3.to_string()]),
            &BTreeSet::new(),
        );

        assert!(
            report
                .per_file_impacts
                .iter()
                .all(|f| f.actual_impacts_from_current_corruption
                    == vec![FileImpactCause {
                        cause_node: STRUCTURE_IDX3.to_string(),
                        reason: PropagationImpactReason::CorruptedRequiredStructure,
                    }])
        );
        assert!(report.entry_impacts.iter().all(
            |entry| entry.supported_trust_classes == vec![EntryTrustClass::RecoveredAnonymous]
        ));
    }
}
