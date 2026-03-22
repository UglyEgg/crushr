// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn scan_blk3_candidates(
    bytes: &[u8],
    verified_dict_ids: &[u32],
    dictionary_available: bool,
) -> Vec<BlockCandidate> {
    let mut candidates = Vec::new();
    let mut offset = 0usize;
    let verified_dict_set: BTreeSet<u32> = verified_dict_ids.iter().copied().collect();

    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] == BLK3_MAGIC {
            let mut candidate = BlockCandidate {
                scan_offset: offset as u64,
                mapped_block_id: None,
                structural_status: "detected",
                header_status: "invalid",
                header_reason: "header_parse_failed",
                payload_bounds_status: "unknown",
                payload_hash_status: "unavailable",
                dictionary_required: false,
                dictionary_id: None,
                dictionary_dependency_status: "not_required",
                decompression_status: "not_attempted",
                raw_hash_status: "not_attempted",
                content_verification_status: "not_content_verified",
                content_verification_reasons: vec![ReasonCode::HeaderInvalid],
                usable_for_indexed_planning: false,
                verified_raw_len: None,
            };

            if offset + 6 <= bytes.len() {
                let header_len =
                    u16::from_le_bytes([bytes[offset + 4], bytes[offset + 5]]) as usize;
                if offset + header_len <= bytes.len() {
                    if let Ok(header) =
                        read_blk3_header(Cursor::new(&bytes[offset..offset + header_len]))
                    {
                        candidate.header_status = "valid";
                        candidate.header_reason = "ok";
                        candidate.dictionary_required = header.flags.uses_dict();
                        if candidate.dictionary_required {
                            candidate.dictionary_id = Some(header.dict_id);
                            candidate.dictionary_dependency_status =
                                if verified_dict_set.contains(&header.dict_id) {
                                    "satisfied"
                                } else if dictionary_available {
                                    "missing"
                                } else {
                                    "unresolved"
                                };
                        }

                        let payload_offset = offset + header.header_len as usize;
                        if let Some(payload_end) =
                            payload_offset.checked_add(header.comp_len as usize)
                        {
                            if payload_end <= bytes.len() {
                                candidate.payload_bounds_status = "in_bounds";
                                candidate.payload_hash_status = "unavailable";
                                if let Some(expected) = header.payload_hash {
                                    let actual = blake3::hash(&bytes[payload_offset..payload_end]);
                                    candidate.payload_hash_status =
                                        if actual.as_bytes() == &expected {
                                            "verified"
                                        } else {
                                            "mismatch"
                                        };
                                }

                                if candidate.dictionary_dependency_status == "satisfied"
                                    || candidate.dictionary_dependency_status == "not_required"
                                {
                                    if header.codec != 1 {
                                        candidate.decompression_status = "unsupported_codec";
                                    } else {
                                        match zstd::decode_all(Cursor::new(
                                            &bytes[payload_offset..payload_end],
                                        )) {
                                            Ok(raw) => {
                                                candidate.decompression_status = "success";
                                                if raw.len() as u64 == header.raw_len {
                                                    candidate.verified_raw_len =
                                                        Some(raw.len() as u64);
                                                }
                                                candidate.raw_hash_status = if let Some(raw_hash) =
                                                    header.raw_hash
                                                {
                                                    if blake3::hash(&raw).as_bytes() == &raw_hash {
                                                        "verified"
                                                    } else {
                                                        "mismatch"
                                                    }
                                                } else {
                                                    "unavailable"
                                                };
                                            }
                                            Err(_) => {
                                                candidate.decompression_status = "failed";
                                                candidate.raw_hash_status =
                                                    if header.raw_hash.is_some() {
                                                        "not_attempted"
                                                    } else {
                                                        "unavailable"
                                                    };
                                            }
                                        }
                                    }
                                } else {
                                    candidate.decompression_status = "not_attempted";
                                    candidate.raw_hash_status = if header.raw_hash.is_some() {
                                        "not_attempted"
                                    } else {
                                        "unavailable"
                                    };
                                }
                            } else {
                                candidate.payload_bounds_status = "out_of_bounds";
                            }
                        } else {
                            candidate.payload_bounds_status = "out_of_bounds";
                        }

                        let mut reasons: BTreeSet<ReasonCode> = BTreeSet::new();
                        if candidate.payload_bounds_status != "in_bounds" {
                            reasons.insert(ReasonCode::PayloadOutOfBounds);
                        }
                        if candidate.payload_hash_status == "mismatch" {
                            reasons.insert(ReasonCode::PayloadHashMismatch);
                        }
                        if matches!(
                            candidate.dictionary_dependency_status,
                            "missing" | "invalid" | "unresolved"
                        ) {
                            reasons.insert(ReasonCode::DictionaryDependencyUnsatisfied);
                        }
                        if candidate.decompression_status != "success" {
                            reasons.insert(ReasonCode::DecompressionNotSuccessful);
                        }
                        if candidate.raw_hash_status == "mismatch" {
                            reasons.insert(ReasonCode::RawHashMismatch);
                        }

                        if reasons.is_empty() {
                            candidate.content_verification_status = "content_verified";
                            candidate.content_verification_reasons =
                                vec![ReasonCode::AllRequiredChecksPassed];
                        } else {
                            candidate.content_verification_reasons = reasons.into_iter().collect();
                        }

                        candidate.usable_for_indexed_planning =
                            candidate.content_verification_status == "content_verified";
                    }
                } else {
                    candidate.header_reason = "header_out_of_bounds";
                    candidate.content_verification_reasons = vec![ReasonCode::HeaderOutOfBounds];
                }
            } else {
                candidate.header_reason = "header_prefix_out_of_bounds";
                candidate.content_verification_reasons = vec![ReasonCode::HeaderPrefixOutOfBounds];
            }

            candidates.push(candidate);
        }

        offset += 1;
    }

    candidates
}

