//! Snapshot envelope types shared by tools and the TUI.
//!
//! The normative JSON contract is documented in `docs/SNAPSHOT_FORMAT.md`.

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

/// Minimal info snapshot payload (v1 skeleton).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfoSnapshotV1 {
    pub summary: serde_json::Value,
    pub tail_frames: serde_json::Value,
    pub dicts: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<serde_json::Value>,
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

    #[test]
    fn fingerprint_is_stable() {
        let idx = [1u8; 32];
        let ftr = [2u8; 32];
        let a = ArchiveFingerprint::from_tail_hashes(idx, ftr);
        let b = ArchiveFingerprint::from_tail_hashes(idx, ftr);
        assert_eq!(a, b);
    }

    #[test]
    fn envelope_serializes() {
        let fp = ArchiveFingerprint("deadbeef".into());
        let env = SnapshotEnvelope::new(
            "crushr-info",
            "0.0.0",
            "1970-01-01T00:00:00Z",
            fp,
            InfoSnapshotV1::default(),
        );
        let _ = serde_json::to_string(&env).unwrap();
    }
}
