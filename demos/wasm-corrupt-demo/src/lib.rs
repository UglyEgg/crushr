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
    random_flip_offset: Option<usize>,
    random_flip_span: Option<usize>,
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
    corruption_ranges: Vec<ByteRange>,
}

#[derive(Serialize)]
struct ArchiveInspection {
    total_bytes: usize,
    archive_hash_blake3: String,
    total_entries: usize,
    total_blocks: usize,
    extents_valid: bool,
    strict_extraction_supported: bool,
    layout_segments: Vec<LayoutSegment>,
}

#[derive(Serialize, Clone)]
struct ByteRange {
    start: u64,
    end: u64,
}

#[derive(Serialize, Clone)]
struct LayoutSegment {
    kind: &'static str,
    start: u64,
    end: u64,
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
    let layout_segments = build_layout_segments(&index, &opened);

    Ok(ArchiveInspection {
        total_bytes: archive_bytes.len(),
        archive_hash_blake3: blake3::hash(archive_bytes).to_hex().to_string(),
        total_entries: index.entries.len(),
        total_blocks: blocks.len(),
        extents_valid,
        strict_extraction_supported: extents_valid,
        layout_segments,
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
            let random_start = config.random_flip_offset.unwrap_or(0).min(archive_bytes.len().saturating_sub(1));
            let span_default = archive_bytes.len().saturating_sub(random_start);
            let random_span = config.random_flip_span.unwrap_or(span_default).max(1);
            let random_end = random_start.saturating_add(random_span).min(archive_bytes.len());

            if random_start >= random_end {
                bail!("random flip window is outside archive bounds")
            }
            let mut rng = DeterministicRng::new(seed);
            let mut flipped_positions = Vec::with_capacity(flip_count);

            for _ in 0..flip_count {
                let index = random_start + ((rng.next_u64() as usize) % (random_end - random_start));
                let bit = (rng.next_u64() % 8) as u8;
                archive_bytes[index] ^= 1u8 << bit;
                flipped_positions.push(index as u64);
            }

            Ok(CorruptionResult {
                archive_bytes,
                mode_slug: "randflip",
                seed_applied: Some(seed),
                operation_summary: format!(
                    "flipped {flip_count} bytes deterministically in window {random_start}..{}",
                    random_end
                ),
                corruption_ranges: merge_single_point_ranges(&flipped_positions),
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
                corruption_ranges: vec![ByteRange {
                    start: offset as u64,
                    end: end as u64,
                }],
            })
        }
        CorruptionMode::Truncate => {
            let original_len = archive_bytes.len();
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
                corruption_ranges: vec![ByteRange {
                    start: cut as u64,
                    end: original_len as u64,
                }],
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
                corruption_ranges: vec![ByteRange {
                    start: offset as u64,
                    end: end as u64,
                }],
            })
        }
    }
}

fn merge_single_point_ranges(points: &[u64]) -> Vec<ByteRange> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut sorted = points.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut out = Vec::new();
    let mut range_start = sorted[0];
    let mut range_end = sorted[0].saturating_add(1);

    for point in &sorted[1..] {
        let point_start = *point;
        let point_end = point_start.saturating_add(1);
        if point_start <= range_end {
            range_end = range_end.max(point_end);
            continue;
        }
        out.push(ByteRange {
            start: range_start,
            end: range_end,
        });
        range_start = point_start;
        range_end = point_end;
    }

    out.push(ByteRange {
        start: range_start,
        end: range_end,
    });
    out
}

fn build_layout_segments(
    index: &format::Index,
    opened: &crushr_core::open::OpenArchiveV1,
) -> Vec<LayoutSegment> {
    let mut payload_ranges = Vec::new();
    for entry in &index.entries {
        for ex in &entry.extents {
            let start = ex.offset;
            let end = ex.offset.saturating_add(ex.len);
            if end > start {
                payload_ranges.push(ByteRange { start, end });
            }
        }
    }

    if payload_ranges.is_empty() && opened.tail.footer.blocks_end_offset > 0 {
        payload_ranges.push(ByteRange {
            start: 0,
            end: opened.tail.footer.blocks_end_offset,
        });
    }

    let mut metadata_ranges = Vec::new();
    if opened.tail.footer.dct_len > 0 {
        metadata_ranges.push(ByteRange {
            start: opened.tail.footer.dct_offset,
            end: opened
                .tail
                .footer
                .dct_offset
                .saturating_add(opened.tail.footer.dct_len),
        });
    }
    metadata_ranges.push(ByteRange {
        start: opened.tail.footer.index_offset,
        end: opened
            .tail
            .footer
            .index_offset
            .saturating_add(opened.tail.footer.index_len),
    });
    if opened.tail.footer.ledger_len > 0 {
        metadata_ranges.push(ByteRange {
            start: opened.tail.footer.ledger_offset,
            end: opened
                .tail
                .footer
                .ledger_offset
                .saturating_add(opened.tail.footer.ledger_len),
        });
    }

    let payload_ranges = merge_ranges(payload_ranges);
    let metadata_ranges = merge_ranges(metadata_ranges);
    let tail_range = ByteRange {
        start: opened.footer_offset,
        end: opened.archive_len,
    };

    let mut segments = Vec::new();
    for range in payload_ranges {
        segments.push(LayoutSegment {
            kind: "payload",
            start: range.start,
            end: range.end,
        });
    }
    for range in metadata_ranges {
        segments.push(LayoutSegment {
            kind: "metadata",
            start: range.start,
            end: range.end,
        });
    }
    if tail_range.end > tail_range.start {
        segments.push(LayoutSegment {
            kind: "tail",
            start: tail_range.start,
            end: tail_range.end,
        });
    }

    segments.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
    segments
}

fn merge_ranges(mut ranges: Vec<ByteRange>) -> Vec<ByteRange> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));

    let mut out = Vec::new();
    let mut current = ranges[0].clone();
    for range in ranges.into_iter().skip(1) {
        if range.start <= current.end {
            current.end = current.end.max(range.end);
            continue;
        }
        out.push(current);
        current = range;
    }
    out.push(current);
    out
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
            random_flip_offset: None,
            random_flip_span: None,
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
            random_flip_offset: None,
            random_flip_span: None,
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
            random_flip_offset: None,
            random_flip_span: None,
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
