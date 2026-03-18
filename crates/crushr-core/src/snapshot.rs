//! Snapshot envelope types shared by tools and the TUI.
//!
//! The normative JSON contract is documented in `docs/SNAPSHOT_FORMAT.md`.

use crate::impact::{enumerate_impact_v1, ImpactReportV1};
use crate::io::{Len, ReadAt};
use crate::open::OpenArchiveV1;
use crate::verify::verify_block_payloads_v1;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsckVerifyV1 {
    pub status: String,
    pub has_footer: bool,
    pub has_tail_frame: bool,
    pub has_valid_idx3_hash: bool,
    pub has_valid_ldg1_hash: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsckBlastRadiusV1 {
    pub structural_components_untrusted: Vec<String>,
    pub impact: ImpactReportV1,
}

/// Minimal fsck snapshot payload (v1, read-only metadata validation path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsckSnapshotV1 {
    pub verify: FsckVerifyV1,
    pub blast_radius: FsckBlastRadiusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dump_paths: Option<serde_json::Value>,
}

pub fn fsck_clean_report() -> ImpactReportV1 {
    enumerate_impact_v1(&Default::default(), &[])
}

pub fn fsck_snapshot_from_open_archive<R: ReadAt + Len>(
    open: &OpenArchiveV1,
    reader: &R,
) -> anyhow::Result<FsckSnapshotV1> {
    let corrupted_blocks = verify_block_payloads_v1(reader, open.tail.footer.blocks_end_offset)?;

    Ok(FsckSnapshotV1 {
        verify: FsckVerifyV1 {
            status: "ok".to_string(),
            has_footer: open.footer_len > 0,
            has_tail_frame: open.tail_frame_len > 0,
            has_valid_idx3_hash: true,
            has_valid_ldg1_hash: true,
        },
        blast_radius: FsckBlastRadiusV1 {
            structural_components_untrusted: Vec::new(),
            impact: enumerate_impact_v1(&corrupted_blocks, &[]),
        },
        dump_paths: None,
    })
}

pub fn fsck_envelope_from_open_archive<R: ReadAt + Len>(
    open: &OpenArchiveV1,
    reader: &R,
    tool_version: &str,
    generated_at_utc: &str,
) -> anyhow::Result<SnapshotEnvelope<FsckSnapshotV1>> {
    Ok(SnapshotEnvelope::new(
        "crushr-fsck",
        tool_version,
        generated_at_utc,
        ArchiveFingerprint::from_tail_hashes(
            open.tail.footer.index_hash,
            open.tail.footer.footer_hash,
        ),
        fsck_snapshot_from_open_archive(open, reader)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open::open_archive_v1;
    use crate::{io::Len, io::ReadAt};
    use anyhow::Result;
    use crushr_format::{
        blk3::{write_blk3_header, Blk3Flags, Blk3Header},
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

    fn build_archive_with_real_block(payload: &[u8]) -> Vec<u8> {
        let mut blocks = Vec::new();
        let header = Blk3Header {
            header_len: 68,
            flags: Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH),
            codec: 1,
            level: 3,
            dict_id: 0,
            raw_len: payload.len() as u64,
            comp_len: payload.len() as u64,
            payload_hash: Some(*blake3::hash(payload).as_bytes()),
            raw_hash: None,
        };
        write_blk3_header(&mut blocks, &header).unwrap();
        blocks.extend_from_slice(payload);

        let blocks_end_offset = blocks.len() as u64;
        let tail = assemble_tail_frame(blocks_end_offset, None, b"IDX33D", None).unwrap();
        blocks.extend_from_slice(&tail);
        blocks
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
        let bytes = build_archive_with_real_block(b"payload");
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

    #[test]
    fn crushr_extract_verify_emits_clean_json_for_synthetic_archive() {
        let bytes = build_archive_with_real_block(b"payload");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("crushr-extract-verify-synth-valid-{unique}.crs"));
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
                "crushr-extract",
                "--",
                "--verify",
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
        assert_eq!(value["verification_status"], "verified");
        assert_eq!(value["safe_for_strict_extraction"], true);
        assert_eq!(value["failed_check_count"], 0);
    }

    #[test]
    fn crushr_extract_verify_fails_cleanly_for_corrupt_footer() {
        let mut bytes = build_archive_with_real_block(b"payload");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("crushr-extract-verify-synth-ftr-{unique}.crs"));
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
                "crushr-extract",
                "--",
                "--verify",
                path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        let _ = fs::remove_file(&path);

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn crushr_extract_verify_fails_cleanly_for_corrupt_idx3_hash() {
        let mut bytes = build_archive_with_real_block(b"payload");
        let open = open_archive_v1(&MemReader {
            bytes: bytes.clone(),
        })
        .unwrap();
        let idx_off = open.tail.footer.index_offset as usize;
        bytes[idx_off + 1] ^= 0x01;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("crushr-extract-verify-synth-idx-{unique}.crs"));
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
                "crushr-extract",
                "--",
                "--verify",
                path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        let _ = fs::remove_file(&path);

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn crushr_info_binary_fails_cleanly_for_corrupt_footer() {
        let mut bytes = build_archive_with_real_block(b"payload");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("crushr-info-synth-ftr-{unique}.crs"));
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

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn crushr_info_and_extract_verify_use_same_exit_code_for_missing_archive() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("crushr-missing-{unique}.crs"));

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();

        let info = Command::new("cargo")
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

        let verify = Command::new("cargo")
            .current_dir(workspace_root)
            .args([
                "run",
                "-q",
                "-p",
                "crushr",
                "--bin",
                "crushr-extract",
                "--",
                "--verify",
                path.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();

        assert_eq!(info.status.code(), Some(2));
        assert_eq!(verify.status.code(), Some(2));
        assert!(info.stdout.is_empty());
        assert!(verify.stdout.is_empty());
    }

    #[test]
    fn fsck_serialization_is_deterministic_for_identical_archive_bytes() {
        let bytes = build_archive_with_real_block(b"payload");
        let open_a = open_from_bytes(bytes.clone());
        let open_b = open_from_bytes(bytes.clone());

        let reader_a = MemReader {
            bytes: bytes.clone(),
        };
        let reader_b = MemReader { bytes };

        let env_a =
            fsck_envelope_from_open_archive(&open_a, &reader_a, "0.2.2", "1970-01-01T00:00:00Z")
                .unwrap();
        let env_b =
            fsck_envelope_from_open_archive(&open_b, &reader_b, "0.2.2", "1970-01-01T00:00:00Z")
                .unwrap();

        let json_a = serialize_snapshot_json(&env_a).unwrap();
        let json_b = serialize_snapshot_json(&env_b).unwrap();
        assert_eq!(json_a, json_b);
    }
}
