// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[path = "../../../crates/crushr/src/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/index_codec.rs"]
mod index_codec;

use anyhow::{Context, Result, bail};
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{scan_blocks_v1, verify_block_payloads_clean_v1},
};
use index_codec::decode_index;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const MAX_FLIP_COUNT: usize = 250_000;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CorruptionMode {
    RandomFlip,
    Overwrite,
    Truncate,
    Remove,
}

#[derive(Deserialize)]
struct CorruptionConfig {
    mode: CorruptionMode,
    seed: Option<u64>,
    flip_count: Option<usize>,
    overwrite_offset: Option<usize>,
    overwrite_len: Option<usize>,
    overwrite_value: Option<u8>,
    truncate_offset: Option<usize>,
    remove_offset: Option<usize>,
    remove_len: Option<usize>,
}

#[derive(Serialize)]
struct CorruptionResult {
    archive_bytes: Vec<u8>,
    mode_slug: &'static str,
    seed_applied: Option<u64>,
    operation_summary: String,
}

#[derive(Serialize)]
struct ArchiveInspection {
    total_bytes: usize,
    archive_hash_blake3: String,
    total_entries: usize,
    total_blocks: usize,
    extents_valid: bool,
    strict_extraction_supported: bool,
}

#[derive(Clone)]
struct SliceReader<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl ReadAt for SliceReader<'_> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> anyhow::Result<usize> {
        let Ok(start) = usize::try_from(offset) else {
            return Ok(0);
        };
        if start >= self.bytes.len() {
            return Ok(0);
        }
        let available = self.bytes.len() - start;
        let n = available.min(buf.len());
        buf[..n].copy_from_slice(&self.bytes[start..start + n]);
        Ok(n)
    }
}

impl Len for SliceReader<'_> {
    fn len(&self) -> anyhow::Result<u64> {
        Ok(self.bytes.len() as u64)
    }
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn inspect_archive(archive_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
    let summary = inspect_archive_impl(&archive_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&summary).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn corrupt_archive(archive_bytes: Vec<u8>, config: JsValue) -> Result<JsValue, JsValue> {
    let config: CorruptionConfig =
        serde_wasm_bindgen::from_value(config).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = corrupt_archive_impl(archive_bytes, config).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn inspect_archive_impl(archive_bytes: &[u8]) -> Result<ArchiveInspection> {
    let reader = SliceReader::new(archive_bytes);
    let opened = open_archive_v1(&reader).context("open archive")?;
    let index = decode_index(&opened.tail.idx3_bytes).context("decode index")?;
    let extents_valid = verify_block_payloads_clean_v1(&reader, opened.tail.footer.blocks_end_offset)
        .context("verify block payloads")?;
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset).context("scan blocks")?;

    Ok(ArchiveInspection {
        total_bytes: archive_bytes.len(),
        archive_hash_blake3: blake3::hash(archive_bytes).to_hex().to_string(),
        total_entries: index.entries.len(),
        total_blocks: blocks.len(),
        extents_valid,
        strict_extraction_supported: extents_valid,
    })
}

fn corrupt_archive_impl(mut archive_bytes: Vec<u8>, config: CorruptionConfig) -> Result<CorruptionResult> {
    if archive_bytes.is_empty() {
        bail!("archive is empty")
    }

    match config.mode {
        CorruptionMode::RandomFlip => {
            let seed = config.seed.context("seed is required for random flip")?;
            let flip_count = config.flip_count.unwrap_or(1).clamp(1, MAX_FLIP_COUNT);
            let mut rng = DeterministicRng::new(seed);

            for _ in 0..flip_count {
                let index = (rng.next_u64() as usize) % archive_bytes.len();
                let bit = (rng.next_u64() % 8) as u8;
                archive_bytes[index] ^= 1u8 << bit;
            }

            Ok(CorruptionResult {
                archive_bytes,
                mode_slug: "randflip",
                seed_applied: Some(seed),
                operation_summary: format!("flipped {flip_count} bytes deterministically"),
            })
        }
        CorruptionMode::Overwrite => {
            let offset = config.overwrite_offset.unwrap_or(0).min(archive_bytes.len());
            let len = config.overwrite_len.unwrap_or(1).max(1);
            let value = config.overwrite_value.unwrap_or(0x00);
            let end = offset.saturating_add(len).min(archive_bytes.len());

            if offset >= end {
                bail!("overwrite range is outside archive bounds")
            }

            archive_bytes[offset..end].fill(value);
            Ok(CorruptionResult {
                archive_bytes,
                mode_slug: "overwrite",
                seed_applied: None,
                operation_summary: format!("overwrote bytes {offset}..{} with 0x{value:02x}", end),
            })
        }
        CorruptionMode::Truncate => {
            let cut = config
                .truncate_offset
                .context("truncate_offset is required for truncate mode")?
                .min(archive_bytes.len());
            archive_bytes.truncate(cut);

            Ok(CorruptionResult {
                archive_bytes,
                mode_slug: "truncate",
                seed_applied: None,
                operation_summary: format!("truncated archive to {cut} bytes"),
            })
        }
        CorruptionMode::Remove => {
            let offset = config
                .remove_offset
                .context("remove_offset is required for remove mode")?
                .min(archive_bytes.len());
            let len = config.remove_len.context("remove_len is required for remove mode")?.max(1);
            let end = offset.saturating_add(len).min(archive_bytes.len());

            if offset >= end {
                bail!("remove range is outside archive bounds")
            }

            archive_bytes.drain(offset..end);

            Ok(CorruptionResult {
                archive_bytes,
                mode_slug: "remove",
                seed_applied: None,
                operation_summary: format!("removed byte range {offset}..{}", end),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bytes() -> Vec<u8> {
        (0..64).map(|v| v as u8).collect()
    }

    #[test]
    fn random_flip_is_deterministic() {
        let cfg = CorruptionConfig {
            mode: CorruptionMode::RandomFlip,
            seed: Some(42),
            flip_count: Some(8),
            overwrite_offset: None,
            overwrite_len: None,
            overwrite_value: None,
            truncate_offset: None,
            remove_offset: None,
            remove_len: None,
        };

        let out_a = corrupt_archive_impl(sample_bytes(), cfg).expect("corrupt");

        let cfg2 = CorruptionConfig {
            mode: CorruptionMode::RandomFlip,
            seed: Some(42),
            flip_count: Some(8),
            overwrite_offset: None,
            overwrite_len: None,
            overwrite_value: None,
            truncate_offset: None,
            remove_offset: None,
            remove_len: None,
        };
        let out_b = corrupt_archive_impl(sample_bytes(), cfg2).expect("corrupt");

        assert_eq!(out_a.archive_bytes, out_b.archive_bytes);
    }

    #[test]
    fn remove_mode_changes_length() {
        let cfg = CorruptionConfig {
            mode: CorruptionMode::Remove,
            seed: None,
            flip_count: None,
            overwrite_offset: None,
            overwrite_len: None,
            overwrite_value: None,
            truncate_offset: None,
            remove_offset: Some(5),
            remove_len: Some(10),
        };

        let output = corrupt_archive_impl(sample_bytes(), cfg).expect("corrupt");
        assert_eq!(output.archive_bytes.len(), 54);
    }
}
