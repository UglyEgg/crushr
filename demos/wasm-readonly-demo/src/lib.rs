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
    EntryMatch, EntryReport, analyze_propagation_bytes, find_entries_bytes, inspect_archive_bytes,
    inspect_entry_bytes,
};
use crushr_core::propagation::{EntryTrustClass, PropagationImpactReason};
use serde::Serialize;
use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;

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
    let summary = inspect_archive_bytes(archive_bytes, "wasm-demo")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let payload = ArchiveLoadResponse { file_name, summary };
    serde_wasm_bindgen::to_value(&payload).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn find(file_bytes: &[u8], query: String) -> Result<JsValue, JsValue> {
    let matches: Vec<EntryMatch> =
        find_entries_bytes(file_bytes, &query, None).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&matches).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn entry(file_bytes: &[u8], path: String) -> Result<JsValue, JsValue> {
    let detail: Option<EntryReport> =
        inspect_entry_bytes(file_bytes, &path).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&detail).map_err(|e| JsValue::from_str(&e.to_string()))
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
    let report = analyze_propagation_bytes(file_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
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