pub(super) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(super) fn build_block_verification<R: ReadAt + Len>(
    reader: &R,
    footer: &Ftr4,
    archive_bytes: &[u8],
    verified_dict_ids: &[u32],
) -> BTreeMap<u32, BlockVerification> {
    let mut out = BTreeMap::new();
    let spans = match scan_blocks_v1(reader, footer.blocks_end_offset) {
        Ok(v) => v,
        Err(_) => return out,
    };

    let candidates = scan_blk3_candidates(archive_bytes, verified_dict_ids, true)
        .into_iter()
        .map(|c| (c.scan_offset, c))
        .collect::<BTreeMap<_, _>>();

    for span in spans {
        if let Some(candidate) = candidates.get(&span.header_offset) {
            out.insert(
                span.block_id,
                BlockVerification {
                    content_verified: candidate.content_verification_status == "content_verified",
                    verified_raw_len: candidate.verified_raw_len,
                },
            );
        }
    }

    out
}

pub(super) fn classify_file(
    extents: &[Extent],
    required_block_ids: &[u32],
    block_verification: &BTreeMap<u32, BlockVerification>,
) -> (&'static str, &'static str, Vec<ReasonCode>) {
    if required_block_ids.is_empty() {
        return (
            "UNSALVAGEABLE",
            "no_required_blocks",
            vec![ReasonCode::NoRequiredBlocks],
        );
    }

    let mut failure_reasons = BTreeSet::new();

    for block_id in required_block_ids {
        let state = match block_verification.get(block_id) {
            Some(v) => v,
            None => {
                failure_reasons.insert(ReasonCode::RequiredBlockUnmapped);
                continue;
            }
        };

        if !state.content_verified {
            failure_reasons.insert(ReasonCode::RequiredBlockNotContentVerified);
        }
    }

    for extent in extents {
        if let Some(state) = block_verification.get(&extent.block_id)
            && let Some(raw_len) = state.verified_raw_len
        {
            let end = extent.offset.saturating_add(extent.len);
            if end > raw_len {
                failure_reasons.insert(ReasonCode::RequiredExtentOutOfBounds);
            }
        }
    }

    if failure_reasons.is_empty() {
        (
            "SALVAGEABLE",
            "all_required_dependencies_verified",
            Vec::new(),
        )
    } else {
        let reasons = failure_reasons.into_iter().collect::<Vec<_>>();
        ("UNSALVAGEABLE", reasons[0].as_str(), reasons)
    }
}
