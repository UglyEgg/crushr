// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};

pub(crate) fn comparison_scenarios() -> Vec<ComparisonScenario> {
    let datasets = ["smallfiles", "mixed", "largefiles"];
    let targets = [
        ("header", "byte_flip"),
        ("index", "byte_flip"),
        ("payload", "byte_flip"),
        ("tail", "truncate"),
    ];
    let magnitudes = ["small", "medium"];

    let mut scenarios = Vec::new();
    let mut seed = 100u64;
    for dataset in datasets {
        for (target, model) in targets {
            for magnitude in magnitudes {
                let break_redundant_map =
                    dataset == "mixed" && target == "index" && magnitude == "medium";
                scenarios.push(ComparisonScenario {
                    scenario_id: format!("{}_{}_{}_{}", dataset, target, model, magnitude),
                    dataset,
                    corruption_model: model,
                    corruption_target: target,
                    magnitude,
                    seed,
                    break_redundant_map,
                });
                seed += 1;
            }
        }
    }
    scenarios
}

pub(crate) fn write_dataset_fixture(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input).with_context(|| format!("create {}", input.display()))?;

    match dataset {
        "smallfiles" => {
            fs::write(input.join("tiny.txt"), b"small-dataset-payload")?;
        }
        "mixed" => {
            fs::write(
                input.join("mixed.bin"),
                (0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
            )?;
        }
        "largefiles" => {
            fs::write(input.join("large.dat"), vec![13u8; 8192])?;
        }
        _ => bail!("unsupported dataset {dataset}"),
    }

    Ok(())
}

