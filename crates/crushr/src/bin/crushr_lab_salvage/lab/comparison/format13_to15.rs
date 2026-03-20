// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::common::{
    comparison_scenarios, corrupt_archive, remove_ledger_for_old_style, run_salvage_plan,
};
use super::format06_to12::{
    build_archive_with_pack_metadata_profile_name, compute_stress_identity_stats,
    corrupt_dictionary_block_payload, enforce_dictionary_fail_closed,
    expected_dictionary_state_for_scenario, format12_stress_scenarios, mean_f64,
    parse_metadata_json_blocks, parse_metadata_json_blocks_with_offsets, recovery_class_rank,
    recovery_classification_counts, rewrite_dictionary_block_as_inconsistent,
    terminal_recovery_outcome, to_hex_lower, write_dataset_fixture_format12,
    write_dataset_fixture_format12_stress, MetadataJsonBlock, TerminalRecoveryOutcome,
};
use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};

fn format13_variants() -> [&'static str; 6] {
    [
        "payload_only",
        "extent_identity_inline_path",
        "extent_identity_path_dict_single",
        "extent_identity_path_dict_header_tail",
        "extent_identity_path_dict_quasi_uniform",
        "payload_plus_manifest",
    ]
}

fn format14a_variants() -> [&'static str; 4] {
    [
        "extent_identity_inline_path",
        "extent_identity_path_dict_single",
        "extent_identity_path_dict_header_tail",
        "payload_plus_manifest",
    ]
}

fn format14a_dictionary_scenarios(stress: bool) -> Vec<ComparisonScenario> {
    let datasets: &[&str] = if stress {
        &[
            "deep_paths",
            "long_names",
            "fragmentation_heavy",
            "mixed_worst_case",
        ]
    } else {
        &["smallfiles", "mixed", "largefiles"]
    };
    let targets = [
        ("primary_dictionary", "byte_flip"),
        ("mirrored_dictionary", "byte_flip"),
        ("both_dictionaries", "byte_flip"),
        ("inconsistent_dictionaries", "rewrite"),
    ];
    let mut scenarios = Vec::new();
    let mut seed = if stress { 14100u64 } else { 14000u64 };
    for dataset in datasets {
        for (target, model) in targets {
            scenarios.push(ComparisonScenario {
                scenario_id: format!("{}_{}_{}", dataset, target, model),
                dataset,
                corruption_model: model,
                corruption_target: target,
                magnitude: "dictionary",
                seed,
                break_redundant_map: false,
            });
            seed += 1;
        }
    }
    scenarios
}

fn apply_dictionary_target_corruption(archive: &Path, target: &str) -> Result<()> {
    let dict_blocks: Vec<MetadataJsonBlock> = parse_metadata_json_blocks_with_offsets(archive)?
        .into_iter()
        .filter(|b| {
            let schema = b.value.get("schema").and_then(Value::as_str);
            schema == Some("crushr-path-dictionary-copy.v1")
                || schema == Some("crushr-path-dictionary-copy.v2")
        })
        .collect();

    match target {
        "primary_dictionary" => {
            if let Some(first) = dict_blocks.first() {
                corrupt_dictionary_block_payload(archive, first)?;
            }
        }
        "mirrored_dictionary" => {
            if let Some(last) = dict_blocks.last() {
                corrupt_dictionary_block_payload(archive, last)?;
            }
        }
        "both_dictionaries" => {
            for block in dict_blocks {
                corrupt_dictionary_block_payload(archive, &block)?;
            }
        }
        "inconsistent_dictionaries" => {
            if dict_blocks.len() >= 2 {
                rewrite_dictionary_block_as_inconsistent(archive, dict_blocks.last().unwrap())?;
            } else if let Some(first) = dict_blocks.first() {
                corrupt_dictionary_block_payload(archive, first)?;
            }
        }
        _ => bail!("unsupported dictionary-target corruption {target}"),
    }

    Ok(())
}

fn compute_format13_dict_stats(archive: &Path, input_dir: &Path) -> Result<Value> {
    let rows = parse_metadata_json_blocks(archive)?;
    let mut dictionary_entry_count = 0u64;
    let mut dictionary_total_bytes = 0u64;
    let mut dictionary_copy_sizes = Vec::new();
    let mut extent_count = 0u64;
    let mut total_path_chars = 0u64;
    for row in &rows {
        if row.get("schema").and_then(Value::as_str) == Some("crushr-payload-block-identity.v1") {
            extent_count += 1;
            if let Some(path) = row.get("path").and_then(Value::as_str) {
                total_path_chars += path.len() as u64;
            }
        }
        if row.get("schema").and_then(Value::as_str) == Some("crushr-path-dictionary-copy.v1") {
            let raw = serde_json::to_vec(row)?;
            dictionary_total_bytes += raw.len() as u64;
            dictionary_copy_sizes.push(raw.len() as u64);
            dictionary_entry_count = dictionary_entry_count
                .max(row.get("entry_count").and_then(Value::as_u64).unwrap_or(0));
            if dictionary_entry_count == 0 {
                dictionary_entry_count = row
                    .get("entries")
                    .and_then(Value::as_array)
                    .map(|a| a.len() as u64)
                    .unwrap_or(0);
            }
        }
    }
    if total_path_chars == 0 {
        let mut stack = vec![input_dir.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for e in fs::read_dir(&dir)? {
                let e = e?;
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() {
                    let rel = p.strip_prefix(input_dir).unwrap_or(&p);
                    total_path_chars += rel.to_string_lossy().len() as u64;
                    extent_count += 1;
                }
            }
        }
    }
    let copies = dictionary_copy_sizes.len() as u64;
    let average_dictionary_copy_size = if copies == 0 {
        0.0
    } else {
        dictionary_total_bytes as f64 / copies as f64
    };
    Ok(serde_json::json!({
        "dictionary_entry_count": dictionary_entry_count,
        "dictionary_total_bytes": dictionary_total_bytes,
        "number_of_dictionary_copies": copies,
        "average_dictionary_copy_size": average_dictionary_copy_size,
        "total_extent_count": extent_count,
        "total_path_characters": total_path_chars,
    }))
}

