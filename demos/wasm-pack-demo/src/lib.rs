// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[path = "../../../crates/crushr/src/format.rs"]
mod format;
#[allow(dead_code)]
#[path = "../../../crates/crushr/src/index_codec.rs"]
mod index_codec;

use anyhow::{Context, Result};
use crushr_format::blk3::{Blk3Flags, Blk3Header, write_blk3_header};
use crushr_format::tailframe::assemble_tail_frame;
use format::{Entry, EntryKind, Extent, Index, PreservationProfile};
use index_codec::encode_index;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const BLK3_HEADER_WITH_HASHES_LEN: u64 = (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u64;
const ZSTD_CODEC: u32 = 1;

#[derive(Deserialize)]
struct BrowserPackInput {
    path: String,
    mtime_unix_seconds: i64,
    bytes: Vec<u8>,
}

#[derive(Serialize)]
struct BrowserPackOutput {
    archive_bytes: Vec<u8>,
    file_count: usize,
    total_input_bytes: u64,
}

struct DeterministicCompressor {
    compressor: zstd::bulk::Compressor<'static>,
    output: Vec<u8>,
}

impl DeterministicCompressor {
    fn new(level: i32) -> Result<Self> {
        let mut compressor =
            zstd::bulk::Compressor::new(level).context("create zstd compressor")?;
        compressor
            .include_checksum(false)
            .context("set zstd checksum flag")?;
        compressor
            .include_contentsize(true)
            .context("set zstd content-size flag")?;
        compressor
            .include_dictid(false)
            .context("set zstd dict-id flag")?;
        Ok(Self {
            compressor,
            output: Vec::new(),
        })
    }

    fn compress(&mut self, raw: &[u8]) -> Result<&[u8]> {
        self.output.clear();
        self.output
            .reserve(zstd::zstd_safe::compress_bound(raw.len()));
        self.compressor
            .compress_to_buffer(raw, &mut self.output)
            .context("zstd compress")?;
        Ok(&self.output)
    }
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn pack_files(inputs: JsValue) -> Result<JsValue, JsValue> {
    let files: Vec<BrowserPackInput> =
        serde_wasm_bindgen::from_value(inputs).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let payload = build_archive(files).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&payload).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build_archive(files: Vec<BrowserPackInput>) -> Result<BrowserPackOutput> {
    let mut archive = Vec::<u8>::new();
    let mut write_offset = 0u64;
    let mut entries = Vec::<Entry>::with_capacity(files.len());
    let mut compressor = DeterministicCompressor::new(3)?;
    let mut total_input_bytes = 0u64;

    for (file_index, file) in files.into_iter().enumerate() {
        let raw_len = file.bytes.len() as u64;
        total_input_bytes += raw_len;
        let compressed = compressor.compress(&file.bytes)?;

        let payload_hash = *blake3::hash(compressed).as_bytes();
        let raw_hash = *blake3::hash(&file.bytes).as_bytes();
        let header = Blk3Header {
            header_len: BLK3_HEADER_WITH_HASHES_LEN as u16,
            flags: Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH),
            codec: ZSTD_CODEC,
            level: 3,
            dict_id: 0,
            raw_len,
            comp_len: compressed.len() as u64,
            payload_hash: Some(payload_hash),
            raw_hash: Some(raw_hash),
        };

        write_blk3_header(&mut archive, &header)?;
        archive.extend_from_slice(compressed);

        entries.push(Entry {
            path: file.path,
            kind: EntryKind::Regular,
            mode: 0o100644,
            mtime: file.mtime_unix_seconds,
            size: raw_len,
            extents: vec![Extent {
                block_id: file_index as u32,
                offset: 0,
                len: raw_len,
                logical_offset: 0,
            }],
            link_target: None,
            xattrs: Vec::new(),
            uid: 0,
            gid: 0,
            uname: None,
            gname: None,
            hardlink_group_id: None,
            sparse: false,
            device_major: None,
            device_minor: None,
            acl_access: None,
            acl_default: None,
            selinux_label: None,
            linux_capability: None,
        });

        write_offset += BLK3_HEADER_WITH_HASHES_LEN + compressed.len() as u64;
    }

    let index = Index {
        preservation_profile: PreservationProfile::PayloadOnly,
        entries,
    };
    let index_bytes = encode_index(&index);
    let tail_bytes = assemble_tail_frame(write_offset, None, &index_bytes, None)?;
    archive.extend_from_slice(&tail_bytes);

    Ok(BrowserPackOutput {
        archive_bytes: archive,
        file_count: index.entries.len(),
        total_input_bytes,
    })
}
