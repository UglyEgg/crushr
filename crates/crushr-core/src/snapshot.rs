//! Snapshot envelope types shared by tools and the TUI.
//!
//! The normative JSON contract is documented in `docs/SNAPSHOT_FORMAT.md`.

use crate::open::OpenArchiveV1;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Version of the JSON snapshot schema.
///
/// This is independent of the on-disk archive format version.
pub const SNAPSHOT_SCHEMA_V1: u32 = 1;

/// A stable identifier for an archive instance used to match snapshots.
///
/// This is a *fingerprint*, not a security boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveFingerprint(pub String);

impl fmt::Display for ArchiveFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ArchiveFingerprint {
    /// Derive a fingerprint from tail hashes.
    ///
    /// Recommended input is the last valid tail frame's `index_hash` and `footer_hash`.
    pub fn from_tail_hashes(index_hash: [u8; 32], footer_hash: [u8; 32]) -> Self {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&index_hash);
        buf[32..].copy_from_slice(&footer_hash);
        let h = blake3::hash(&buf);
        Self(h.to_hex().to_string())
    }
}

/// Common snapshot envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEnvelope<T> {
    pub schema_version: u32,
    pub tool: String,
    pub tool_version: String,
    pub generated_at_utc: String,
    pub archive_fingerprint: ArchiveFingerprint,
    pub payload: T,
}

impl<T> SnapshotEnvelope<T> {
    pub fn new(
        tool: impl Into<String>,
        tool_version: impl Into<String>,
        generated_at_utc: impl Into<String>,
        archive_fingerprint: ArchiveFingerprint,
        payload: T,
    ) -> Self {
        Self {
            schema_version: SNAPSHOT_SCHEMA_V1,
            tool: tool.into(),
            tool_version: tool_version.into(),
            generated_at_utc: generated_at_utc.into(),
            archive_fingerprint,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoSummaryV1 {
    pub archive_len: u64,
    pub footer_offset: u64,
    pub footer_len: u64,
    pub has_footer: bool,
    pub tail_frame_offset: u64,
    pub tail_frame_len: u64,
    pub raw_idx3_len: u64,
    pub has_dct1: bool,
    pub has_ldg1: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailFrameSnapshotV1 {
    pub blocks_end_offset: u64,
    pub dct_offset: u64,
    pub dct_len: u64,
    pub index_offset: u64,
    pub index_len: u64,
    pub ledger_offset: u64,
    pub ledger_len: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictSummaryV1 {
    pub count: u32,
}

/// Minimal info snapshot payload (v1, real archive metadata path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoSnapshotV1 {
    pub summary: InfoSummaryV1,
    pub tail_frames: Vec<TailFrameSnapshotV1>,
    pub dicts: DictSummaryV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<serde_json::Value>,
}

pub fn info_snapshot_from_open_archive(open: &OpenArchiveV1) -> InfoSnapshotV1 {
    let footer = &open.tail.footer;

    InfoSnapshotV1 {
        summary: InfoSummaryV1 {
            archive_len: open.archive_len,
            footer_offset: open.footer_offset,
            footer_len: open.footer_len,
            has_footer: true,
            tail_frame_offset: open.tail_frame_offset,
            tail_frame_len: open.tail_frame_len,
            raw_idx3_len: open.tail.idx3_bytes.len() as u64,
            has_dct1: open.tail.dct1.is_some(),
            has_ldg1: open.tail.ldg1.is_some(),
        },
        tail_frames: vec![TailFrameSnapshotV1 {
            blocks_end_offset: footer.blocks_end_offset,
            dct_offset: footer.dct_offset,
            dct_len: footer.dct_len,
            index_offset: footer.index_offset,
            index_len: footer.index_len,
            ledger_offset: footer.ledger_offset,
            ledger_len: footer.ledger_len,
        }],
        dicts: DictSummaryV1 {
            count: open
                .tail
                .dct1
                .as_ref()
                .map_or(0, |d| d.entries.len() as u32),
        },
        ledger: None,
        files: None,
        blocks: None,
    }
}

pub fn info_envelope_from_open_archive(
    open: &OpenArchiveV1,
    tool_version: &str,
    generated_at_utc: &str,
) -> SnapshotEnvelope<InfoSnapshotV1> {
    SnapshotEnvelope::new(
        "crushr-info",
        tool_version,
        generated_at_utc,
        ArchiveFingerprint::from_tail_hashes(
            open.tail.footer.index_hash,
            open.tail.footer.footer_hash,
        ),
        info_snapshot_from_open_archive(open),
    )
}

pub fn serialize_snapshot_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string_pretty(value)
}

/// Minimal fsck snapshot payload (v1 skeleton).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FsckSnapshotV1 {
    pub verify: serde_json::Value,
    pub blast_radius: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salvage_plan: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dump_paths: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open::open_archive_v1;
    use crate::{io::Len, io::ReadAt};
    use anyhow::Result;
    use crushr_format::{
        dct1::{Dct1Entry, Dct1Table},
        ledger::LedgerBlob,
        tailframe::assemble_tail_frame,
    };
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone)]
    struct MemReader {
        bytes: Vec<u8>,
    }

