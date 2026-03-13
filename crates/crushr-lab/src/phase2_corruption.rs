use anyhow::{bail, Result};
use serde::Serialize;

use crate::phase2_domain::{CorruptionType, Magnitude, TargetClass, LOCKED_CORE_SEEDS};

#[derive(Debug, Clone)]
pub struct CorruptionRequest {
    pub source_archive: String,
    pub scenario_id: String,
    pub corruption_type: CorruptionType,
    pub target: TargetClass,
    pub magnitude: Magnitude,
    pub seed: u64,
    pub forced_offset: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MutationDetail {
    pub operation: String,
    pub offset: Option<u64>,
    pub before: Option<u8>,
    pub after: Option<u8>,
    pub length: Option<u64>,
    pub range_start: Option<u64>,
    pub range_end: Option<u64>,
    pub bit_index: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorruptionProvenance {
    pub source_archive: String,
    pub scenario_id: String,
    pub corruption_type: CorruptionType,
    pub target: TargetClass,
    pub magnitude: Magnitude,
    pub seed: u64,
    pub concrete_mutation_details: Vec<MutationDetail>,
}

pub fn apply_locked_corruption(
    input: &[u8],
    request: &CorruptionRequest,
) -> Result<(Vec<u8>, CorruptionProvenance)> {
    if !LOCKED_CORE_SEEDS.contains(&request.seed) {
        bail!(
            "seed {} is not in locked policy [1337, 2600, 65535]",
            request.seed
        );
    }

    let magnitude_bytes = request.magnitude.bytes() as usize;
    let (region_start, region_end) = target_region(input.len(), request.target);
    let mut output = input.to_vec();

    let details = match request.corruption_type {
        CorruptionType::BitFlip => apply_bit_flip(
            &mut output,
            region_start,
            region_end,
            magnitude_bytes,
            request.seed,
            request.forced_offset,
        ),
        CorruptionType::ByteOverwrite => apply_byte_overwrite(
            &mut output,
            region_start,
            region_end,
            magnitude_bytes,
            request.seed,
        ),
        CorruptionType::ZeroFill => apply_zero_fill(
            &mut output,
            region_start,
            region_end,
            magnitude_bytes,
            request.seed,
        ),
        CorruptionType::Truncation => apply_truncation(
            &mut output,
            region_start,
            region_end,
            magnitude_bytes,
            request.seed,
        ),
        CorruptionType::TailDamage => apply_tail_damage(
            &mut output,
            region_start,
            region_end,
            magnitude_bytes,
            request.seed,
        ),
    };

    Ok((
        output,
        CorruptionProvenance {
            source_archive: request.source_archive.clone(),
            scenario_id: request.scenario_id.clone(),
            corruption_type: request.corruption_type,
            target: request.target,
            magnitude: request.magnitude,
            seed: request.seed,
            concrete_mutation_details: details,
        },
    ))
}

fn apply_bit_flip(
    bytes: &mut [u8],
    region_start: usize,
    region_end: usize,
    magnitude_bytes: usize,
    seed: u64,
    forced_offset: Option<u64>,
) -> Vec<MutationDetail> {
    let mut details = Vec::new();
    if bytes.is_empty() || region_start >= region_end {
        return details;
    }

    if let Some(offset) = forced_offset {
        if let Ok(ix) = usize::try_from(offset) {
            if ix < bytes.len() {
                let bit_index = (seed % 8) as u8;
                let mask = 1_u8 << bit_index;
                let before = bytes[ix];
                bytes[ix] ^= mask;
                details.push(MutationDetail {
                    operation: "bit_flip".to_string(),
                    offset: Some(ix as u64),
                    before: Some(before),
                    after: Some(bytes[ix]),
                    length: Some(1),
                    range_start: Some(ix as u64),
                    range_end: Some(ix as u64 + 1),
                    bit_index: Some(bit_index),
                });
            }
        }
        return details;
    }

    let count = magnitude_bytes.min(region_end - region_start);
    let start = pick_start(region_start, region_end, count, seed);
    for i in 0..count {
        let ix = start + i;
        let bit_index = ((seed + i as u64) % 8) as u8;
        let mask = 1_u8 << bit_index;
        let before = bytes[ix];
        bytes[ix] ^= mask;
        details.push(MutationDetail {
            operation: "bit_flip".to_string(),
            offset: Some(ix as u64),
            before: Some(before),
            after: Some(bytes[ix]),
            length: Some(1),
            range_start: Some(ix as u64),
            range_end: Some(ix as u64 + 1),
            bit_index: Some(bit_index),
        });
    }
    details
}

fn apply_byte_overwrite(
    bytes: &mut [u8],
    region_start: usize,
    region_end: usize,
    magnitude_bytes: usize,
    seed: u64,
) -> Vec<MutationDetail> {
    let mut details = Vec::new();
    if bytes.is_empty() || region_start >= region_end {
        return details;
    }
    let count = magnitude_bytes.min(region_end - region_start);
    let start = pick_start(region_start, region_end, count, seed);
    for i in 0..count {
        let ix = start + i;
        let before = bytes[ix];
        let after = deterministic_byte(seed, ix as u64, i as u64);
        bytes[ix] = after;
        details.push(MutationDetail {
            operation: "byte_overwrite".to_string(),
            offset: Some(ix as u64),
            before: Some(before),
            after: Some(after),
            length: Some(1),
            range_start: Some(ix as u64),
            range_end: Some(ix as u64 + 1),
            bit_index: None,
        });
    }
    details
}

fn apply_zero_fill(
    bytes: &mut [u8],
    region_start: usize,
    region_end: usize,
    magnitude_bytes: usize,
    seed: u64,
) -> Vec<MutationDetail> {
    if bytes.is_empty() || region_start >= region_end {
        return Vec::new();
    }
    let count = magnitude_bytes.min(region_end - region_start);
    let start = pick_start(region_start, region_end, count, seed);
    let end = start + count;
    for b in &mut bytes[start..end] {
        *b = 0;
    }
    vec![MutationDetail {
        operation: "zero_fill".to_string(),
        offset: None,
        before: None,
        after: None,
        length: Some(count as u64),
        range_start: Some(start as u64),
        range_end: Some(end as u64),
        bit_index: None,
    }]
}

fn apply_truncation(
    bytes: &mut Vec<u8>,
    region_start: usize,
    region_end: usize,
    magnitude_bytes: usize,
    seed: u64,
) -> Vec<MutationDetail> {
    if bytes.is_empty() || region_start >= region_end {
        return Vec::new();
    }
    let start = pick_start(region_start, region_end, 1, seed);
    let remove_len = magnitude_bytes.min(bytes.len().saturating_sub(start));
    let new_len = bytes.len().saturating_sub(remove_len);
    bytes.truncate(new_len);
    vec![MutationDetail {
        operation: "truncation".to_string(),
        offset: None,
        before: None,
        after: None,
        length: Some(remove_len as u64),
        range_start: Some(new_len as u64),
        range_end: Some((new_len + remove_len) as u64),
        bit_index: None,
    }]
}

fn apply_tail_damage(
    bytes: &mut [u8],
    region_start: usize,
    region_end: usize,
    magnitude_bytes: usize,
    seed: u64,
) -> Vec<MutationDetail> {
    let mut details = Vec::new();
    if bytes.is_empty() || region_start >= region_end {
        return details;
    }
    let count = magnitude_bytes.min(region_end - region_start);
    let end = region_end;
    let start = end.saturating_sub(count);
    for (i, ix) in (start..end).enumerate() {
        let before = bytes[ix];
        let after = before ^ deterministic_byte(seed, ix as u64, i as u64) ^ 0xA5;
        bytes[ix] = after;
        details.push(MutationDetail {
            operation: "tail_damage".to_string(),
            offset: Some(ix as u64),
            before: Some(before),
            after: Some(after),
            length: Some(1),
            range_start: Some(ix as u64),
            range_end: Some(ix as u64 + 1),
            bit_index: None,
        });
    }
    details
}

fn target_region(len: usize, target: TargetClass) -> (usize, usize) {
    if len == 0 {
        return (0, 0);
    }
    let header_end = (len / 8).max(1);
    let index_end = (len / 2).max(header_end + 1).min(len);
    let payload_end = (len.saturating_mul(7) / 8).max(index_end + 1).min(len);
    match target {
        TargetClass::Header => (0, header_end.min(len)),
        TargetClass::Index => (header_end.min(len), index_end),
        TargetClass::Payload => (index_end, payload_end),
        TargetClass::Tail => (payload_end, len),
    }
}

fn pick_start(region_start: usize, region_end: usize, span: usize, seed: u64) -> usize {
    if region_end <= region_start || span == 0 {
        return region_start;
    }
    let available = region_end.saturating_sub(region_start);
    if span >= available {
        return region_start;
    }
    let spread = available - span + 1;
    region_start + (seed as usize % spread)
}

fn deterministic_byte(seed: u64, offset: u64, ordinal: u64) -> u8 {
    let mixed = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(offset.wrapping_mul(0x85EB_CA6B))
        .wrapping_add(ordinal.wrapping_mul(0xC2B2_AE35));
    (mixed ^ (mixed >> 16) ^ (mixed >> 32)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        (0..=255).cycle().take(16 * 1024).collect()
    }

    #[test]
    fn repeated_locked_scenario_is_byte_and_provenance_stable() {
        let input = fixture();
        let request = CorruptionRequest {
            source_archive: "archives/mixed.zip".to_string(),
            scenario_id: "p2-core-mixed-zip-byte_overwrite-index-256B-1337".to_string(),
            corruption_type: CorruptionType::ByteOverwrite,
            target: TargetClass::Index,
            magnitude: Magnitude::TwoHundredFiftySixBytes,
            seed: 1337,
            forced_offset: None,
        };

        let (a_bytes, a_prov) = apply_locked_corruption(&input, &request).expect("corrupt a");
        let (b_bytes, b_prov) = apply_locked_corruption(&input, &request).expect("corrupt b");

        assert_eq!(a_bytes, b_bytes);
        assert_eq!(
            serde_json::to_value(&a_prov).unwrap(),
            serde_json::to_value(&b_prov).unwrap()
        );
    }

    #[test]
    fn truncation_scenario_has_stable_mutation_details() {
        let input = fixture();
        let request = CorruptionRequest {
            source_archive: "archives/largefiles.tar.zst".to_string(),
            scenario_id: "p2-core-largefiles-tar_zstd-truncation-payload-4KB-65535".to_string(),
            corruption_type: CorruptionType::Truncation,
            target: TargetClass::Payload,
            magnitude: Magnitude::FourKilobytes,
            seed: 65535,
            forced_offset: None,
        };

        let (_, first) = apply_locked_corruption(&input, &request).expect("first truncation");
        let (_, second) = apply_locked_corruption(&input, &request).expect("second truncation");

        assert_eq!(
            first.concrete_mutation_details,
            second.concrete_mutation_details
        );
        assert_eq!(first.seed, 65535);
    }

    #[test]
    fn locked_seed_policy_is_enforced() {
        let input = fixture();
        let request = CorruptionRequest {
            source_archive: "archives/smallfiles.crs".to_string(),
            scenario_id: "p2-core-smallfiles-crushr-bit_flip-header-1B-42".to_string(),
            corruption_type: CorruptionType::BitFlip,
            target: TargetClass::Header,
            magnitude: Magnitude::OneByte,
            seed: 42,
            forced_offset: None,
        };
        let err = apply_locked_corruption(&input, &request).expect_err("seed should be rejected");
        assert!(err.to_string().contains("locked policy"));
    }
}
