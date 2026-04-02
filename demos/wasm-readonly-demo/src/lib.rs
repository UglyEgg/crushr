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
#[path = "../../../crates/crushr/src/introspection.rs"]
mod introspection;

use introspection::{
    ArchiveIntrospectionState, BoundedFindResult, EntryReport, analyze_propagation_bytes,
    find_entries_with_state_bounded, inspect_archive_bytes, inspect_entry_with_state,
    prepare_introspection_state_bytes,
};
use crushr_core::propagation::{EntryTrustClass, PropagationImpactReason};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;

thread_local! {
    static LOADED_STATE: RefCell<Option<ArchiveIntrospectionState>> = const { RefCell::new(None) };
    static LOADED_BYTES: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

const MAX_FIND_RESULTS: usize = 500;

#[derive(Serialize)]
struct ArchiveLoadResponse {
    file_name: String,
    summary: introspection::ArchiveSummary,
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn archive_summary(file_name: String, archive_bytes: &[u8]) -> Result<JsValue, JsValue> {
    LOADED_BYTES.with(|slot| {
        *slot.borrow_mut() = Some(archive_bytes.to_vec());
    });
    LOADED_STATE.with(|slot| {
        *slot.borrow_mut() = None;
    });
    let summary = inspect_archive_bytes(archive_bytes, "wasm-demo")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let payload = ArchiveLoadResponse { file_name, summary };
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