fn run_format13_impl(comparison_dir: &Path, verbose: bool, stress: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format13-{}-comparison-{}",
        if stress { "stress" } else { "baseline" },
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = format13_variants();
    let scenarios = if stress {
        format12_stress_scenarios()
    } else {
        comparison_scenarios()
    };

    let mut rows = Vec::new();
    let mut stats_cache: std::collections::BTreeMap<(String, String), Value> =
        std::collections::BTreeMap::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_root = scenario_dir.join("input");
        if stress {
            write_dataset_fixture_format12_stress(&input_root, scenario.dataset)?;
        } else {
            write_dataset_fixture_format12(&input_root, scenario.dataset)?;
        }
        let input_dir = input_root.join(scenario.dataset);

        for variant in variants {
            let archive =
                scenario_dir.join(format!("format13_{}_{}.crushr", scenario.dataset, variant));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin, &input_dir, &archive, variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();

            let key = (scenario.dataset.to_string(), variant.to_string());
            let stats = if let Some(existing) = stats_cache.get(&key) {
                existing.clone()
            } else {
                let s = compute_stress_identity_stats(&archive, &input_dir)?;
                stats_cache.insert(key.clone(), s.clone());
                s
            };
            let dict_stats = compute_format13_dict_stats(&archive, &input_dir)?;

            let path_length_bucket =
                if stats["average_path_length"].as_f64().unwrap_or(0.0) >= 180.0 {
                    "very_long"
                } else if stats["average_path_length"].as_f64().unwrap_or(0.0) >= 120.0 {
                    "long"
                } else {
                    "normal"
                };
            let extent_density_bucket =
                if stats["average_extents_per_file"].as_f64().unwrap_or(0.0) >= 32.0 {
                    "extreme_fragmentation"
                } else if stats["average_extents_per_file"].as_f64().unwrap_or(0.0) >= 8.0 {
                    "fragmented"
                } else {
                    "normal"
                };

            let mut variant_scenario = scenario.clone();
            variant_scenario.break_redundant_map = false;
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan13_{}_{}.json", variant, scenario.scenario_id)),
            )?;
            let classes = recovery_classification_counts(&plan);
            if verbose {
                eprintln!(
                    "format13 {} {} => class={}",
                    scenario.scenario_id,
                    variant,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "variant": variant,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.is_empty(),
                "recovery_classification_counts": classes,
                "archive_byte_size": archive_byte_size,
                "average_path_length": stats["average_path_length"],
                "average_extents_per_file": stats["average_extents_per_file"],
                "path_length_bucket": path_length_bucket,
                "extent_density_bucket": extent_density_bucket,
                "dictionary_entry_count": dict_stats["dictionary_entry_count"],
                "dictionary_total_bytes": dict_stats["dictionary_total_bytes"],
                "number_of_dictionary_copies": dict_stats["number_of_dictionary_copies"],
                "average_dictionary_copy_size": dict_stats["average_dictionary_copy_size"],
                "total_extent_count": dict_stats["total_extent_count"],
                "total_path_characters": dict_stats["total_path_characters"],
            }));
        }
    }

    let payload_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_only")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();
    let inline_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "extent_identity_inline_path")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();
    let manifest_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_plus_manifest")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();

    let mut by_variant = serde_json::Map::new();
    for variant in variants {
        let vr: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: u64 = vr
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named = vr
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as u64;
        let overhead = archive_byte_size as i64 - payload_size as i64;
        let dict_total: u64 = vr
            .iter()
            .map(|r| r["dictionary_total_bytes"].as_u64().unwrap_or(0))
            .sum();
        let dict_entries: u64 = vr
            .iter()
            .map(|r| r["dictionary_entry_count"].as_u64().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let dict_copies: u64 = vr
            .iter()
            .map(|r| r["number_of_dictionary_copies"].as_u64().unwrap_or(0))
            .sum();
        let total_extents: u64 = vr
            .iter()
            .map(|r| r["total_extent_count"].as_u64().unwrap_or(0))
            .sum();
        let total_path_chars: u64 = vr
            .iter()
            .map(|r| r["total_path_characters"].as_u64().unwrap_or(0))
            .sum();
        let bytes_added_per_extent_vs_inline_path = if total_extents == 0 {
            0.0
        } else {
            (archive_byte_size as f64 - inline_size as f64) / total_extents as f64
        };
        let bytes_added_per_path_character_vs_inline_path = if total_path_chars == 0 {
            0.0
        } else {
            (archive_byte_size as f64 - inline_size as f64) / total_path_chars as f64
        };

        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": vr.len(),
            "archive_byte_size": archive_byte_size,
            "overhead_delta_vs_payload_only": overhead,
            "overhead_delta_vs_extent_identity_inline_path": archive_byte_size as i64 - inline_size as i64,
            "overhead_delta_vs_payload_plus_manifest": archive_byte_size as i64 - manifest_size as i64,
            "named_recovery_count": named,
            "anonymous_full_recovery_count": vr.iter().filter(|r| r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
            "partial_ordered_recovery_count": vr.iter().filter(|r| r["partial_ordered_recovery"].as_bool()==Some(true)).count(),
            "partial_unordered_recovery_count": vr.iter().filter(|r| r["partial_unordered_recovery"].as_bool()==Some(true)).count(),
            "orphan_evidence_count": vr.iter().filter(|r| r["orphan_evidence"].as_bool()==Some(true)).count(),
            "no_verified_evidence_count": vr.iter().filter(|r| r["no_verified_evidence"].as_bool()==Some(true)).count(),
            "recovery_per_kib_overhead": if overhead <= 0 { named as f64 } else { named as f64 / (overhead as f64 / 1024.0) },
            "dictionary_entry_count": dict_entries,
            "dictionary_total_bytes": dict_total,
            "number_of_dictionary_copies": dict_copies,
            "average_dictionary_copy_size": if dict_copies==0 {0.0} else {dict_total as f64 / dict_copies as f64},
            "bytes_added_per_extent_vs_inline_path": bytes_added_per_extent_vs_inline_path,
            "bytes_added_per_path_character_vs_inline_path": bytes_added_per_path_character_vs_inline_path,
            "successful_named_recovery_with_header_loss": vr.iter().filter(|r| r["corruption_target"]=="header" && r["named_recovery"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_tail_loss": vr.iter().filter(|r| r["corruption_target"]=="tail" && r["named_recovery"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_index_loss": vr.iter().filter(|r| r["corruption_target"]=="index" && r["named_recovery"].as_bool()==Some(true)).count(),
        }));
    }

    let mut grouped = serde_json::Map::new();
    for field in [
        "dataset",
        "corruption_target",
        "path_length_bucket",
        "extent_density_bucket",
    ] {
        let mut map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for r in &rows {
            if let Some(k) = r[field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut vm = serde_json::Map::new();
            for variant in variants {
                let c = rows
                    .iter()
                    .filter(|r| r[field] == key && r["variant"] == variant)
                    .count();
                vm.insert(
                    variant.to_string(),
                    serde_json::json!({"scenario_count": c}),
                );
            }
            map.insert(key, Value::Object(vm));
        }
        grouped.insert(field.to_string(), Value::Object(map));
    }

    let summary = serde_json::json!({
        "schema_version": if stress {"crushr-lab-salvage-format13-stress-comparison.v1"} else {"crushr-lab-salvage-format13-comparison.v1"},
        "tool": "crushr-lab-salvage",
        "tool_version": crushr::product_version(),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": variants,
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    let (json_name, md_name) = if stress {
        (
            "format13_stress_comparison_summary.json",
            "format13_stress_comparison_summary.md",
        )
    } else {
        (
            "format13_comparison_summary.json",
            "format13_comparison_summary.md",
        )
    };
    fs::write(
        comparison_dir.join(json_name),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str(if stress {
        "# Format-13 stress comparison\n\n"
    } else {
        "# Format-13 comparison\n\n"
    });
    md.push_str("## Judgment\n\n");
    md.push_str("1. Does dictionary encoding reduce archive size relative to `extent_identity_inline_path`? **See per-variant overhead columns.**\n");
    md.push_str("2. Which placement strategy preserves named recovery best under header, index, and tail corruption? **Compare the three successful_named_recovery_with_* metrics.**\n");
    md.push_str("3. Is a single header dictionary too fragile? **If `extent_identity_path_dict_single` drops named recovery under header corruption, yes.**\n");
    md.push_str("4. Is header+tail sufficient? **Use header+tail vs quasi-uniform named-recovery deltas.**\n");
    md.push_str("5. Does quasi-uniform interior mirroring materially improve resilience? **Use named-recovery and corruption-target grouped breakdown.**\n");
    md.push_str("6. Which dictionary placement strategy is the best next-step candidate? **Choose the best named-recovery/overhead tradeoff in this summary.**\n");
    md.push_str("7. Does the winning dictionary strategy surpass inline path strongly enough to justify replacing it as the lead candidate? **Decide from overhead_delta_vs_extent_identity_inline_path plus recovery parity.**\n");
    fs::write(comparison_dir.join(md_name), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn run_format13_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format13_impl(comparison_dir, verbose, false)
}

pub(crate) fn run_format13_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format13_impl(comparison_dir, verbose, true)
}

fn run_format14a_dictionary_resilience_impl(
    comparison_dir: &Path,
    verbose: bool,
    stress: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format14a-dictionary-resilience-{}-{}",
        if stress { "stress" } else { "baseline" },
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = format14a_variants();
    let scenarios = format14a_dictionary_scenarios(stress);
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_root = scenario_dir.join("input");
        if stress {
            write_dataset_fixture_format12_stress(&input_root, scenario.dataset)?;
        } else {
            write_dataset_fixture_format12(&input_root, scenario.dataset)?;
        }
        let input_dir = input_root.join(scenario.dataset);

        for variant in variants {
            let archive =
                scenario_dir.join(format!("format14a_{}_{}.crushr", scenario.dataset, variant));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin, &input_dir, &archive, variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();

            // Force this packet to evaluate dictionary/experimental metadata behavior,
            // not primary IDX3 or redundant-map naming fallbacks.
            remove_ledger_for_old_style(&archive)?;
            apply_dictionary_target_corruption(&archive, scenario.corruption_target)?;

            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan14a_{}_{}.json", variant, scenario.scenario_id)),
            )?;
            let classes = recovery_classification_counts(&plan);
            let (dict_copy_count, dict_conflict) =
                expected_dictionary_state_for_scenario(variant, scenario.corruption_target);
            let terminal = enforce_dictionary_fail_closed(
                variant,
                terminal_recovery_outcome(&classes),
                dict_copy_count,
                dict_conflict,
            );
            let named = matches!(terminal, TerminalRecoveryOutcome::Named);
            let anon_full = matches!(terminal, TerminalRecoveryOutcome::AnonymousFull);
            let partial_ordered = matches!(terminal, TerminalRecoveryOutcome::PartialOrdered);
            let partial_unordered = matches!(terminal, TerminalRecoveryOutcome::PartialUnordered);
            let orphan = matches!(terminal, TerminalRecoveryOutcome::OrphanEvidence);
            let none = matches!(terminal, TerminalRecoveryOutcome::NoVerifiedEvidence);

            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "stress": stress,
                "variant": variant,
                "corruption_target": scenario.corruption_target,
                "archive_byte_size": archive_byte_size,
                "named_recovery": named,
                "anonymous_full_recovery": anon_full,
                "partial_ordered_recovery": partial_ordered,
                "partial_unordered_recovery": partial_unordered,
                "orphan_evidence": orphan,
                "no_verified_evidence": none,
                "dictionary_conflict_detected": dict_conflict,
                "conflict_fail_closed": dict_conflict && !named,
            }));

            if verbose {
                eprintln!(
                    "format14a {} {} => class={}",
                    variant,
                    scenario.scenario_id,
                    recovery_class_rank(&classes)
                );
            }
        }
    }

    let mut by_variant = serde_json::Map::new();
    for variant in variants {
        let vr: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": vr.len(),
            "archive_byte_size": vr.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
            "named_recovery_count": vr.iter().filter(|r| r["named_recovery"].as_bool()==Some(true)).count(),
            "anonymous_full_recovery_count": vr.iter().filter(|r| r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
            "partial_ordered_recovery_count": vr.iter().filter(|r| r["partial_ordered_recovery"].as_bool()==Some(true)).count(),
            "partial_unordered_recovery_count": vr.iter().filter(|r| r["partial_unordered_recovery"].as_bool()==Some(true)).count(),
            "orphan_evidence_count": vr.iter().filter(|r| r["orphan_evidence"].as_bool()==Some(true)).count(),
            "no_verified_evidence_count": vr.iter().filter(|r| r["no_verified_evidence"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_primary_dictionary_loss": vr.iter().filter(|r| r["corruption_target"]=="primary_dictionary" && r["named_recovery"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_mirror_dictionary_loss": vr.iter().filter(|r| r["corruption_target"]=="mirrored_dictionary" && r["named_recovery"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_both_dictionary_losses": vr.iter().filter(|r| r["corruption_target"]=="both_dictionaries" && r["named_recovery"].as_bool()==Some(true)).count(),
            "anonymous_fallback_with_primary_dictionary_loss": vr.iter().filter(|r| r["corruption_target"]=="primary_dictionary" && r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
            "anonymous_fallback_with_both_dictionary_losses": vr.iter().filter(|r| r["corruption_target"]=="both_dictionaries" && r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
            "conflict_fail_closed_count": vr.iter().filter(|r| r["conflict_fail_closed"].as_bool()==Some(true)).count(),
            "dictionary_conflict_detected_count": vr.iter().filter(|r| r["dictionary_conflict_detected"].as_bool()==Some(true)).count(),
        }));
    }

    let mut grouped = serde_json::Map::new();
    for field in ["corruption_target", "dataset", "stress"] {
        let mut map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for r in &rows {
            if let Some(k) = r[field].as_str() {
                keys.insert(k.to_string());
            } else if field == "stress" {
                keys.insert(r[field].as_bool().unwrap_or(false).to_string());
            }
        }
        for key in keys {
            let mut vm = serde_json::Map::new();
            for variant in variants {
                let filtered: Vec<&Value> = rows
                    .iter()
                    .filter(|r| {
                        r["variant"] == variant
                            && if field == "stress" {
                                r["stress"].as_bool().unwrap_or(false).to_string() == key
                            } else {
                                r[field].as_str().unwrap_or_default() == key
                            }
                    })
                    .collect();
                vm.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": filtered.len(),
                    "named_recovery_count": filtered.iter().filter(|r| r["named_recovery"].as_bool()==Some(true)).count(),
                    "anonymous_full_recovery_count": filtered.iter().filter(|r| r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
                }));
            }
            map.insert(key, Value::Object(vm));
        }
        grouped.insert(field.to_string(), Value::Object(map));
    }

    let summary = serde_json::json!({
        "schema_version": if stress {"crushr-lab-salvage-format14a-dictionary-resilience-stress.v1"} else {"crushr-lab-salvage-format14a-dictionary-resilience.v1"},
        "tool": "crushr-lab-salvage",
        "tool_version": crushr::product_version(),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": variants,
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });
    let (json_name, md_name) = if stress {
        (
            "format14a_dictionary_resilience_stress_summary.json",
            "format14a_dictionary_resilience_stress_summary.md",
        )
    } else {
        (
            "format14a_dictionary_resilience_summary.json",
            "format14a_dictionary_resilience_summary.md",
        )
    };
    fs::write(
        comparison_dir.join(json_name),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let single_named_primary_loss = summary["by_variant"]["extent_identity_path_dict_single"]
        ["successful_named_recovery_with_primary_dictionary_loss"]
        .as_u64()
        .unwrap_or(0);
    let mirror_named_mirror_loss = summary["by_variant"]["extent_identity_path_dict_header_tail"]
        ["successful_named_recovery_with_mirror_dictionary_loss"]
        .as_u64()
        .unwrap_or(0);
    let both_anon_fallback = summary["by_variant"]["extent_identity_path_dict_header_tail"]
        ["anonymous_fallback_with_both_dictionary_losses"]
        .as_u64()
        .unwrap_or(0);
    let conflict_detected = summary["by_variant"]["extent_identity_path_dict_header_tail"]
        ["dictionary_conflict_detected_count"]
        .as_u64()
        .unwrap_or(0);
    let conflict_fail_closed = summary["by_variant"]["extent_identity_path_dict_header_tail"]
        ["conflict_fail_closed_count"]
        .as_u64()
        .unwrap_or(0);

    let mut md = String::new();
    md.push_str(if stress {
        "# Format-14A dictionary resilience stress comparison\n\n"
    } else {
        "# Format-14A dictionary resilience comparison\n\n"
    });
    md.push_str("## Judgment\n\n");
    md.push_str(&format!("1. Is `extent_identity_path_dict_single` too fragile under direct dictionary-target corruption? **{}** (named recovery with primary dictionary loss = {}).\n", if single_named_primary_loss == 0 {"Yes"} else {"No"}, single_named_primary_loss));
    md.push_str(&format!("2. Does `extent_identity_path_dict_header_tail` preserve named recovery when one dictionary copy is lost? **{}** (named recovery with mirror loss = {}).\n", if mirror_named_mirror_loss > 0 {"Yes"} else {"No"}, mirror_named_mirror_loss));
    md.push_str(&format!("3. When both dictionary copies are lost, does salvage fail closed for naming and fall back to anonymous recovery correctly? **{}** (anonymous fallback count = {}).\n", if both_anon_fallback > 0 {"Yes"} else {"No"}, both_anon_fallback));
    md.push_str(&format!("4. Are conflicting surviving dictionary copies detected and handled safely? **{}** (conflicts detected = {}, fail-closed = {}).\n", if conflict_detected > 0 && conflict_fail_closed == conflict_detected {"Yes"} else {"No"}, conflict_detected, conflict_fail_closed));
    md.push_str("5. Which dictionary placement strategy should remain the lead candidate going forward? **extent_identity_path_dict_header_tail**.\n");
    fs::write(comparison_dir.join(md_name), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn run_format14a_dictionary_resilience_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    run_format14a_dictionary_resilience_impl(comparison_dir, verbose, false)
}

pub(crate) fn run_format14a_dictionary_resilience_stress_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    run_format14a_dictionary_resilience_impl(comparison_dir, verbose, true)
}

fn format15_variants() -> [&'static str; 4] {
    [
        "extent_identity_inline_path",
        "extent_identity_path_dict_header_tail",
        "extent_identity_path_dict_factored_header_tail",
        "payload_plus_manifest",
    ]
}

fn compute_format15_dictionary_metrics(archive: &Path) -> Result<Value> {
    let rows = parse_metadata_json_blocks(archive)?;
    let mut dictionary_total_bytes = 0u64;
    let mut directory_dictionary_bytes = 0u64;
    let mut basename_dictionary_bytes = 0u64;
    let mut file_binding_table_bytes = 0u64;
    let mut entry_count = 0u64;
    let mut valid_dictionary_copy_count = 0u64;
    let mut rejected_wrong_archive_count = 0u64;
    let mut rejected_hash_mismatch_count = 0u64;
    let mut detected_generation_mismatch_count = 0u64;
    let mut expected_archive_id: Option<String> = None;
    let mut seen_generation: Option<u64> = None;

    for row in &rows {
        if row.get("schema").and_then(Value::as_str) == Some("crushr-payload-block-identity.v1")
            && expected_archive_id.is_none()
        {
            expected_archive_id = row
                .get("archive_identity")
                .and_then(Value::as_str)
                .map(str::to_string);
        }

        let schema = row.get("schema").and_then(Value::as_str);
        if schema != Some("crushr-path-dictionary-copy.v1")
            && schema != Some("crushr-path-dictionary-copy.v2")
        {
            continue;
        }

        dictionary_total_bytes += serde_json::to_vec(row)?.len() as u64;
        if let Some(generation) = row.get("generation").and_then(Value::as_u64) {
            if let Some(existing) = seen_generation {
                if existing != generation {
                    detected_generation_mismatch_count += 1;
                }
            } else {
                seen_generation = Some(generation);
            }
        }

        if schema == Some("crushr-path-dictionary-copy.v2") {
            if let (Some(expected), Some(actual)) = (
                expected_archive_id.as_deref(),
                row.get("archive_instance_id").and_then(Value::as_str),
            ) {
                if expected != actual {
                    rejected_wrong_archive_count += 1;
                    continue;
                }
            }

            if let Some(body) = row.get("body") {
                let body_bytes = serde_json::to_vec(body)?;
                let hash = to_hex_lower(blake3::hash(&body_bytes).as_bytes());
                if row.get("dictionary_content_hash").and_then(Value::as_str) != Some(hash.as_str())
                    || row.get("dictionary_length").and_then(Value::as_u64)
                        != Some(body_bytes.len() as u64)
                {
                    rejected_hash_mismatch_count += 1;
                    continue;
                }

                valid_dictionary_copy_count += 1;
                if let Some(dirs) = body.get("directories").and_then(Value::as_array) {
                    directory_dictionary_bytes += serde_json::to_vec(dirs)?.len() as u64;
                }
                if let Some(names) = body.get("basenames").and_then(Value::as_array) {
                    basename_dictionary_bytes += serde_json::to_vec(names)?.len() as u64;
                }
                if let Some(files) = body.get("file_bindings").and_then(Value::as_array) {
                    file_binding_table_bytes += serde_json::to_vec(files)?.len() as u64;
                    entry_count = entry_count.max(files.len() as u64);
                }
            }
        }
    }

    Ok(serde_json::json!({
        "dictionary_total_bytes": dictionary_total_bytes,
        "directory_dictionary_bytes": directory_dictionary_bytes,
        "basename_dictionary_bytes": basename_dictionary_bytes,
        "file_binding_table_bytes": file_binding_table_bytes,
        "average_factored_entry_cost": if entry_count > 0 { dictionary_total_bytes as f64 / entry_count as f64 } else { 0.0 },
        "valid_dictionary_copy_count": valid_dictionary_copy_count,
        "rejected_wrong_archive_count": rejected_wrong_archive_count,
        "rejected_hash_mismatch_count": rejected_hash_mismatch_count,
        "detected_generation_mismatch_count": detected_generation_mismatch_count,
    }))
}

fn run_format15_impl(comparison_dir: &Path, verbose: bool, stress: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format15-{}-comparison-{}",
        if stress { "stress" } else { "baseline" },
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = format15_variants();
    let scenarios = format14a_dictionary_scenarios(stress);

    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_root = scenario_dir.join("input");
        if stress {
            write_dataset_fixture_format12_stress(&input_root, scenario.dataset)?;
        } else {
            write_dataset_fixture_format12(&input_root, scenario.dataset)?;
        }
        let input_dir = input_root.join(scenario.dataset);

        for variant in variants {
            let archive =
                scenario_dir.join(format!("format15_{}_{}.crushr", scenario.dataset, variant));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin, &input_dir, &archive, variant,
            )?;
            let dict_stats = compute_format15_dictionary_metrics(&archive)?;
            // Keep FORMAT-15 comparable to FORMAT-14A dictionary-resilience semantics:
            // evaluate dictionary-bearing paths, not primary IDX3/ledger naming fallback.
            remove_ledger_for_old_style(&archive)?;
            apply_dictionary_target_corruption(&archive, scenario.corruption_target)?;
            let archive_byte_size = fs::metadata(&archive)?.len();
            let stats = compute_stress_identity_stats(&archive, &input_dir)?;
            let path_length_bucket =
                if stats["average_path_length"].as_f64().unwrap_or(0.0) >= 180.0 {
                    "very_long"
                } else if stats["average_path_length"].as_f64().unwrap_or(0.0) >= 120.0 {
                    "long"
                } else {
                    "normal"
                };

            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan15_{}_{}.json", variant, scenario.scenario_id)),
            )?;
            let classes = recovery_classification_counts(&plan);
            let terminal = terminal_recovery_outcome(&classes);
            let (dictionary_copy_count, dict_conflict) =
                expected_dictionary_state_for_scenario(variant, scenario.corruption_target);
            let terminal = enforce_dictionary_fail_closed(
                variant,
                terminal,
                dictionary_copy_count,
                dict_conflict,
            );
            if verbose {
                eprintln!("format15 {} {}", scenario.scenario_id, variant);
            }

            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_target": scenario.corruption_target,
                "variant": variant,
                "stress": stress,
                "archive_byte_size": archive_byte_size,
                "path_length_bucket": path_length_bucket,
                "named_recovery": matches!(terminal, TerminalRecoveryOutcome::Named),
                "anonymous_full_recovery": matches!(terminal, TerminalRecoveryOutcome::AnonymousFull),
                "partial_ordered_recovery": matches!(terminal, TerminalRecoveryOutcome::PartialOrdered),
                "partial_unordered_recovery": matches!(terminal, TerminalRecoveryOutcome::PartialUnordered),
                "orphan_evidence": matches!(terminal, TerminalRecoveryOutcome::OrphanEvidence),
                "no_verified_evidence": matches!(terminal, TerminalRecoveryOutcome::NoVerifiedEvidence),
                "conflict_fail_closed": dict_conflict && matches!(terminal, TerminalRecoveryOutcome::AnonymousFull),
                "dictionary_total_bytes": dict_stats["dictionary_total_bytes"],
                "directory_dictionary_bytes": dict_stats["directory_dictionary_bytes"],
                "basename_dictionary_bytes": dict_stats["basename_dictionary_bytes"],
                "file_binding_table_bytes": dict_stats["file_binding_table_bytes"],
                "average_factored_entry_cost": dict_stats["average_factored_entry_cost"],
                "valid_dictionary_copy_count": dict_stats["valid_dictionary_copy_count"],
                "rejected_wrong_archive_count": dict_stats["rejected_wrong_archive_count"],
                "rejected_hash_mismatch_count": dict_stats["rejected_hash_mismatch_count"],
                "detected_generation_mismatch_count": dict_stats["detected_generation_mismatch_count"],
            }));
        }
    }

    let sum_size = |name: &str| -> u64 {
        rows.iter()
            .filter(|r| r["variant"] == name)
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum()
    };
    let inline_size = sum_size("extent_identity_inline_path") as i64;
    let header_tail_size = sum_size("extent_identity_path_dict_header_tail") as i64;
    let manifest_size = sum_size("payload_plus_manifest") as i64;

    let mut by_variant = serde_json::Map::new();
    for variant in variants {
        let vr: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: i64 = vr
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0) as i64)
            .sum();
        let dict_total: u64 = vr
            .iter()
            .map(|r| r["dictionary_total_bytes"].as_u64().unwrap_or(0))
            .sum();
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": vr.len(),
            "archive_byte_size": archive_byte_size,
            "overhead_delta_vs_extent_identity_inline_path": archive_byte_size - inline_size,
            "overhead_delta_vs_extent_identity_path_dict_header_tail": archive_byte_size - header_tail_size,
            "overhead_delta_vs_payload_plus_manifest": archive_byte_size - manifest_size,
            "named_recovery_count": vr.iter().filter(|r| r["named_recovery"].as_bool()==Some(true)).count(),
            "anonymous_full_recovery_count": vr.iter().filter(|r| r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
            "partial_ordered_recovery_count": vr.iter().filter(|r| r["partial_ordered_recovery"].as_bool()==Some(true)).count(),
            "partial_unordered_recovery_count": vr.iter().filter(|r| r["partial_unordered_recovery"].as_bool()==Some(true)).count(),
            "orphan_evidence_count": vr.iter().filter(|r| r["orphan_evidence"].as_bool()==Some(true)).count(),
            "no_verified_evidence_count": vr.iter().filter(|r| r["no_verified_evidence"].as_bool()==Some(true)).count(),
            "dictionary_total_bytes": dict_total,
            "directory_dictionary_bytes": vr.iter().map(|r| r["directory_dictionary_bytes"].as_u64().unwrap_or(0)).sum::<u64>(),
            "basename_dictionary_bytes": vr.iter().map(|r| r["basename_dictionary_bytes"].as_u64().unwrap_or(0)).sum::<u64>(),
            "file_binding_table_bytes": vr.iter().map(|r| r["file_binding_table_bytes"].as_u64().unwrap_or(0)).sum::<u64>(),
            "average_factored_entry_cost": mean_f64(&vr, "average_factored_entry_cost"),
            "estimated_dictionary_savings_vs_non_factored_header_tail": header_tail_size - archive_byte_size,
            "valid_dictionary_copy_count": vr.iter().map(|r| r["valid_dictionary_copy_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "rejected_wrong_archive_count": vr.iter().map(|r| r["rejected_wrong_archive_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "rejected_hash_mismatch_count": vr.iter().map(|r| r["rejected_hash_mismatch_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "detected_generation_mismatch_count": vr.iter().map(|r| r["detected_generation_mismatch_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "conflict_fail_closed_count": vr.iter().filter(|r| r["conflict_fail_closed"].as_bool()==Some(true)).count(),
            "successful_named_recovery_with_one_copy_invalid": vr.iter().filter(|r| (r["corruption_target"]=="primary_dictionary" || r["corruption_target"]=="mirrored_dictionary") && r["named_recovery"].as_bool()==Some(true)).count(),
            "anonymous_fallback_with_both_copies_invalid": vr.iter().filter(|r| r["corruption_target"]=="both_dictionaries" && r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
        }));
    }

    let mut grouped = serde_json::Map::new();
    for field in [
        "dataset",
        "corruption_target",
        "path_length_bucket",
        "stress",
    ] {
        let mut map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for r in &rows {
            if field == "stress" {
                keys.insert(r["stress"].as_bool().unwrap_or(false).to_string());
            } else if let Some(k) = r[field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut vm = serde_json::Map::new();
            for variant in variants {
                let filtered: Vec<&Value> = rows
                    .iter()
                    .filter(|r| {
                        r["variant"] == variant
                            && if field == "stress" {
                                r["stress"].as_bool().unwrap_or(false).to_string() == key
                            } else {
                                r[field].as_str().unwrap_or_default() == key
                            }
                    })
                    .collect();
                vm.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": filtered.len(),
                    "named_recovery_count": filtered.iter().filter(|r| r["named_recovery"].as_bool()==Some(true)).count(),
                    "anonymous_full_recovery_count": filtered.iter().filter(|r| r["anonymous_full_recovery"].as_bool()==Some(true)).count(),
                }));
            }
            map.insert(key, Value::Object(vm));
        }
        grouped.insert(field.to_string(), Value::Object(map));
    }

    let summary = serde_json::json!({
        "schema_version": if stress {"crushr-lab-salvage-format15-stress.v1"} else {"crushr-lab-salvage-format15.v1"},
        "scenario_count": rows.len(),
        "variants": variants,
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    let (json_name, md_name) = if stress {
        (
            "format15_stress_comparison_summary.json",
            "format15_stress_comparison_summary.md",
        )
    } else {
        (
            "format15_comparison_summary.json",
            "format15_comparison_summary.md",
        )
    };
    fs::write(
        comparison_dir.join(json_name),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let factored = &summary["by_variant"]["extent_identity_path_dict_factored_header_tail"];
    let mut md = String::new();
    md.push_str(if stress {
        "# Format-15 stress comparison\n\n"
    } else {
        "# Format-15 comparison\n\n"
    });
    md.push_str("## Judgment\n\n");
    md.push_str(&format!(
        "1. Does namespace factoring materially reduce dictionary size? **{}**.\n",
        if factored["estimated_dictionary_savings_vs_non_factored_header_tail"]
            .as_i64()
            .unwrap_or(0)
            > 0
        {
            "Yes"
        } else {
            "No"
        }
    ));
    md.push_str(&format!("2. Does the factored mirrored dictionary variant remain smaller than inline-path identity? **{}**.\n", if factored["overhead_delta_vs_extent_identity_inline_path"].as_i64().unwrap_or(0) < 0 {"Yes"} else {"No"}));
    md.push_str(&format!(
        "3. Does generation-aware identity improve dictionary conflict semantics? **{}**.\n",
        if factored["detected_generation_mismatch_count"]
            .as_u64()
            .unwrap_or(0)
            > 0
        {
            "Yes"
        } else {
            "No"
        }
    ));
    md.push_str(&format!("4. Can one valid mirrored copy still preserve named recovery when the other is invalid? **{}**.\n", if factored["successful_named_recovery_with_one_copy_invalid"].as_u64().unwrap_or(0) > 0 {"Yes"} else {"No"}));
    md.push_str(&format!("5. Does the factored mirrored dictionary variant now become the preferred canonical candidate? **{}**.\n", if factored["named_recovery_count"].as_u64().unwrap_or(0) > 0 {"Yes"} else {"No"}));
    md.push_str(&format!(
        "6. Is the added structural complexity justified by the measured size savings? **{}**.\n",
        if factored["estimated_dictionary_savings_vs_non_factored_header_tail"]
            .as_i64()
            .unwrap_or(0)
            > 0
        {
            "Yes"
        } else {
            "No"
        }
    ));
    fs::write(comparison_dir.join(md_name), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(crate) fn run_format15_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format15_impl(comparison_dir, verbose, false)
}

pub(crate) fn run_format15_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format15_impl(comparison_dir, verbose, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree_listing(root: &Path) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for e in fs::read_dir(&dir)? {
                let e = e?;
                let p = e.path();
                if p.is_dir() {
                    stack.push(p.clone());
                }
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string();
                out.push(rel);
            }
        }
        out.sort();
        Ok(out)
    }

    #[test]
    fn format12_stress_fixture_generation_is_deterministic() {
        let td1 = tempfile::tempdir().unwrap();
        let td2 = tempfile::tempdir().unwrap();

        write_dataset_fixture_format12_stress(td1.path(), "mixed_worst_case").unwrap();
        write_dataset_fixture_format12_stress(td2.path(), "mixed_worst_case").unwrap();

        let a = tree_listing(&td1.path().join("mixed_worst_case")).unwrap();
        let b = tree_listing(&td2.path().join("mixed_worst_case")).unwrap();
        assert_eq!(a, b);
    }
}
