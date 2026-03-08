use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileExtentV1 {
    pub block_id: u32,
    pub offset_in_block: u32,
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntryV1 {
    pub file_id: u64,
    pub path: String,
    pub extents: Vec<FileExtentV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffectedFileV1 {
    pub file_id: u64,
    pub path: String,
    pub affected_extents: Vec<FileExtentV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnaffectedFileV1 {
    pub file_id: u64,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReportV1 {
    pub schema_version: u32,
    pub corrupted_blocks: Vec<u32>,
    pub affected_files: Vec<AffectedFileV1>,
    pub unaffected_files: Vec<UnaffectedFileV1>,
}

pub fn enumerate_impact_v1(
    corrupted_blocks: &BTreeSet<u32>,
    files: &[FileEntryV1],
) -> ImpactReportV1 {
    let mut affected_files = Vec::new();
    let mut unaffected_files = Vec::new();

    for file in files {
        let mut impacted_extents: Vec<FileExtentV1> = file
            .extents
            .iter()
            .filter(|e| corrupted_blocks.contains(&e.block_id))
            .cloned()
            .collect();
        impacted_extents.sort_by_key(|e| (e.block_id, e.offset_in_block, e.len));

        if impacted_extents.is_empty() {
            unaffected_files.push(UnaffectedFileV1 {
                file_id: file.file_id,
                path: file.path.clone(),
            });
        } else {
            affected_files.push(AffectedFileV1 {
                file_id: file.file_id,
                path: file.path.clone(),
                affected_extents: impacted_extents,
            });
        }
    }

    affected_files.sort_by(|a, b| a.path.cmp(&b.path).then(a.file_id.cmp(&b.file_id)));
    unaffected_files.sort_by(|a, b| a.path.cmp(&b.path).then(a.file_id.cmp(&b.file_id)));

    ImpactReportV1 {
        schema_version: 1,
        corrupted_blocks: corrupted_blocks.iter().copied().collect(),
        affected_files,
        unaffected_files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerates_affected_files_without_decompression() {
        let files = vec![
            FileEntryV1 {
                file_id: 1,
                path: "README.md".into(),
                extents: vec![FileExtentV1 {
                    block_id: 1,
                    offset_in_block: 0,
                    len: 10,
                }],
            },
            FileEntryV1 {
                file_id: 2,
                path: "assets/a.bin".into(),
                extents: vec![FileExtentV1 {
                    block_id: 2,
                    offset_in_block: 0,
                    len: 20,
                }],
            },
            FileEntryV1 {
                file_id: 3,
                path: "assets/b.bin".into(),
                extents: vec![
                    FileExtentV1 {
                        block_id: 2,
                        offset_in_block: 20,
                        len: 5,
                    },
                    FileExtentV1 {
                        block_id: 3,
                        offset_in_block: 0,
                        len: 5,
                    },
                ],
            },
        ];
        let corrupted = BTreeSet::from([2u32]);
        let report = enumerate_impact_v1(&corrupted, &files);
        assert_eq!(report.corrupted_blocks, vec![2]);
        assert_eq!(report.unaffected_files.len(), 1);
        assert_eq!(report.unaffected_files[0].path, "README.md");
        assert_eq!(report.affected_files.len(), 2);
    }
}