pub(crate) fn build_archive_with_pack(pack_bin: &Path, input: &Path, output: &Path) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn build_archive_with_pack_experimental(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--experimental-self-describing-extents")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn build_archive_with_pack_file_identity(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--experimental-file-identity-extents")
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn build_archive_with_pack_format05(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg(FORMAT05_PACK_FLAG)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn build_archive_with_pack_format08(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
    strategy: &str,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg(FORMAT05_PACK_FLAG)
        .arg(FORMAT06_PACK_FLAG)
        .arg("--placement-strategy")
        .arg(strategy)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(crate) fn remove_ledger_for_old_style(archive_path: &Path) -> Result<()> {
    let bytes = fs::read(archive_path)?;
    let footer_offset = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let tail = parse_tail_frame(&bytes[blocks_end..])?;
    let mut rewritten = bytes[..blocks_end].to_vec();
    let tail_bytes = assemble_tail_frame(footer.blocks_end_offset, None, &tail.idx3_bytes, None)?;
    rewritten.extend_from_slice(&tail_bytes);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

pub(crate) fn corrupt_archive(archive_path: &Path, scenario: &ComparisonScenario) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    let len = bytes.len();
    let shift = (scenario.seed as usize) % 11;
    let mag = if scenario.magnitude == "small" {
        1usize
    } else {
        8usize
    };

    match scenario.corruption_target {
        "header" => {
            for (i, byte) in bytes.iter_mut().enumerate().take(mag.min(len)) {
                *byte ^= 0x11 + ((i + shift) % 7) as u8;
            }
        }
        "payload" => {
            let start = (len / 3).saturating_add(shift);
            for i in 0..mag {
                let idx = (start + i).min(len.saturating_sub(1));
                bytes[idx] ^= 0x33 + (i % 5) as u8;
            }
        }
        "tail" => {
            let cut = if scenario.magnitude == "small" {
                32
            } else {
                128
            };
            let new_len = len.saturating_sub(cut);
            bytes.truncate(new_len.max(16));
        }
        "index" => {
            let footer_offset = len.saturating_sub(FTR4_LEN);
            let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
            if scenario.dataset == "smallfiles" {
                let blocks_end = footer.blocks_end_offset as usize;
                let mut tail = parse_tail_frame(&bytes[blocks_end..])?;
                if tail.idx3_bytes.len() > 4 {
                    tail.idx3_bytes[4] ^= 0x7F;
                }
                for i in 0..mag {
                    let idx = 5 + ((i + shift) % 16);
                    if idx < tail.idx3_bytes.len() {
                        tail.idx3_bytes[idx] ^= 0x5A + (i % 3) as u8;
                    }
                }
                let mut rewritten = bytes[..blocks_end].to_vec();
                let tail_bytes = assemble_tail_frame(
                    footer.blocks_end_offset,
                    None,
                    &tail.idx3_bytes,
                    tail.ldg1.as_ref(),
                )?;
                rewritten.extend_from_slice(&tail_bytes);
                fs::write(archive_path, rewritten)?;
                return Ok(());
            }

            let index_offset = footer.index_offset as usize;
            if index_offset < footer_offset {
                bytes[index_offset] ^= 0x7F;
            }
            for i in 0..mag {
                let idx = index_offset + 4 + ((i + shift) % 16);
                if idx < footer_offset {
                    bytes[idx] ^= 0x5A + (i % 3) as u8;
                }
            }
        }
        _ => bail!(
            "unsupported corruption target {}",
            scenario.corruption_target
        ),
    }

    fs::write(archive_path, bytes)?;
    Ok(())
}

pub(crate) fn damage_redundant_map_ledger(archive_path: &Path) -> Result<()> {
    let bytes = fs::read(archive_path)?;
    let footer_offset = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let tail = parse_tail_frame(&bytes[blocks_end..])?;
    let Some(mut ledger) = tail.ldg1 else {
        return Ok(());
    };

    let mut value: Value = serde_json::from_slice(&ledger.json)?;
    if let Some(files) = value.get_mut("files").and_then(Value::as_array_mut) {
        if let Some(first) = files.first_mut() {
            if let Some(extents) = first.get_mut("extents").and_then(Value::as_array_mut) {
                if let Some(ext) = extents.first_mut() {
                    ext["block_id"] = Value::from(999999u64);
                }
            }
        }
    }
    ledger.json = serde_json::to_vec(&value)?;

    let mut rewritten = bytes[..blocks_end].to_vec();
    let tail_bytes = assemble_tail_frame(
        footer.blocks_end_offset,
        None,
        &tail.idx3_bytes,
        Some(&ledger),
    )?;
    rewritten.extend_from_slice(&tail_bytes);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

pub(crate) fn run_salvage_plan(
    salvage_bin: &Path,
    archive_path: &Path,
    plan_path: &Path,
) -> Result<Value> {
    let out = Command::new(salvage_bin)
        .arg(archive_path)
        .arg("--json-out")
        .arg(plan_path)
        .output()
        .with_context(|| format!("run {:?}", salvage_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-salvage failed
stdout:
{}
stderr:
{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(serde_json::from_slice(&fs::read(plan_path)?)?)
}

pub(crate) fn outcome_from_plan(plan: &Value) -> OutcomeMetrics {
    let verified_block_count = plan
        .get("block_candidates")
        .and_then(Value::as_array)
        .map_or(0, |rows| {
            rows.iter()
                .filter(|r| {
                    r.get("content_verification_status").and_then(Value::as_str)
                        == Some("content_verified")
                })
                .count() as u64
        });
    let salvageable_file_count = plan["summary"]["salvageable_files"].as_u64().unwrap_or(0);
    let exported_full_file_count =
        plan.get("file_plans")
            .and_then(Value::as_array)
            .map_or(0, |rows| {
                rows.iter()
                    .filter(|r| {
                        r.get("salvage_status").and_then(Value::as_str) == Some("fully_salvageable")
                    })
                    .count() as u64
            });
    let outcome = if exported_full_file_count > 0 {
        "FULL_FILE_SALVAGE_AVAILABLE"
    } else if salvageable_file_count > 0 {
        "PARTIAL_FILE_SALVAGE"
    } else if verified_block_count > 0 {
        "ORPHAN_EVIDENCE_ONLY"
    } else {
        "NO_VERIFIED_EVIDENCE"
    };

    OutcomeMetrics {
        outcome: outcome.to_string(),
        verified_block_count,
        salvageable_file_count,
        exported_full_file_count,
    }
}

pub(crate) fn outcome_rank(value: &str) -> i32 {
    match value {
        "NO_VERIFIED_EVIDENCE" => 0,
        "ORPHAN_EVIDENCE_ONLY" => 1,
        "PARTIAL_FILE_SALVAGE" => 2,
        "FULL_FILE_SALVAGE_AVAILABLE" => 3,
        _ => -1,
    }
}

pub(crate) fn classify_improvement(old: &str, new: &str) -> String {
    match (old, new) {
        ("ORPHAN_EVIDENCE_ONLY", "PARTIAL_FILE_SALVAGE") => {
            "IMPROVED_ORPHAN_TO_PARTIAL".to_string()
        }
        ("ORPHAN_EVIDENCE_ONLY", "FULL_FILE_SALVAGE_AVAILABLE") => {
            "IMPROVED_ORPHAN_TO_FULL".to_string()
        }
        ("NO_VERIFIED_EVIDENCE", "PARTIAL_FILE_SALVAGE") => "IMPROVED_NONE_TO_PARTIAL".to_string(),
        ("NO_VERIFIED_EVIDENCE", "FULL_FILE_SALVAGE_AVAILABLE") => {
            "IMPROVED_NONE_TO_FULL".to_string()
        }
        _ if outcome_rank(new) > outcome_rank(old) => "IMPROVED_OTHER".to_string(),
        _ if outcome_rank(new) < outcome_rank(old) => "DEGRADED".to_string(),
        _ => "UNCHANGED".to_string(),
    }
}

pub(crate) fn build_groups(
    rows: &[ScenarioRow],
    key_fn: impl Fn(&ScenarioRow) -> &str,
) -> Vec<ComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&ScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| ComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            new_outcome_counts: count_outcomes(values.iter().map(|r| r.new_outcome.as_str())),
            verified_block_delta: values
                .iter()
                .map(|r| r.new_verified_block_count as i64 - r.old_verified_block_count as i64)
                .sum(),
            salvageable_file_delta: values
                .iter()
                .map(|r| r.new_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
                .sum(),
            exported_full_file_delta: values
                .iter()
                .map(|r| {
                    r.new_exported_full_file_count as i64 - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

pub(crate) fn count_outcomes<'a>(items: impl Iterator<Item = &'a str>) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for item in items {
        *map.entry(item.to_string()).or_insert(0) += 1;
    }
    map
}

pub(crate) fn render_comparison_markdown(summary: &ComparisonSummary) -> String {
    let mut md = String::new();
    md.push_str(
        "# Redundant Map Salvage Comparison

",
    );
    md.push_str("Purpose: targeted deterministic before/after comparison of old-style (no redundant map) and new-style (redundant map) archives.

");
    md.push_str("Strict boundary reminder: research-only salvage outputs; canonical extraction semantics remain unchanged.

");
    md.push_str(&format!(
        "- Scenario count: `{}`
",
        summary.scenario_count
    ));
    md.push_str(&format!(
        "- Old archive count: `{}`
",
        summary.old_archive_count
    ));
    md.push_str(&format!(
        "- New archive count: `{}`
",
        summary.new_archive_count
    ));
    md.push_str(&format!(
        "- Orphan→salvage improvements: `{}`
",
        summary.orphan_to_salvage_improvements
    ));
    md.push_str(&format!(
        "- No-evidence→salvage improvements: `{}`

",
        summary.no_evidence_to_salvage_improvements
    ));

    md.push_str(
        "## Outcome totals

",
    );
    md.push_str(
        "- Old:
",
    );
    for (k, v) in &summary.old_outcome_counts {
        md.push_str(&format!(
            "  - {}: {}
",
            k, v
        ));
    }
    md.push_str(
        "- New:
",
    );
    for (k, v) in &summary.new_outcome_counts {
        md.push_str(&format!(
            "  - {}: {}
",
            k, v
        ));
    }

    md.push_str(
        "
## Grouped by dataset

",
    );
    for g in &summary.by_dataset {
        md.push_str(&format!(
            "- `{}`: scenarios={}, Δverified={}, Δsalvageable={}
",
            g.key, g.scenario_count, g.verified_block_delta, g.salvageable_file_delta
        ));
    }

    md.push_str(
        "
## Grouped by corruption target

",
    );
    for g in &summary.by_corruption_target {
        md.push_str(&format!(
            "- `{}`: scenarios={}, Δverified={}, Δsalvageable={}
",
            g.key, g.scenario_count, g.verified_block_delta, g.salvageable_file_delta
        ));
    }

    md.push_str(
        "
## Most improved scenarios

",
    );
    for row in summary
        .per_scenario_rows
        .iter()
        .filter(|r| r.improvement_class.starts_with("IMPROVED"))
        .take(5)
    {
        md.push_str(&format!(
            "- `{}`: {} → {} ({})
",
            row.scenario_id, row.old_outcome, row.new_outcome, row.improvement_class
        ));
    }

    md.push_str(
        "
## No improvement / degraded scenarios

",
    );
    for row in summary
        .per_scenario_rows
        .iter()
        .filter(|r| r.improvement_class == "UNCHANGED" || r.improvement_class == "DEGRADED")
        .take(8)
    {
        md.push_str(&format!(
            "- `{}`: {} → {} ({})
",
            row.scenario_id, row.old_outcome, row.new_outcome, row.improvement_class
        ));
    }

    md
}

pub(crate) fn run_redundant_map_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "crushr-lab-comparison-{}-{}",
        std::process::id(),
        unique
    ));
    if temp.exists() {
        fs::remove_dir_all(&temp).with_context(|| format!("remove {}", temp.display()))?;
    }
    fs::create_dir_all(&temp).with_context(|| format!("create {}", temp.display()))?;
    let datasets_root = temp.join("datasets");
    let archives_root = temp.join("archives");
    fs::create_dir_all(&datasets_root)?;
    fs::create_dir_all(&archives_root)?;

    let pack_bin = resolve_pack_bin()?;
    let salvage_bin = resolve_salvage_bin()?;

    let mut rows = Vec::new();
    for scenario in comparison_scenarios() {
        let dataset_input = datasets_root.join(scenario.dataset);
        if !dataset_input.exists() {
            write_dataset_fixture(&datasets_root, scenario.dataset)?;
        }

        let old_archive = archives_root.join(format!("{}_old.crushr", scenario.scenario_id));
        let new_archive = archives_root.join(format!("{}_new.crushr", scenario.scenario_id));
        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &new_archive)?;
        remove_ledger_for_old_style(&old_archive)?;

        if scenario.break_redundant_map {
            damage_redundant_map_ledger(&new_archive)?;
        }
        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&new_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let new_plan = run_salvage_plan(
            &salvage_bin,
            &new_archive,
            &archives_root.join(format!("{}_new_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let new_metrics = outcome_from_plan(&new_plan);
        let improvement = classify_improvement(&old_metrics.outcome, &new_metrics.outcome);
        if verbose {
            eprintln!("scenario {} => {}", scenario.scenario_id, improvement);
        }

        rows.push(ScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            new_outcome: new_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            new_verified_block_count: new_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            new_salvageable_file_count: new_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            new_exported_full_file_count: new_metrics.exported_full_file_count,
            improvement_class: improvement,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));

    let summary = ComparisonSummary {
        schema_version: "crushr-lab-salvage-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: crushr::product_version(),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_archive_count: rows.len(),
        new_archive_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        new_outcome_counts: count_outcomes(rows.iter().map(|r| r.new_outcome.as_str())),
        orphan_to_salvage_improvements: rows
            .iter()
            .filter(|r| {
                matches!(
                    r.improvement_class.as_str(),
                    "IMPROVED_ORPHAN_TO_PARTIAL" | "IMPROVED_ORPHAN_TO_FULL"
                )
            })
            .count() as u64,
        no_evidence_to_salvage_improvements: rows
            .iter()
            .filter(|r| {
                matches!(
                    r.improvement_class.as_str(),
                    "IMPROVED_NONE_TO_PARTIAL" | "IMPROVED_NONE_TO_FULL"
                )
            })
            .count() as u64,
        unchanged_outcome_count: rows
            .iter()
            .filter(|r| r.improvement_class == "UNCHANGED")
            .count() as u64,
        degraded_outcome_count: rows
            .iter()
            .filter(|r| r.improvement_class == "DEGRADED")
            .count() as u64,
        total_verified_block_delta: rows
            .iter()
            .map(|r| r.new_verified_block_count as i64 - r.old_verified_block_count as i64)
            .sum(),
        total_salvageable_file_delta: rows
            .iter()
            .map(|r| r.new_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
            .sum(),
        total_exported_full_file_delta: rows
            .iter()
            .map(|r| r.new_exported_full_file_count as i64 - r.old_exported_full_file_count as i64)
            .sum(),
        by_dataset: build_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_groups(&rows, |r| &r.corruption_target),
        by_corruption_model: build_groups(&rows, |r| &r.corruption_model),
        by_magnitude: build_groups(&rows, |r| &r.magnitude),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("comparison_summary.md"),
        render_comparison_markdown(&summary),
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}
