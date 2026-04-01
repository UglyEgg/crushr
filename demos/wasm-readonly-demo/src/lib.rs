// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../../crates/crushr/src/format.rs"]
mod format;
#[path = "../../../crates/crushr/src/index_codec.rs"]
mod index_codec;
#[path = "../../../crates/crushr/src/extraction_payload_core.rs"]
mod extraction_payload_core;
#[path = "../../../crates/crushr/src/introspection.rs"]
mod introspection;

use introspection::{
    EntryMatch, EntryReport, find_entries_bytes, inspect_archive_bytes, inspect_entry_bytes,
};
use serde::Serialize;
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
