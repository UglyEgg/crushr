// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[path = "../../../crates/crushr/src/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/index_codec.rs"]
mod index_codec;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/extraction_payload_core.rs"]
mod extraction_payload_core;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/recovery_classification.rs"]
mod recovery_classification;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/introspection.rs"]
mod introspection;

use introspection::{
    ArchiveIntrospectionState, BoundedFindResult, EntryReport, analyze_propagation_bytes,
    find_entries_with_state_bounded, inspect_archive_bytes, inspect_entry_with_state,
    prepare_introspection_state_bytes,
};
use crushr_core::propagation::{EntryTrustClass, PropagationImpactReason};
use crushr_core::{io::{Len, ReadAt}, open::open_archive_v1, verify::verify_block_payloads_clean_v1};
use extraction_payload_core::block_raw_payload;
use index_codec::decode_index;
use recovery_classification::{RecoveryConfidence, classify_content};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function perf_now_ms() { return globalThis.performance ? globalThis.performance.now() : Date.now(); }")]
extern "C" {
    fn perf_now_ms() -> f64;
}

thread_local! {
    static LOADED_STATE: RefCell<Option<ArchiveIntrospectionState>> = const { RefCell::new(None) };
    static LOADED_BYTES: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

const MAX_FIND_RESULTS: usize = 500;
const PREVIEW_MAX_BYTES: usize = 5 * 1024;
const DERIVED_EXTENT_METADATA_BYTES: u64 = 28;

#[derive(Serialize)]
struct ArchiveLoadResponse {
    file_name: String,
    summary: introspection::ArchiveSummary,
}

#[derive(Serialize)]
struct ArchiveSummaryStageBreakdown {
    index_decode_parse_ms: f64,
    block_verification_scan_ms: f64,
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

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn archive_summary(file_name: String, archive_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
    LOADED_STATE.with(|slot| {
        *slot.borrow_mut() = None;
    });
    let summary = inspect_archive_bytes(&archive_bytes, "wasm-demo")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    LOADED_BYTES.with(|slot| {
        *slot.borrow_mut() = Some(archive_bytes);
    });
    let payload = ArchiveLoadResponse { file_name, summary };
    serde_wasm_bindgen::to_value(&payload).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn archive_summary_stage_breakdown(file_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let reader = SliceReader::new(file_bytes);
    let opened = open_archive_v1(&reader).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let decode_start = perf_now_ms();
    let _decoded = decode_index(&opened.tail.idx3_bytes)
        .map_err(|e| JsValue::from_str(&format!("decode IDX3 index: {e}")))?;
    let index_decode_parse_ms = perf_now_ms() - decode_start;

    let verify_start = perf_now_ms();
    let _clean = verify_block_payloads_clean_v1(&reader, opened.tail.footer.blocks_end_offset)
        .map_err(|e| JsValue::from_str(&format!("verify blocks: {e}")))?;
    let block_verification_scan_ms = perf_now_ms() - verify_start;

    let payload = ArchiveSummaryStageBreakdown {
        index_decode_parse_ms,
        block_verification_scan_ms,
    };
    serde_wasm_bindgen::to_value(&payload).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn reset_loaded_archive() {
    LOADED_STATE.with(|slot| {
        *slot.borrow_mut() = None;
    });
    LOADED_BYTES.with(|slot| {
        *slot.borrow_mut() = None;
    });
}


#[wasm_bindgen]
pub fn prepare_loaded_archive_state() -> Result<(), JsValue> {
    ensure_loaded_state()
}

#[wasm_bindgen]
pub fn find(file_bytes: &[u8], query: String) -> Result<JsValue, JsValue> {
    let _ = file_bytes;
    ensure_loaded_state()?;
    let matches: BoundedFindResult = LOADED_STATE
        .with(|slot| {
            slot.borrow()
                .as_ref()
                .map(|state| find_entries_with_state_bounded(state, &query, MAX_FIND_RESULTS))
        })
        .ok_or_else(|| JsValue::from_str("No archive loaded."))?;
    serde_wasm_bindgen::to_value(&matches).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn entry(file_bytes: &[u8], path: String) -> Result<JsValue, JsValue> {
    let _ = file_bytes;
    ensure_loaded_state()?;
    let detail: Option<EntryReport> = LOADED_STATE.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|state| inspect_entry_with_state(state, &path))
    });
    if detail.is_none() {
        let loaded = LOADED_STATE.with(|slot| slot.borrow().is_some());
        if !loaded {
            return Err(JsValue::from_str("No archive loaded."));
        }
    }
    serde_wasm_bindgen::to_value(&detail).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PreviewKind {
    Text,
    Binary,
    Unavailable,
}

#[derive(Serialize)]
struct EntryPreviewResponse {
    path: String,
    preview_kind: PreviewKind,
    bytes_read: usize,
    cap_bytes: usize,
    truncated: bool,
    text_preview: Option<String>,
    binary_message: Option<String>,
    note: Option<String>,
    extent_data_bytes: u64,
    extent_metadata_bytes_derived: u64,
}

#[wasm_bindgen]
pub fn entry_preview(path: String) -> Result<JsValue, JsValue> {
    let response = LOADED_BYTES.with(|slot| {
        let borrowed = slot.borrow();
        let archive_bytes = borrowed
            .as_deref()
            .ok_or_else(|| JsValue::from_str("No archive loaded."))?;
        build_entry_preview_response(archive_bytes, &path)
    })?;
    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn ensure_loaded_state() -> Result<(), JsValue> {
    if LOADED_STATE.with(|slot| slot.borrow().is_some()) {
        return Ok(());
    }
    let state = LOADED_BYTES
        .with(|slot| {
            let borrowed = slot.borrow();
            let archive_bytes = borrowed
                .as_deref()
                .ok_or_else(|| JsValue::from_str("No archive loaded."))?;
            prepare_introspection_state_bytes(archive_bytes)
                .map_err(|e| JsValue::from_str(&format!("Failed to build introspection state: {e}")))
        })?;
    LOADED_STATE.with(|slot| {
        *slot.borrow_mut() = Some(state);
    });
    Ok(())
}

fn build_entry_preview_response(
    archive_bytes: &[u8],
    path: &str,
) -> Result<EntryPreviewResponse, JsValue> {
    let reader = SliceReader::new(archive_bytes);
    let opened = open_archive_v1(&reader).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let index = decode_index(&opened.tail.idx3_bytes)
        .map_err(|e| JsValue::from_str(&format!("decode IDX3 index: {e}")))?;
    let entry = index
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .ok_or_else(|| JsValue::from_str("Selected entry is no longer available."))?;

    let extent_data_bytes = entry.extents.iter().map(|extent| extent.len).sum::<u64>();
    let extent_metadata_bytes_derived = (entry.extents.len() as u64) * DERIVED_EXTENT_METADATA_BYTES;

    if entry.extents.is_empty() || entry.size == 0 {
        return Ok(EntryPreviewResponse {
            path: path.to_string(),
            preview_kind: PreviewKind::Unavailable,
            bytes_read: 0,
            cap_bytes: PREVIEW_MAX_BYTES,
            truncated: false,
            text_preview: None,
            binary_message: None,
            note: Some("No entry payload bytes available for preview.".to_string()),
            extent_data_bytes,
            extent_metadata_bytes_derived,
        });
    }

    let blocks = crushr_core::verify::scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)
        .map_err(|e| JsValue::from_str(&format!("scan blocks: {e}")))?;
    let (preview_bytes, truncated) = read_entry_preview_bytes(&reader, entry, &blocks, PREVIEW_MAX_BYTES)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if preview_bytes.is_empty() {
        return Ok(EntryPreviewResponse {
            path: path.to_string(),
            preview_kind: PreviewKind::Unavailable,
            bytes_read: 0,
            cap_bytes: PREVIEW_MAX_BYTES,
            truncated,
            text_preview: None,
            binary_message: None,
            note: Some("Entry payload preview is empty.".to_string()),
            extent_data_bytes,
            extent_metadata_bytes_derived,
        });
    }

    if let Ok(text) = std::str::from_utf8(&preview_bytes) {
        return Ok(EntryPreviewResponse {
            path: path.to_string(),
            preview_kind: PreviewKind::Text,
            bytes_read: preview_bytes.len(),
            cap_bytes: PREVIEW_MAX_BYTES,
            truncated,
            text_preview: Some(text.to_string()),
            binary_message: None,
            note: Some("Showing UTF-8 preview from the first 5 KiB of payload bytes.".to_string()),
            extent_data_bytes,
            extent_metadata_bytes_derived,
        });
    }

    let classification = classify_content(&preview_bytes);
    let binary_message = match classification.confidence {
        RecoveryConfidence::High => format!("appears to be {}", classification.kind),
        RecoveryConfidence::Medium => format!("possibly {} (unverified)", classification.kind),
        RecoveryConfidence::Low => "unknown".to_string(),
    };

    Ok(EntryPreviewResponse {
        path: path.to_string(),
        preview_kind: PreviewKind::Binary,
        bytes_read: preview_bytes.len(),
        cap_bytes: PREVIEW_MAX_BYTES,
        truncated,
        text_preview: None,
        binary_message: Some(binary_message),
        note: Some(
            "Binary classification reuses recover-mode content classification on preview bytes only."
                .to_string(),
        ),
        extent_data_bytes,
        extent_metadata_bytes_derived,
    })
}

fn read_entry_preview_bytes<R: ReadAt>(
    reader: &R,
    entry: &format::Entry,
    blocks: &[crushr_core::verify::BlockSpanV1],
    cap_bytes: usize,
) -> anyhow::Result<(Vec<u8>, bool)> {
    let target_cap = cap_bytes.min(entry.size as usize);
    let mut out = vec![0u8; target_cap];
    let mut written = 0usize;

    for extent in &entry.extents {
        if written >= target_cap {
            break;
        }
        let block = blocks
            .get(extent.block_id as usize)
            .ok_or_else(|| anyhow::anyhow!("extent references missing block {}", extent.block_id))?;
        let raw = block_raw_payload(reader, block)?;
        let begin = extent.offset as usize;
        let end = begin.saturating_add(extent.len as usize).min(raw.len());
        if begin >= end {
            continue;
        }
        let available = end - begin;
        let remaining = target_cap - written;
        let take = available.min(remaining);
        out[written..written + take].copy_from_slice(&raw[begin..begin + take]);
        written += take;
    }
    out.truncate(written);
    let truncated = entry.size as usize > written;
    Ok((out, truncated))
}

#[derive(Serialize)]
struct PropagationViewResponse {
    impacted_entries: Vec<ImpactedEntryView>,
    no_impact_message: String,
}

#[derive(Serialize)]
struct ImpactedEntryView {
    path: String,
    impact_status: String,
    consequence: String,
    canonical_blocked: bool,
    trust_class_support: Vec<String>,
    impact_reasons: Vec<String>,
    relevant_structures: Vec<String>,
}

fn impact_reason_label(reason: &PropagationImpactReason) -> &'static str {
    match reason {
        PropagationImpactReason::CorruptedRequiredStructure => {
            "requires a corrupted archive structure"
        }
        PropagationImpactReason::CorruptedRequiredBlock => "requires a corrupted payload block",
    }
}

fn trust_class_label(trust_class: &EntryTrustClass) -> &'static str {
    match trust_class {
        EntryTrustClass::Canonical => "canonical",
        EntryTrustClass::MetadataDegraded => "metadata_degraded",
        EntryTrustClass::RecoveredNamed => "recovered_named",
        EntryTrustClass::RecoveredAnonymous => "recovered_anonymous",
        EntryTrustClass::Unrecoverable => "unrecoverable",
    }
}

fn structure_label(node: &str) -> String {
    match node {
        "structure:ftr4" => "archive footer".to_string(),
        "structure:tail_frame" => "tail frame".to_string(),
        "structure:idx3" => "index".to_string(),
        _ => {
            if let Some(block_id) = node.strip_prefix("block:") {
                format!("payload block {block_id}")
            } else {
                "archive structure".to_string()
            }
        }
    }
}

#[wasm_bindgen]
pub fn propagation(file_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let _ = file_bytes;
    let report = LOADED_BYTES
        .with(|slot| {
            let borrowed = slot.borrow();
            let archive_bytes = borrowed
                .as_deref()
                .ok_or_else(|| JsValue::from_str("No archive loaded."))?;
            analyze_propagation_bytes(archive_bytes).map_err(|e| JsValue::from_str(&e.to_string()))
        })?;
    let mut impacted_entries = report
        .entry_impacts
        .into_iter()
        .filter(|entry| !entry.activated_causes.is_empty() || entry.canonical_blocked)
        .map(|entry| {
            let mut reason_labels = entry
                .activated_causes
                .iter()
                .map(|cause| impact_reason_label(&cause.reason).to_string())
                .collect::<BTreeSet<_>>();
            if reason_labels.is_empty() && entry.canonical_blocked {
                reason_labels.insert("canonical extraction is blocked".to_string());
            }
            let mut structures = entry
                .activated_causes
                .iter()
                .map(|cause| structure_label(&cause.cause_node))
                .collect::<BTreeSet<_>>();
            if structures.is_empty() {
                structures.insert("none".to_string());
            }
            ImpactedEntryView {
                path: entry.file_path,
                impact_status: if entry.canonical_blocked {
                    "Canonical extraction blocked".to_string()
                } else {
                    "Impacted by detected corruption".to_string()
                },
                consequence: if entry.canonical_blocked {
                    "Canonical extraction is blocked for this entry.".to_string()
                } else {
                    "Entry remains viewable in this report, but propagation impact is present.".to_string()
                },
                canonical_blocked: entry.canonical_blocked,
                trust_class_support: entry
                    .supported_trust_classes
                    .iter()
                    .map(|trust| trust_class_label(trust).to_string())
                    .collect(),
                impact_reasons: reason_labels.into_iter().collect(),
                relevant_structures: structures.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();
    impacted_entries.sort_by(|a, b| a.path.cmp(&b.path));
    let payload = PropagationViewResponse {
        no_impact_message: "No impacted entries detected from current corruption inputs.".to_string(),
        impacted_entries,
    };
    serde_wasm_bindgen::to_value(&payload).map_err(|e| JsValue::from_str(&e.to_string()))
}