    impl ReadAt for MemReader {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
            let offset = offset as usize;
            if offset >= self.bytes.len() {
                return Ok(0);
            }
            let n = (self.bytes.len() - offset).min(buf.len());
            buf[..n].copy_from_slice(&self.bytes[offset..offset + n]);
            Ok(n)
        }
    }

    impl Len for MemReader {
        fn len(&self) -> Result<u64> {
            Ok(self.bytes.len() as u64)
        }
    }

    fn open_from_bytes(bytes: Vec<u8>) -> OpenArchiveV1 {
        open_archive_v1(&MemReader { bytes }).unwrap()
    }

    fn build_archive(
        blocks_end_offset: u64,
        dct1: Option<&Dct1Table>,
        idx3: &[u8],
        ledger: Option<&LedgerBlob>,
    ) -> Vec<u8> {
        let mut bytes = vec![0u8; blocks_end_offset as usize];
        let tail = assemble_tail_frame(blocks_end_offset, dct1, idx3, ledger).unwrap();
        bytes.extend_from_slice(&tail);
        bytes
    }

    #[test]
    fn fingerprint_is_stable() {
        let idx = [1u8; 32];
        let ftr = [2u8; 32];
        let a = ArchiveFingerprint::from_tail_hashes(idx, ftr);
        let b = ArchiveFingerprint::from_tail_hashes(idx, ftr);
        assert_eq!(a, b);
    }

    #[test]
    fn minimal_archive_emits_info_snapshot_json() {
        let idx3 = b"IDX3\x10\x11\x12";
        let bytes = build_archive(16, None, idx3, None);
        let open = open_from_bytes(bytes);

        let env = info_envelope_from_open_archive(&open, "0.2.2", "1970-01-01T00:00:00Z");
        let json = serialize_snapshot_json(&env).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["tool"], "crushr-info");
        assert_eq!(value["tool_version"], "0.2.2");
        assert!(value["archive_fingerprint"].is_string());

        let summary = &value["payload"]["summary"];
        assert_eq!(summary["archive_len"], open.archive_len);
        assert_eq!(summary["footer_offset"], open.footer_offset);
        assert_eq!(summary["footer_len"], open.footer_len);
        assert_eq!(summary["has_footer"], true);
        assert_eq!(summary["tail_frame_offset"], open.tail_frame_offset);
        assert_eq!(summary["tail_frame_len"], open.tail_frame_len);
        assert_eq!(summary["raw_idx3_len"], idx3.len() as u64);
        assert_eq!(summary["has_dct1"], false);
        assert_eq!(summary["has_ldg1"], false);

        let tail = &value["payload"]["tail_frames"][0];
        assert_eq!(tail["index_len"], open.tail.footer.index_len);
        assert_eq!(tail["ledger_len"], open.tail.footer.ledger_len);
    }

    #[test]
    fn archive_with_dct1_and_ldg1_reflects_presence() {
        let dct1 = Dct1Table {
            entries: vec![Dct1Entry {
                dict_id: 7,
                dict_bytes: b"abcd".to_vec(),
                dict_hash: *blake3::hash(b"abcd").as_bytes(),
            }],
        };
        let ledger = LedgerBlob::from_value(&serde_json::json!({"k":"v"})).unwrap();
        let bytes = build_archive(16, Some(&dct1), b"IDX3\x01", Some(&ledger));
        let open = open_from_bytes(bytes);

        let snap = info_snapshot_from_open_archive(&open);
        assert!(snap.summary.has_dct1);
        assert!(snap.summary.has_ldg1);
        assert_eq!(snap.dicts.count, 1);
        assert_eq!(snap.tail_frames[0].ledger_len, open.tail.footer.ledger_len);
    }

    #[test]
    fn serialization_is_deterministic_for_identical_archive_bytes() {
        let bytes = build_archive(16, None, b"IDX3\x01\x02", None);
        let open_a = open_from_bytes(bytes.clone());
        let open_b = open_from_bytes(bytes);

        let env_a = info_envelope_from_open_archive(&open_a, "0.2.2", "1970-01-01T00:00:00Z");
        let env_b = info_envelope_from_open_archive(&open_b, "0.2.2", "1970-01-01T00:00:00Z");

        let json_a = serialize_snapshot_json(&env_a).unwrap();
        let json_b = serialize_snapshot_json(&env_b).unwrap();
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn invalid_archive_fails_without_snapshot_output() {
        let err = open_archive_v1(&MemReader {
            bytes: vec![0u8; 16],
        })
        .unwrap_err();

        let msg = format!("{err:#}");
        assert!(msg.contains("archive too short") || msg.contains("parse FTR4"));
    }

    #[test]
    fn crushr_info_binary_emits_json_for_synthetic_archive() {
        let bytes = build_archive(24, None, b"IDX3\x33\x44", None);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("crushr-info-synth-{unique}.crs"));
        fs::write(&path, &bytes).unwrap();

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();

        let output = Command::new("cargo")
            .current_dir(workspace_root)
            .args([
                "run",
                "-q",
                "-p",
                "crushr",
                "--bin",
                "crushr-info",
                "--",
                path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        let _ = fs::remove_file(&path);

        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["tool"], "crushr-info");
        assert_eq!(value["payload"]["summary"]["raw_idx3_len"], 6);
    }
}
