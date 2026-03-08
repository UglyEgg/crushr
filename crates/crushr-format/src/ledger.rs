//! Ledger framing and canonical JSON helpers.
//!
//! The ledger is a small, explainable metadata payload (JSON) embedded in a tail frame.
//! It is intended to be deterministic and stable across identical inputs.

use anyhow::{bail, Context, Result};
use blake3::Hash;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{Read, Write};

/// Magic for a Ledger v1 frame.
pub const LDG1_MAGIC: [u8; 4] = *b"LDG1";

/// Maximum accepted ledger payload size.
///
/// The ledger is intended to stay small. This protects against pathological inputs and
/// corrupted length fields.
pub const LDG1_MAX_LEN: u64 = 16 * 1024 * 1024; // 16 MiB

/// A decoded ledger payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerBlob {
    /// Canonical UTF-8 JSON bytes.
    pub json: Vec<u8>,
    /// BLAKE3 hash of `json`.
    pub hash: [u8; 32],
}

impl LedgerBlob {
    /// Build a ledger blob from a JSON value by canonicalizing and hashing it.
    pub fn from_value(value: &Value) -> Result<Self> {
        let json = canonical_json_bytes(value)?;
        let hash = blake3::hash(&json).as_bytes().to_owned();
        Ok(Self { json, hash })
    }

    /// Parse and verify a ledger blob from canonical JSON bytes.
    pub fn from_canonical_json_bytes(json: Vec<u8>) -> Result<Self> {
        // Ensure it's valid JSON and also canonicalize again to guarantee determinism.
        let v: Value = serde_json::from_slice(&json).context("ledger JSON is not valid")?;
        let canon = canonical_json_bytes(&v)?;
        if canon != json {
            bail!("ledger JSON bytes are not canonical");
        }
        let hash = blake3::hash(&json).as_bytes().to_owned();
        Ok(Self { json, hash })
    }
}

/// Encode a ledger frame in LDG1 framing.
///
/// Layout (little-endian):
/// - magic: 4 bytes ("LDG1")
/// - len: u64 (payload length)
/// - hash: 32 bytes (BLAKE3 of payload JSON bytes)
/// - payload: `len` bytes (UTF-8 JSON, canonical form)
pub fn write_ldg1<W: Write>(mut w: W, blob: &LedgerBlob) -> Result<()> {
    let len: u64 = blob.json.len() as u64;
    if len > LDG1_MAX_LEN {
        bail!("ledger too large: {} bytes (max {})", len, LDG1_MAX_LEN);
    }

    w.write_all(&LDG1_MAGIC)?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&blob.hash)?;
    w.write_all(&blob.json)?;
    Ok(())
}

/// Decode and verify a ledger frame in LDG1 framing.
pub fn read_ldg1<R: Read>(mut r: R) -> Result<LedgerBlob> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic).context("reading LDG1 magic")?;
    if magic != LDG1_MAGIC {
        bail!("invalid ledger magic: expected LDG1");
    }

    let mut len_bytes = [0u8; 8];
    r.read_exact(&mut len_bytes).context("reading LDG1 len")?;
    let len = u64::from_le_bytes(len_bytes);
    if len > LDG1_MAX_LEN {
        bail!(
            "ledger length too large: {} bytes (max {})",
            len,
            LDG1_MAX_LEN
        );
    }

    let mut hash = [0u8; 32];
    r.read_exact(&mut hash).context("reading LDG1 hash")?;

    let mut json = vec![0u8; len as usize];
    r.read_exact(&mut json).context("reading LDG1 payload")?;

    let actual: Hash = blake3::hash(&json);
    if actual.as_bytes() != &hash {
        bail!("ledger payload hash mismatch");
    }

    // Enforce canonical bytes.
    LedgerBlob::from_canonical_json_bytes(json)
}

/// Canonicalize a JSON value into stable bytes.
///
/// Rules:
/// - UTF-8 JSON
/// - no insignificant whitespace
/// - object keys sorted lexicographically (recursively)
/// - arrays preserved in order
pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>> {
    let canon = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canon).context("serializing canonical JSON")?;
    Ok(bytes)
}

fn canonicalize_value(v: &Value) -> Value {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => v.clone(),
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut bt: BTreeMap<String, Value> = BTreeMap::new();
            for (k, vv) in map.iter() {
                bt.insert(k.clone(), canonicalize_value(vv));
            }
            // Convert back to serde_json::Map preserving BTreeMap iteration order.
            let mut out = serde_json::Map::with_capacity(bt.len());
            for (k, vv) in bt {
                out.insert(k, vv);
            }
            Value::Object(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_sorts_keys_recursively() {
        let v: Value = serde_json::json!({
            "z": 1,
            "a": {"b": 2, "a": 1},
            "m": [ {"y": 2, "x": 1} ]
        });

        let bytes = canonical_json_bytes(&v).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        // Top-level keys: a, m, z
        assert!(s.starts_with("{\"a\":{\"a\":1,\"b\":2},\"m\":[{\"x\":1,\"y\":2}],\"z\":1}"));
    }

    #[test]
    fn ldg1_roundtrip_and_hash_verification() {
        let v: Value = serde_json::json!({"b":1,"a":2});
        let blob = LedgerBlob::from_value(&v).unwrap();

        let mut buf = Vec::new();
        write_ldg1(&mut buf, &blob).unwrap();

        let decoded = read_ldg1(buf.as_slice()).unwrap();
        assert_eq!(decoded, blob);
    }
}
