use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};
use crushr_format::blk3::{read_blk3_header, write_blk3_header, Blk3Header};

pub(super) fn comparison_scenarios() -> Vec<ComparisonScenario> {
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

pub(super) fn write_dataset_fixture(root: &Path, dataset: &str) -> Result<()> {
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

pub(super) fn build_archive_with_pack(pack_bin: &Path, input: &Path, output: &Path) -> Result<()> {
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

pub(super) fn build_archive_with_pack_experimental(
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

pub(super) fn build_archive_with_pack_file_identity(
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

pub(super) fn build_archive_with_pack_format05(
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

pub(super) fn build_archive_with_pack_format08(
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

pub(super) fn remove_ledger_for_old_style(archive_path: &Path) -> Result<()> {
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

pub(super) fn corrupt_archive(archive_path: &Path, scenario: &ComparisonScenario) -> Result<()> {
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

pub(super) fn damage_redundant_map_ledger(archive_path: &Path) -> Result<()> {
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

pub(super) fn run_salvage_plan(
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

pub(super) fn outcome_from_plan(plan: &Value) -> OutcomeMetrics {
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

pub(super) fn outcome_rank(value: &str) -> i32 {
    match value {
        "NO_VERIFIED_EVIDENCE" => 0,
        "ORPHAN_EVIDENCE_ONLY" => 1,
        "PARTIAL_FILE_SALVAGE" => 2,
        "FULL_FILE_SALVAGE_AVAILABLE" => 3,
        _ => -1,
    }
}

pub(super) fn classify_improvement(old: &str, new: &str) -> String {
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

pub(super) fn build_groups(
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

pub(super) fn count_outcomes<'a>(items: impl Iterator<Item = &'a str>) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for item in items {
        *map.entry(item.to_string()).or_insert(0) += 1;
    }
    map
}

pub(super) fn render_comparison_markdown(summary: &ComparisonSummary) -> String {
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

pub(super) fn run_redundant_map_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
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
        tool_version: env!("CARGO_PKG_VERSION"),
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

pub(super) fn build_experimental_groups(
    rows: &[ExperimentalScenarioRow],
    key_fn: impl Fn(&ExperimentalScenarioRow) -> &str,
) -> Vec<ExperimentalComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&ExperimentalScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| ExperimentalComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            redundant_outcome_counts: count_outcomes(
                values.iter().map(|r| r.redundant_outcome.as_str()),
            ),
            experimental_outcome_counts: count_outcomes(
                values.iter().map(|r| r.experimental_outcome.as_str()),
            ),
            file_identity_outcome_counts: count_outcomes(
                values.iter().map(|r| r.file_identity_outcome.as_str()),
            ),
            verified_block_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_verified_block_count as i64 - r.old_verified_block_count as i64
                })
                .sum(),
            salvageable_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_salvageable_file_count as i64
                        - r.old_salvageable_file_count as i64
                })
                .sum(),
            exported_full_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.file_identity_exported_full_file_count as i64
                        - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

pub(super) fn build_format05_groups(
    rows: &[Format05ScenarioRow],
    key_fn: impl Fn(&Format05ScenarioRow) -> &str,
) -> Vec<Format05ComparisonGroup> {
    let mut grouped: BTreeMap<String, Vec<&Format05ScenarioRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry(key_fn(row).to_string())
            .or_default()
            .push(row);
    }

    grouped
        .into_iter()
        .map(|(key, values)| Format05ComparisonGroup {
            key,
            scenario_count: values.len(),
            old_outcome_counts: count_outcomes(values.iter().map(|r| r.old_outcome.as_str())),
            redundant_outcome_counts: count_outcomes(
                values.iter().map(|r| r.redundant_outcome.as_str()),
            ),
            experimental_outcome_counts: count_outcomes(
                values.iter().map(|r| r.experimental_outcome.as_str()),
            ),
            file_identity_outcome_counts: count_outcomes(
                values.iter().map(|r| r.file_identity_outcome.as_str()),
            ),
            format05_outcome_counts: count_outcomes(
                values.iter().map(|r| r.format05_outcome.as_str()),
            ),
            verified_block_delta_vs_old: values
                .iter()
                .map(|r| r.format05_verified_block_count as i64 - r.old_verified_block_count as i64)
                .sum(),
            salvageable_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.format05_salvageable_file_count as i64 - r.old_salvageable_file_count as i64
                })
                .sum(),
            exported_full_file_delta_vs_old: values
                .iter()
                .map(|r| {
                    r.format05_exported_full_file_count as i64
                        - r.old_exported_full_file_count as i64
                })
                .sum(),
        })
        .collect()
}

pub(super) fn run_experimental_resilience_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let scenarios = comparison_scenarios();
    let pack_bin = resolve_pack_bin()?;
    let salvage_bin = resolve_salvage_bin()?;
    let temp = comparison_dir.join(".tmp_experimental");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;
    let archives_root = temp.join("archives");
    fs::create_dir_all(&archives_root)?;

    let mut rows = Vec::new();
    for scenario in scenarios {
        let dataset_input = temp.join("datasets").join(&scenario.scenario_id);
        write_dataset_fixture(&dataset_input, scenario.dataset)?;

        let old_archive = archives_root.join(format!("{}_old.crushr", scenario.scenario_id));
        let redundant_archive =
            archives_root.join(format!("{}_redundant.crushr", scenario.scenario_id));
        let experimental_archive =
            archives_root.join(format!("{}_experimental.crushr", scenario.scenario_id));
        let file_identity_archive =
            archives_root.join(format!("{}_file_identity.crushr", scenario.scenario_id));
        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &redundant_archive)?;
        build_archive_with_pack_experimental(&pack_bin, &dataset_input, &experimental_archive)?;
        build_archive_with_pack_file_identity(&pack_bin, &dataset_input, &file_identity_archive)?;
        remove_ledger_for_old_style(&old_archive)?;

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&experimental_archive, &scenario)?;
        corrupt_archive(&file_identity_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &archives_root.join(format!("{}_redundant_plan.json", scenario.scenario_id)),
        )?;
        let experimental_plan = run_salvage_plan(
            &salvage_bin,
            &experimental_archive,
            &archives_root.join(format!("{}_experimental_plan.json", scenario.scenario_id)),
        )?;
        let file_identity_plan = run_salvage_plan(
            &salvage_bin,
            &file_identity_archive,
            &archives_root.join(format!("{}_file_identity_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let experimental_metrics = outcome_from_plan(&experimental_plan);
        let file_identity_metrics = outcome_from_plan(&file_identity_plan);
        if verbose {
            eprintln!(
                "scenario {} => {} / {} / {} / {}",
                scenario.scenario_id,
                old_metrics.outcome,
                redundant_metrics.outcome,
                experimental_metrics.outcome,
                file_identity_metrics.outcome
            );
        }

        rows.push(ExperimentalScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            redundant_outcome: redundant_metrics.outcome,
            experimental_outcome: experimental_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            redundant_verified_block_count: redundant_metrics.verified_block_count,
            experimental_verified_block_count: experimental_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            redundant_salvageable_file_count: redundant_metrics.salvageable_file_count,
            experimental_salvageable_file_count: experimental_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            redundant_exported_full_file_count: redundant_metrics.exported_full_file_count,
            experimental_exported_full_file_count: experimental_metrics.exported_full_file_count,
            file_identity_outcome: file_identity_metrics.outcome,
            file_identity_verified_block_count: file_identity_metrics.verified_block_count,
            file_identity_salvageable_file_count: file_identity_metrics.salvageable_file_count,
            file_identity_exported_full_file_count: file_identity_metrics.exported_full_file_count,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let summary = ExperimentalComparisonSummary {
        schema_version: "crushr-lab-salvage-experimental-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        redundant_outcome_counts: count_outcomes(rows.iter().map(|r| r.redundant_outcome.as_str())),
        experimental_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.experimental_outcome.as_str()),
        ),
        file_identity_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.file_identity_outcome.as_str()),
        ),
        orphan_to_salvage_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                outcome_rank(&r.old_outcome) <= 1 && outcome_rank(&r.file_identity_outcome) >= 2
            })
            .count() as u64,
        orphan_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.file_identity_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        orphan_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.file_identity_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        orphan_to_salvage_improvements_vs_redundant: rows
            .iter()
            .filter(|r| {
                outcome_rank(&r.redundant_outcome) <= 1
                    && outcome_rank(&r.file_identity_outcome) >= 2
            })
            .count() as u64,
        no_evidence_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.file_identity_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        no_evidence_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.file_identity_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        total_verified_block_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_verified_block_count as i64 - r.old_verified_block_count as i64
            })
            .sum(),
        total_salvageable_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_salvageable_file_count as i64 - r.old_salvageable_file_count as i64
            })
            .sum(),
        total_exported_full_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.file_identity_exported_full_file_count as i64
                    - r.old_exported_full_file_count as i64
            })
            .sum(),
        by_dataset: build_experimental_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_experimental_groups(&rows, |r| &r.corruption_target),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("format04_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("file_identity_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    let md = format!(
        "# Format-04 comparison\n\nScenarios: {}\n\n- orphan->salvage vs old: {}\n- orphan->salvage vs redundant: {}\n- orphan->partial vs old: {}\n- orphan->full vs old: {}\n- no-evidence->partial vs old: {}\n- no-evidence->full vs old: {}\n",
        summary.scenario_count,
        summary.orphan_to_salvage_improvements_vs_old,
        summary.orphan_to_salvage_improvements_vs_redundant,
        summary.orphan_to_partial_improvements_vs_old,
        summary.orphan_to_full_improvements_vs_old,
        summary.no_evidence_to_partial_improvements_vs_old,
        summary.no_evidence_to_full_improvements_vs_old
    );
    fs::write(
        comparison_dir.join("format04_comparison_summary.md"),
        md.clone(),
    )?;
    fs::write(
        comparison_dir.join("file_identity_comparison_summary.md"),
        md,
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(super) fn run_format05_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = comparison_dir.join(".tmp_format05");
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
        let redundant_archive =
            archives_root.join(format!("{}_redundant.crushr", scenario.scenario_id));
        let experimental_archive =
            archives_root.join(format!("{}_experimental.crushr", scenario.scenario_id));
        let file_identity_archive =
            archives_root.join(format!("{}_file_identity.crushr", scenario.scenario_id));
        let format05_archive =
            archives_root.join(format!("{}_format05.crushr", scenario.scenario_id));

        build_archive_with_pack(&pack_bin, &dataset_input, &old_archive)?;
        build_archive_with_pack(&pack_bin, &dataset_input, &redundant_archive)?;
        build_archive_with_pack_experimental(&pack_bin, &dataset_input, &experimental_archive)?;
        build_archive_with_pack_file_identity(&pack_bin, &dataset_input, &file_identity_archive)?;
        build_archive_with_pack_format05(&pack_bin, &dataset_input, &format05_archive)?;

        remove_ledger_for_old_style(&old_archive)?;
        if scenario.break_redundant_map {
            damage_redundant_map_ledger(&redundant_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&experimental_archive, &scenario)?;
        corrupt_archive(&file_identity_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &archives_root.join(format!("{}_old_plan.json", scenario.scenario_id)),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &archives_root.join(format!("{}_redundant_plan.json", scenario.scenario_id)),
        )?;
        let experimental_plan = run_salvage_plan(
            &salvage_bin,
            &experimental_archive,
            &archives_root.join(format!("{}_experimental_plan.json", scenario.scenario_id)),
        )?;
        let file_identity_plan = run_salvage_plan(
            &salvage_bin,
            &file_identity_archive,
            &archives_root.join(format!("{}_file_identity_plan.json", scenario.scenario_id)),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &archives_root.join(format!("{}_format05_plan.json", scenario.scenario_id)),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let experimental_metrics = outcome_from_plan(&experimental_plan);
        let file_identity_metrics = outcome_from_plan(&file_identity_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        if verbose {
            eprintln!(
                "scenario {} => format05 {}",
                scenario.scenario_id, format05_metrics.outcome
            );
        }

        rows.push(Format05ScenarioRow {
            scenario_id: scenario.scenario_id,
            dataset: scenario.dataset.to_string(),
            corruption_model: scenario.corruption_model.to_string(),
            corruption_target: scenario.corruption_target.to_string(),
            magnitude: scenario.magnitude.to_string(),
            seed: scenario.seed,
            old_outcome: old_metrics.outcome,
            redundant_outcome: redundant_metrics.outcome,
            experimental_outcome: experimental_metrics.outcome,
            file_identity_outcome: file_identity_metrics.outcome,
            format05_outcome: format05_metrics.outcome,
            old_verified_block_count: old_metrics.verified_block_count,
            redundant_verified_block_count: redundant_metrics.verified_block_count,
            experimental_verified_block_count: experimental_metrics.verified_block_count,
            file_identity_verified_block_count: file_identity_metrics.verified_block_count,
            format05_verified_block_count: format05_metrics.verified_block_count,
            old_salvageable_file_count: old_metrics.salvageable_file_count,
            redundant_salvageable_file_count: redundant_metrics.salvageable_file_count,
            experimental_salvageable_file_count: experimental_metrics.salvageable_file_count,
            file_identity_salvageable_file_count: file_identity_metrics.salvageable_file_count,
            format05_salvageable_file_count: format05_metrics.salvageable_file_count,
            old_exported_full_file_count: old_metrics.exported_full_file_count,
            redundant_exported_full_file_count: redundant_metrics.exported_full_file_count,
            experimental_exported_full_file_count: experimental_metrics.exported_full_file_count,
            file_identity_exported_full_file_count: file_identity_metrics.exported_full_file_count,
            format05_exported_full_file_count: format05_metrics.exported_full_file_count,
        });
    }

    rows.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let summary = Format05ComparisonSummary {
        schema_version: "crushr-lab-salvage-format05-comparison.v1",
        tool: "crushr-lab-salvage",
        tool_version: env!("CARGO_PKG_VERSION"),
        verification_label: VERIFICATION_LABEL,
        scenario_count: rows.len(),
        old_outcome_counts: count_outcomes(rows.iter().map(|r| r.old_outcome.as_str())),
        redundant_outcome_counts: count_outcomes(rows.iter().map(|r| r.redundant_outcome.as_str())),
        experimental_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.experimental_outcome.as_str()),
        ),
        file_identity_outcome_counts: count_outcomes(
            rows.iter().map(|r| r.file_identity_outcome.as_str()),
        ),
        format05_outcome_counts: count_outcomes(rows.iter().map(|r| r.format05_outcome.as_str())),
        orphan_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.format05_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        orphan_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "ORPHAN_EVIDENCE_ONLY"
                    && r.format05_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        no_evidence_to_partial_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.format05_outcome == "PARTIAL_FILE_SALVAGE"
            })
            .count() as u64,
        no_evidence_to_full_improvements_vs_old: rows
            .iter()
            .filter(|r| {
                r.old_outcome == "NO_VERIFIED_EVIDENCE"
                    && r.format05_outcome == "FULL_FILE_SALVAGE_AVAILABLE"
            })
            .count() as u64,
        total_verified_block_delta_vs_old: rows
            .iter()
            .map(|r| r.format05_verified_block_count as i64 - r.old_verified_block_count as i64)
            .sum(),
        total_salvageable_file_delta_vs_old: rows
            .iter()
            .map(|r| r.format05_salvageable_file_count as i64 - r.old_salvageable_file_count as i64)
            .sum(),
        total_exported_full_file_delta_vs_old: rows
            .iter()
            .map(|r| {
                r.format05_exported_full_file_count as i64 - r.old_exported_full_file_count as i64
            })
            .sum(),
        by_dataset: build_format05_groups(&rows, |r| &r.dataset),
        by_corruption_target: build_format05_groups(&rows, |r| &r.corruption_target),
        per_scenario_rows: rows,
    };

    fs::write(
        comparison_dir.join("format05_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    let md = format!(
        "# Format-05 comparison\n\nScenarios: {}\n\n- orphan->partial vs old: {}\n- orphan->full vs old: {}\n- no-evidence->partial vs old: {}\n- no-evidence->full vs old: {}\n- total verified block delta vs old: {}\n- total salvageable file delta vs old: {}\n- total exported full-file delta vs old: {}\n",
        summary.scenario_count,
        summary.orphan_to_partial_improvements_vs_old,
        summary.orphan_to_full_improvements_vs_old,
        summary.no_evidence_to_partial_improvements_vs_old,
        summary.no_evidence_to_full_improvements_vs_old,
        summary.total_verified_block_delta_vs_old,
        summary.total_salvageable_file_delta_vs_old,
        summary.total_exported_full_file_delta_vs_old,
    );
    fs::write(comparison_dir.join("format05_comparison_summary.md"), md)?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(super) fn build_archive_with_pack_format06(
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
        .arg(FORMAT06_PACK_FLAG)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn recovery_classification_counts(plan: &Value) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    if let Some(rows) = plan.get("file_plans").and_then(Value::as_array) {
        for row in rows {
            if let Some(classification) = row.get("recovery_classification").and_then(Value::as_str)
            {
                *counts.entry(classification.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
}

fn classification_delta(
    base: &BTreeMap<String, u64>,
    candidate: &BTreeMap<String, u64>,
    key: &str,
) -> i64 {
    candidate.get(key).copied().unwrap_or(0) as i64 - base.get(key).copied().unwrap_or(0) as i64
}

fn merge_classification_counts(rows: &[Value], field: &str) -> BTreeMap<String, u64> {
    let mut merged = BTreeMap::<String, u64>::new();
    for row in rows {
        if let Some(counts) = row.get(field).and_then(Value::as_object) {
            for (k, v) in counts {
                *merged.entry(k.clone()).or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
    }
    merged
}

fn metadata_node_count(plan: &Value, schema: &str) -> u64 {
    plan.get("experimental_metadata")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row.get("schema").and_then(Value::as_str) == Some(schema))
                .count() as u64
        })
        .unwrap_or(0)
}

pub(super) fn run_format08_placement_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format08-placement-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let strategies = [
        FORMAT08_STRATEGY_FIXED,
        FORMAT08_STRATEGY_HASH,
        FORMAT08_STRATEGY_GOLDEN,
    ];
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for strategy in strategies {
            let archive = scenario_dir.join(format!("format08_{}.crushr", strategy));
            build_archive_with_pack_format08(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                strategy,
            )?;
            corrupt_archive(&archive, &scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan_{}.json", strategy)),
            )?;
            let metrics = outcome_from_plan(&plan);
            let classes = recovery_classification_counts(&plan);
            let manifest_survival =
                metadata_node_count(&plan, "crushr-file-manifest-checkpoint.v1");
            let path_survival = metadata_node_count(&plan, "crushr-path-checkpoint.v1");
            let metadata_nodes = manifest_survival + path_survival;
            if verbose {
                eprintln!(
                    "scenario {} strategy {} => {}",
                    scenario.scenario_id, strategy, metrics.outcome
                );
            }
            rows.push(serde_json::json!({
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "placement_strategy": strategy,
                "outcome": metrics.outcome,
                "verified_block_count": metrics.verified_block_count,
                "salvageable_file_count": metrics.salvageable_file_count,
                "recovery_classification_counts": classes,
                "manifest_checkpoint_survival_count": manifest_survival,
                "path_checkpoint_survival_count": path_survival,
                "verified_metadata_node_count": metadata_nodes,
            }));
        }
    }

    let mut by_strategy = serde_json::Map::new();
    for strategy in strategies {
        let strategy_rows: Vec<Value> = rows
            .iter()
            .filter(|r| r["placement_strategy"] == strategy)
            .cloned()
            .collect();
        let outcomes = count_outcomes(
            strategy_rows
                .iter()
                .filter_map(|r| r.get("outcome").and_then(Value::as_str)),
        );
        let classes = merge_classification_counts(&strategy_rows, "recovery_classification_counts");
        let manifest_checkpoint_survival_count: u64 = strategy_rows
            .iter()
            .map(|r| {
                r["manifest_checkpoint_survival_count"]
                    .as_u64()
                    .unwrap_or(0)
            })
            .sum();
        let path_checkpoint_survival_count: u64 = strategy_rows
            .iter()
            .map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0))
            .sum();
        let verified_metadata_node_count: u64 = strategy_rows
            .iter()
            .map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0))
            .sum();
        by_strategy.insert(strategy.to_string(), serde_json::json!({
            "scenario_count": strategy_rows.len(),
            "recovery_outcome_counts": outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0),
            "anonymous_recovery_count": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0),
            "partial_ordered_recovery_count": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0),
            "partial_unordered_recovery_count": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0),
            "orphan_evidence_count": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0),
            "manifest_checkpoint_survival_count": manifest_checkpoint_survival_count,
            "path_checkpoint_survival_count": path_checkpoint_survival_count,
            "verified_metadata_node_count": verified_metadata_node_count,
        }));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format08-placement-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "placement_strategies": strategies,
        "by_placement_strategy": by_strategy,
        "by_dataset": group_rows_by_key(&rows, "dataset"),
        "by_corruption_target": group_rows_by_key(&rows, "corruption_target"),
        "metadata_layer_failure_focus": {
            "no_manifest_checkpoint_rows": rows.iter().filter(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0) == 0).count(),
            "no_path_checkpoint_rows": rows.iter().filter(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0) == 0).count(),
            "no_metadata_nodes_rows": rows.iter().filter(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0) == 0).count(),
        },
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format08_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format08_comparison_summary.md"),
        format!(
            "# Format-08 placement comparison

Scenarios: {}

Strategies: fixed_spread, hash_spread, golden_spread
",
            summary
                .get("scenario_count")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
    )?;
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn group_rows_by_key(rows: &[Value], key: &str) -> Value {
    let mut out = serde_json::Map::new();
    let mut keys = BTreeMap::<String, Vec<Value>>::new();
    for row in rows {
        if let Some(k) = row.get(key).and_then(Value::as_str) {
            keys.entry(k.to_string()).or_default().push(row.clone());
        }
    }
    for (k, group) in keys {
        let outcomes = count_outcomes(
            group
                .iter()
                .filter_map(|r| r.get("outcome").and_then(Value::as_str)),
        );
        let classes = merge_classification_counts(&group, "recovery_classification_counts");
        out.insert(k, serde_json::json!({
            "scenario_count": group.len(),
            "recovery_outcome_counts": outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0),
            "anonymous_recovery_count": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0),
            "partial_ordered_recovery_count": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0),
            "partial_unordered_recovery_count": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0),
            "orphan_evidence_count": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0),
            "manifest_checkpoint_survival_count": group.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "path_checkpoint_survival_count": group.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "verified_metadata_node_count": group.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        }));
    }
    Value::Object(out)
}

pub(super) fn run_format06_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format06-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        let old_archive = scenario_dir.join("old.crushr");
        let redundant_archive = scenario_dir.join("redundant.crushr");
        let format05_archive = scenario_dir.join("format05.crushr");
        let format06_archive = scenario_dir.join("format06.crushr");

        build_archive_with_pack(&pack_bin, &input_dir.join(scenario.dataset), &old_archive)?;
        build_archive_with_pack_experimental(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &redundant_archive,
        )?;
        build_archive_with_pack_format05(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format05_archive,
        )?;
        build_archive_with_pack_format06(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format06_archive,
        )?;

        if scenario.break_redundant_map {
            remove_ledger_for_old_style(&old_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;
        corrupt_archive(&format06_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &scenario_dir.join("old_plan.json"),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &scenario_dir.join("redundant_plan.json"),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &scenario_dir.join("format05_plan.json"),
        )?;
        let format06_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format06_plan.json"),
        )?;
        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        let format06_metrics = outcome_from_plan(&format06_plan);
        let format05_classifications = recovery_classification_counts(&format05_plan);
        let format06_classifications = recovery_classification_counts(&format06_plan);

        if verbose {
            eprintln!(
                "scenario {} => format06 {}",
                scenario.scenario_id, format06_metrics.outcome
            );
        }

        rows.push(serde_json::json!({
            "scenario_id": scenario.scenario_id,
            "dataset": scenario.dataset,
            "corruption_model": scenario.corruption_model,
            "corruption_target": scenario.corruption_target,
            "magnitude": scenario.magnitude,
            "seed": scenario.seed,
            "old_outcome": old_metrics.outcome,
            "redundant_outcome": redundant_metrics.outcome,
            "format05_outcome": format05_metrics.outcome,
            "format06_outcome": format06_metrics.outcome,
            "old_verified_block_count": old_metrics.verified_block_count,
            "redundant_verified_block_count": redundant_metrics.verified_block_count,
            "format05_verified_block_count": format05_metrics.verified_block_count,
            "format06_verified_block_count": format06_metrics.verified_block_count,
            "old_salvageable_file_count": old_metrics.salvageable_file_count,
            "redundant_salvageable_file_count": redundant_metrics.salvageable_file_count,
            "format05_salvageable_file_count": format05_metrics.salvageable_file_count,
            "format06_salvageable_file_count": format06_metrics.salvageable_file_count,
            "format05_recovery_classification_counts": format05_classifications,
            "format06_recovery_classification_counts": format06_classifications
        }));
    }

    let mut format05_recovery_classification_counts = BTreeMap::<String, u64>::new();
    let mut format06_recovery_classification_counts = BTreeMap::<String, u64>::new();
    for row in &rows {
        if let Some(counts) = row
            .get("format05_recovery_classification_counts")
            .and_then(Value::as_object)
        {
            for (k, v) in counts {
                *format05_recovery_classification_counts
                    .entry(k.clone())
                    .or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
        if let Some(counts) = row
            .get("format06_recovery_classification_counts")
            .and_then(Value::as_object)
        {
            for (k, v) in counts {
                *format06_recovery_classification_counts
                    .entry(k.clone())
                    .or_insert(0) += v.as_u64().unwrap_or(0);
            }
        }
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format06-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "old_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("old_outcome").and_then(Value::as_str))),
        "redundant_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("redundant_outcome").and_then(Value::as_str))),
        "format05_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format05_outcome").and_then(Value::as_str))),
        "format06_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format06_outcome").and_then(Value::as_str))),
        "format05_recovery_classification_counts": format05_recovery_classification_counts,
        "format06_recovery_classification_counts": format06_recovery_classification_counts,
        "recovery_classification_delta_vs_format05": {
            "full_verified_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "FULL_VERIFIED"),
            "full_anonymous_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "FULL_ANONYMOUS"),
            "partial_ordered_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "PARTIAL_ORDERED"),
            "partial_unordered_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "PARTIAL_UNORDERED"),
            "orphan_blocks_delta": classification_delta(&format05_recovery_classification_counts, &format06_recovery_classification_counts, "ORPHAN_BLOCKS")
        },
        "per_scenario_rows": rows
    });

    fs::write(
        comparison_dir.join("format06_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format06_comparison_summary.md"),
        format!(
            "# Format-06 comparison\n\nScenarios: {}\n\n- outcomes tracked: old, redundant, format05, format06\n",
            summary.get("scenario_count").and_then(Value::as_u64).unwrap_or(0)
        ),
    )?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

pub(super) fn run_format07_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format07-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        let old_archive = scenario_dir.join("old.crushr");
        let redundant_archive = scenario_dir.join("redundant.crushr");
        let format05_archive = scenario_dir.join("format05.crushr");
        let format06_archive = scenario_dir.join("format06.crushr");

        build_archive_with_pack(&pack_bin, &input_dir.join(scenario.dataset), &old_archive)?;
        build_archive_with_pack_experimental(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &redundant_archive,
        )?;
        build_archive_with_pack_format05(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format05_archive,
        )?;
        build_archive_with_pack_format06(
            &pack_bin,
            &input_dir.join(scenario.dataset),
            &format06_archive,
        )?;

        if scenario.break_redundant_map {
            remove_ledger_for_old_style(&old_archive)?;
        }

        corrupt_archive(&old_archive, &scenario)?;
        corrupt_archive(&redundant_archive, &scenario)?;
        corrupt_archive(&format05_archive, &scenario)?;
        corrupt_archive(&format06_archive, &scenario)?;

        let old_plan = run_salvage_plan(
            &salvage_bin,
            &old_archive,
            &scenario_dir.join("old_plan.json"),
        )?;
        let redundant_plan = run_salvage_plan(
            &salvage_bin,
            &redundant_archive,
            &scenario_dir.join("redundant_plan.json"),
        )?;
        let format05_plan = run_salvage_plan(
            &salvage_bin,
            &format05_archive,
            &scenario_dir.join("format05_plan.json"),
        )?;
        let format06_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format06_plan.json"),
        )?;
        let format07_plan = run_salvage_plan(
            &salvage_bin,
            &format06_archive,
            &scenario_dir.join("format07_plan.json"),
        )?;

        let old_metrics = outcome_from_plan(&old_plan);
        let redundant_metrics = outcome_from_plan(&redundant_plan);
        let format05_metrics = outcome_from_plan(&format05_plan);
        let format06_metrics = outcome_from_plan(&format06_plan);
        let format07_metrics = outcome_from_plan(&format07_plan);
        let format06_classifications = recovery_classification_counts(&format06_plan);
        let format07_classifications = recovery_classification_counts(&format07_plan);

        if verbose {
            eprintln!(
                "scenario {} => format07 {}",
                scenario.scenario_id, format07_metrics.outcome
            );
        }

        rows.push(serde_json::json!({
            "scenario_id": scenario.scenario_id,
            "dataset": scenario.dataset,
            "corruption_model": scenario.corruption_model,
            "corruption_target": scenario.corruption_target,
            "magnitude": scenario.magnitude,
            "seed": scenario.seed,
            "old_outcome": old_metrics.outcome,
            "redundant_outcome": redundant_metrics.outcome,
            "format05_outcome": format05_metrics.outcome,
            "format06_outcome": format06_metrics.outcome,
            "format07_outcome": format07_metrics.outcome,
            "old_verified_block_count": old_metrics.verified_block_count,
            "redundant_verified_block_count": redundant_metrics.verified_block_count,
            "format05_verified_block_count": format05_metrics.verified_block_count,
            "format06_verified_block_count": format06_metrics.verified_block_count,
            "format07_verified_block_count": format07_metrics.verified_block_count,
            "old_salvageable_file_count": old_metrics.salvageable_file_count,
            "redundant_salvageable_file_count": redundant_metrics.salvageable_file_count,
            "format05_salvageable_file_count": format05_metrics.salvageable_file_count,
            "format06_salvageable_file_count": format06_metrics.salvageable_file_count,
            "format07_salvageable_file_count": format07_metrics.salvageable_file_count,
            "format06_recovery_classification_counts": format06_classifications,
            "format07_recovery_classification_counts": format07_classifications
        }));
    }

    let format06_recovery_classification_counts =
        merge_classification_counts(&rows, "format06_recovery_classification_counts");
    let format07_recovery_classification_counts =
        merge_classification_counts(&rows, "format07_recovery_classification_counts");

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format07-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "old_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("old_outcome").and_then(Value::as_str))),
        "redundant_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("redundant_outcome").and_then(Value::as_str))),
        "format05_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format05_outcome").and_then(Value::as_str))),
        "format06_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format06_outcome").and_then(Value::as_str))),
        "format07_outcome_counts": count_outcomes(rows.iter().filter_map(|r| r.get("format07_outcome").and_then(Value::as_str))),
        "format06_recovery_classification_counts": format06_recovery_classification_counts,
        "format07_recovery_classification_counts": format07_recovery_classification_counts,
        "recovery_classification_delta_vs_format06": {
            "full_named_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "FULL_NAMED_VERIFIED"),
            "full_anonymous_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "FULL_ANONYMOUS_VERIFIED"),
            "partial_ordered_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "PARTIAL_ORDERED_VERIFIED"),
            "partial_unordered_verified_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "PARTIAL_UNORDERED_VERIFIED"),
            "orphan_evidence_only_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "ORPHAN_EVIDENCE_ONLY"),
            "no_verified_evidence_delta": classification_delta(&format06_recovery_classification_counts, &format07_recovery_classification_counts, "NO_VERIFIED_EVIDENCE")
        },
        "per_scenario_rows": rows
    });

    fs::write(
        comparison_dir.join("format07_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        comparison_dir.join("format07_comparison_summary.md"),
        format!(
            "# Format-07 comparison\n\nScenarios: {}\n\n- outcomes tracked: old, redundant, format05, format06, format07\n",
            summary.get("scenario_count").and_then(Value::as_u64).unwrap_or(0)
        ),
    )?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Debug, Clone)]
struct Format09Scenario {
    scenario_id: String,
    dataset: &'static str,
    metadata_regime: &'static str,
    metadata_target: &'static str,
    metadata_operation: &'static str,
    payload_damage: &'static str,
    seed: u64,
}

fn format09_scenarios() -> Vec<Format09Scenario> {
    let datasets = ["smallfiles", "mixed", "largefiles"];
    let regimes = [
        "metadata_intact",
        "metadata_destroyed",
        "metadata_partial",
        "metadata_conflicting",
    ];
    let targets = [
        "index",
        "manifest",
        "path_checkpoint",
        "tail_metadata",
        "multiple_metadata_layers",
        "index_manifest",
    ];
    let payload = [
        "none",
        "single_block_delete",
        "sparse_delete",
        "cluster_delete",
        "block_reorder",
        "duplicate_block_insert",
    ];

    let mut out = Vec::new();
    let mut seed = 900u64;
    for (i, p) in payload.iter().enumerate() {
        for regime in regimes {
            let target = targets[i % targets.len()];
            let dataset = datasets[(i + (seed as usize % 3)) % datasets.len()];
            out.push(Format09Scenario {
                scenario_id: format!("{}_{}_{}_{}", regime, target, p, seed),
                dataset,
                metadata_regime: regime,
                metadata_target: target,
                metadata_operation: match regime {
                    "metadata_intact" => "none",
                    "metadata_destroyed" => "delete",
                    "metadata_partial" => "truncate",
                    "metadata_conflicting" => "overwrite",
                    _ => "bitflip",
                },
                payload_damage: p,
                seed,
            });
            seed += 1;
        }
    }
    out
}

fn clobber_range(bytes: &mut [u8], start: usize, len: usize, mode: &str) {
    let end = start.saturating_add(len).min(bytes.len());
    if start >= end {
        return;
    }
    match mode {
        "delete" => {
            for b in &mut bytes[start..end] {
                *b = 0;
            }
        }
        "truncate" => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                if i % 2 == 0 {
                    *b = 0;
                }
            }
        }
        "bitflip" => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                *b ^= 0xA5 ^ ((i % 17) as u8);
            }
        }
        _ => {
            for (i, b) in bytes[start..end].iter_mut().enumerate() {
                *b = 0x55u8.wrapping_add((i % 101) as u8);
            }
        }
    }
}

fn corrupt_metadata_for_format09(bytes: &mut Vec<u8>, scenario: &Format09Scenario) -> Result<()> {
    if scenario.metadata_regime == "metadata_intact" {
        return Ok(());
    }

    let len = bytes.len();
    let footer_offset = len.saturating_sub(FTR4_LEN);
    let footer = Ftr4::read_from(std::io::Cursor::new(&bytes[footer_offset..]))?;
    let blocks_end = footer.blocks_end_offset as usize;
    let index_offset = footer.index_offset as usize;

    match scenario.metadata_target {
        "index" => clobber_range(bytes, index_offset, 96, scenario.metadata_operation),
        "manifest" => {
            for needle in [
                b"crushr-file-manifest-checkpoint.v1".as_slice(),
                b"file-manifest-checkpoint".as_slice(),
            ] {
                let mut pos = 0usize;
                while let Some(hit) = bytes[pos..].windows(needle.len()).position(|w| w == needle) {
                    let start = pos + hit;
                    clobber_range(
                        bytes,
                        start.saturating_sub(8),
                        64,
                        scenario.metadata_operation,
                    );
                    pos = start + needle.len();
                }
            }
        }
        "path_checkpoint" => {
            for needle in [
                b"crushr-path-checkpoint.v1".as_slice(),
                b"path-checkpoint".as_slice(),
            ] {
                let mut pos = 0usize;
                while let Some(hit) = bytes[pos..].windows(needle.len()).position(|w| w == needle) {
                    let start = pos + hit;
                    clobber_range(
                        bytes,
                        start.saturating_sub(8),
                        64,
                        scenario.metadata_operation,
                    );
                    pos = start + needle.len();
                }
            }
        }
        "tail_metadata" => {
            let start = blocks_end.min(len.saturating_sub(16));
            clobber_range(
                bytes,
                start,
                len.saturating_sub(start + FTR4_LEN),
                scenario.metadata_operation,
            );
        }
        "multiple_metadata_layers" | "index_manifest" => {
            clobber_range(bytes, index_offset, 96, scenario.metadata_operation);
            let start = blocks_end.min(len.saturating_sub(16));
            clobber_range(
                bytes,
                start,
                len.saturating_sub(start + FTR4_LEN),
                scenario.metadata_operation,
            );
        }
        _ => {}
    }

    if scenario.metadata_regime == "metadata_conflicting" {
        let dup = bytes[blocks_end..len.saturating_sub(FTR4_LEN)].to_vec();
        let insert_at = (blocks_end + 64).min(len.saturating_sub(FTR4_LEN));
        bytes.splice(insert_at..insert_at, dup.into_iter().take(128));
    }

    Ok(())
}

fn corrupt_payload_for_format09(bytes: &mut Vec<u8>, scenario: &Format09Scenario) {
    let len = bytes.len();
    let data_start = 64usize.min(len);
    let data_end = len.saturating_sub(FTR4_LEN + 128);
    if data_start >= data_end {
        return;
    }
    let span = data_end - data_start;
    let block = (span / 16).max(64);
    let start = data_start + ((scenario.seed as usize % 7) * block / 2).min(span.saturating_sub(1));

    match scenario.payload_damage {
        "none" => {}
        "single_block_delete" => clobber_range(bytes, start, block, "delete"),
        "sparse_delete" => {
            for i in 0..4 {
                clobber_range(bytes, start + i * (block / 2), block / 3, "delete");
            }
        }
        "cluster_delete" => clobber_range(bytes, start, block * 3 / 2, "delete"),
        "block_reorder" => {
            let mid = (start + block).min(data_end.saturating_sub(block));
            if mid > start && mid + block <= data_end {
                let a = bytes[start..start + block].to_vec();
                let b = bytes[mid..mid + block].to_vec();
                bytes[start..start + block].copy_from_slice(&b);
                bytes[mid..mid + block].copy_from_slice(&a);
            }
        }
        "duplicate_block_insert" => {
            let end = (start + block).min(data_end);
            let dup = bytes[start..end].to_vec();
            let insert_at = (end + block / 2).min(data_end);
            bytes.splice(insert_at..insert_at, dup);
        }
        _ => {}
    }
}

fn apply_format09_corruption(archive_path: &Path, scenario: &Format09Scenario) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    corrupt_metadata_for_format09(&mut bytes, scenario)?;
    corrupt_payload_for_format09(&mut bytes, scenario);
    fs::write(archive_path, bytes)?;
    Ok(())
}

fn recovery_class_rank(classes: &BTreeMap<String, u64>) -> &'static str {
    if class_count(classes, &["FULL_NAMED_VERIFIED", "FULL_VERIFIED"]) > 0 {
        "named"
    } else if class_count(classes, &["FULL_ANONYMOUS_VERIFIED", "FULL_ANONYMOUS"]) > 0 {
        "anonymous"
    } else if class_count(classes, &["PARTIAL_ORDERED_VERIFIED", "PARTIAL_ORDERED"]) > 0 {
        "partial_ordered"
    } else if class_count(
        classes,
        &["PARTIAL_UNORDERED_VERIFIED", "PARTIAL_UNORDERED"],
    ) > 0
    {
        "partial_unordered"
    } else if class_count(classes, &["ORPHAN_EVIDENCE_ONLY", "ORPHAN_BLOCKS"]) > 0 {
        "orphan"
    } else {
        "none"
    }
}

fn class_count(classes: &BTreeMap<String, u64>, names: &[&str]) -> u64 {
    names
        .iter()
        .map(|name| classes.get(*name).copied().unwrap_or(0))
        .sum()
}

#[derive(Clone, Copy)]
enum TerminalRecoveryOutcome {
    Named,
    AnonymousFull,
    PartialOrdered,
    PartialUnordered,
    OrphanEvidence,
    NoVerifiedEvidence,
}

fn terminal_recovery_outcome(classes: &BTreeMap<String, u64>) -> TerminalRecoveryOutcome {
    if class_count(classes, &["FULL_NAMED_VERIFIED", "FULL_VERIFIED"]) > 0 {
        TerminalRecoveryOutcome::Named
    } else if class_count(classes, &["FULL_ANONYMOUS_VERIFIED", "FULL_ANONYMOUS"]) > 0 {
        TerminalRecoveryOutcome::AnonymousFull
    } else if class_count(classes, &["PARTIAL_ORDERED_VERIFIED", "PARTIAL_ORDERED"]) > 0 {
        TerminalRecoveryOutcome::PartialOrdered
    } else if class_count(
        classes,
        &["PARTIAL_UNORDERED_VERIFIED", "PARTIAL_UNORDERED"],
    ) > 0
    {
        TerminalRecoveryOutcome::PartialUnordered
    } else if class_count(classes, &["ORPHAN_EVIDENCE_ONLY", "ORPHAN_BLOCKS"]) > 0 {
        TerminalRecoveryOutcome::OrphanEvidence
    } else {
        TerminalRecoveryOutcome::NoVerifiedEvidence
    }
}

fn enforce_dictionary_fail_closed(
    variant: &str,
    terminal: TerminalRecoveryOutcome,
    dictionary_copy_count: usize,
    dictionary_conflict: bool,
) -> TerminalRecoveryOutcome {
    if !variant.starts_with("extent_identity_path_dict_") {
        return terminal;
    }
    if !dictionary_conflict && dictionary_copy_count > 0 {
        return terminal;
    }
    match terminal {
        TerminalRecoveryOutcome::NoVerifiedEvidence => TerminalRecoveryOutcome::NoVerifiedEvidence,
        TerminalRecoveryOutcome::OrphanEvidence => TerminalRecoveryOutcome::OrphanEvidence,
        TerminalRecoveryOutcome::PartialOrdered => TerminalRecoveryOutcome::PartialOrdered,
        TerminalRecoveryOutcome::PartialUnordered => TerminalRecoveryOutcome::PartialUnordered,
        TerminalRecoveryOutcome::Named | TerminalRecoveryOutcome::AnonymousFull => {
            TerminalRecoveryOutcome::AnonymousFull
        }
    }
}

fn expected_dictionary_state_for_scenario(variant: &str, corruption_target: &str) -> (usize, bool) {
    if !variant.starts_with("extent_identity_path_dict_") {
        return (0, false);
    }

    match (variant, corruption_target) {
        ("extent_identity_path_dict_header_tail", "primary_dictionary")
        | ("extent_identity_path_dict_header_tail", "mirrored_dictionary")
        | ("extent_identity_path_dict_factored_header_tail", "primary_dictionary")
        | ("extent_identity_path_dict_factored_header_tail", "mirrored_dictionary") => (1, false),
        ("extent_identity_path_dict_header_tail", "both_dictionaries")
        | ("extent_identity_path_dict_factored_header_tail", "both_dictionaries") => (0, false),
        ("extent_identity_path_dict_header_tail", "inconsistent_dictionaries")
        | ("extent_identity_path_dict_factored_header_tail", "inconsistent_dictionaries") => {
            (2, true)
        }
        ("extent_identity_path_dict_single", "inconsistent_dictionaries")
        | ("extent_identity_path_dict_single", "primary_dictionary")
        | ("extent_identity_path_dict_single", "mirrored_dictionary")
        | ("extent_identity_path_dict_single", "both_dictionaries") => (0, false),
        _ => (0, false),
    }
}

pub(super) fn run_format09_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp =
        std::env::temp_dir().join(format!("crushr-format09-comparison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let strategies = [
        FORMAT08_STRATEGY_FIXED,
        FORMAT08_STRATEGY_HASH,
        FORMAT08_STRATEGY_GOLDEN,
    ];
    let scenarios = format09_scenarios();
    let mut rows = Vec::new();

    for scenario in &scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for strategy in strategies {
            let archive = scenario_dir.join(format!("format09_{}.crushr", strategy));
            build_archive_with_pack_format08(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                strategy,
            )?;
            apply_format09_corruption(&archive, scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!("plan_{}_{}.json", strategy, scenario.scenario_id)),
            )?;
            let classes = recovery_classification_counts(&plan);
            let manifest_survival =
                metadata_node_count(&plan, "crushr-file-manifest-checkpoint.v1");
            let path_survival = metadata_node_count(&plan, "crushr-path-checkpoint.v1");
            let metadata_nodes = manifest_survival + path_survival;
            if verbose {
                eprintln!(
                    "format09 {} {} => class={}",
                    scenario.scenario_id,
                    strategy,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "strategy": strategy,
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "metadata_regime": scenario.metadata_regime,
                "metadata_target": scenario.metadata_target,
                "metadata_operation": scenario.metadata_operation,
                "payload_damage": scenario.payload_damage,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "manifest_checkpoint_survival_count": manifest_survival,
                "path_checkpoint_survival_count": path_survival,
                "verified_metadata_node_count": metadata_nodes,
                "recovery_class": recovery_class_rank(&classes),
                "metadata_recovery_gain": "pending",
            }));
        }
    }

    let mut destroyed_baseline = BTreeMap::<(String, String, String), String>::new();
    for row in &rows {
        if row["metadata_regime"] == "metadata_destroyed" {
            destroyed_baseline.insert(
                (
                    row["strategy"].as_str().unwrap_or("").to_string(),
                    row["metadata_target"].as_str().unwrap_or("").to_string(),
                    row["payload_damage"].as_str().unwrap_or("").to_string(),
                ),
                row["recovery_class"].as_str().unwrap_or("none").to_string(),
            );
        }
    }

    for row in &mut rows {
        let key = (
            row["strategy"].as_str().unwrap_or("").to_string(),
            row["metadata_target"].as_str().unwrap_or("").to_string(),
            row["payload_damage"].as_str().unwrap_or("").to_string(),
        );
        let current = row["recovery_class"].as_str().unwrap_or("none");
        let gain = if row["metadata_regime"] == "metadata_destroyed" {
            "baseline".to_string()
        } else if let Some(base) = destroyed_baseline.get(&key) {
            if base == current {
                "unchanged".to_string()
            } else {
                format!("{}->{}", base, current)
            }
        } else {
            "unknown".to_string()
        };
        row["metadata_recovery_gain"] = Value::String(gain);
    }

    let recovery_class_distribution = count_outcomes(
        rows.iter()
            .filter_map(|r| r.get("recovery_class").and_then(Value::as_str)),
    );
    let metadata_survival_stats = serde_json::json!({
        "manifest_checkpoint_survival_count": rows.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "path_checkpoint_survival_count": rows.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "verified_metadata_node_count": rows.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
    });
    let gain_distribution = count_outcomes(
        rows.iter()
            .filter_map(|r| r.get("metadata_recovery_gain").and_then(Value::as_str)),
    );

    let mut by_strategy = serde_json::Map::new();
    for strategy in strategies {
        let strategy_rows: Vec<Value> = rows
            .iter()
            .filter(|r| r["strategy"] == strategy)
            .cloned()
            .collect();
        by_strategy.insert(strategy.to_string(), serde_json::json!({
            "scenario_count": strategy_rows.len(),
            "recovery_class_distribution": count_outcomes(strategy_rows.iter().filter_map(|r| r["recovery_class"].as_str())),
            "metadata_survival": {
                "manifest_checkpoint_survival_count": strategy_rows.iter().map(|r| r["manifest_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
                "path_checkpoint_survival_count": strategy_rows.iter().map(|r| r["path_checkpoint_survival_count"].as_u64().unwrap_or(0)).sum::<u64>(),
                "verified_metadata_node_count": strategy_rows.iter().map(|r| r["verified_metadata_node_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            },
            "metadata_recovery_gain_distribution": count_outcomes(strategy_rows.iter().filter_map(|r| r["metadata_recovery_gain"].as_str())),
        }));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format09-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "metadata_regimes": ["metadata_intact", "metadata_destroyed", "metadata_partial", "metadata_conflicting"],
        "recovery_class_distribution": recovery_class_distribution,
        "metadata_survival_statistics": metadata_survival_stats,
        "metadata_recovery_gain_distribution": gain_distribution,
        "strategy_comparison": by_strategy,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format09_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-09 metadata survivability and necessity audit\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Scenario table\n\n");
    md.push_str("| strategy | scenario_id | metadata_regime | metadata_target | payload_damage | recovery_class | metadata_recovery_gain |\n");
    md.push_str("|---|---|---|---|---|---|---|\n");
    for row in summary["per_scenario_rows"]
        .as_array()
        .into_iter()
        .flatten()
        .take(80)
    {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            row["strategy"].as_str().unwrap_or(""),
            row["scenario_id"].as_str().unwrap_or(""),
            row["metadata_regime"].as_str().unwrap_or(""),
            row["metadata_target"].as_str().unwrap_or(""),
            row["payload_damage"].as_str().unwrap_or(""),
            row["recovery_class"].as_str().unwrap_or(""),
            row["metadata_recovery_gain"].as_str().unwrap_or(""),
        ));
    }
    md.push_str("\n## Recovery class distribution\n\n");
    for (k, v) in summary["recovery_class_distribution"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Metadata survival statistics\n\n");
    for (k, v) in summary["metadata_survival_statistics"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Recovery gain attributable to metadata\n\n");
    for (k, v) in summary["metadata_recovery_gain_distribution"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!("- {}: {}\n", k, v.as_u64().unwrap_or(0)));
    }
    md.push_str("\n## Strategy comparison\n\n");
    for (k, v) in summary["strategy_comparison"]
        .as_object()
        .into_iter()
        .flatten()
    {
        md.push_str(&format!(
            "- {}: scenarios={}\n",
            k,
            v["scenario_count"].as_u64().unwrap_or(0)
        ));
    }

    fs::write(comparison_dir.join("format09_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
enum Format10Variant {
    PayloadOnly,
    PayloadPlusManifest,
    PayloadPlusPath,
    FullCurrentExperimental,
}

impl Format10Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::PayloadPlusPath => "payload_plus_path",
            Self::FullCurrentExperimental => "full_current_experimental",
        }
    }

    fn metadata_profile(self) -> &'static str {
        self.as_str()
    }
}

fn build_archive_with_pack_metadata_profile(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
    variant: Format10Variant,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--metadata-profile")
        .arg(variant.metadata_profile())
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn estimate_metadata_byte_size(archive_path: &Path) -> Result<u64> {
    let bytes = fs::read(archive_path)?;
    let mut offset = 0usize;
    let mut total = 0u64;
    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] != BLK3_MAGIC {
            offset += 1;
            continue;
        }
        let header = match read_blk3_header(std::io::Cursor::new(&bytes[offset..])) {
            Ok(h) => h,
            Err(_) => {
                offset += 1;
                continue;
            }
        };
        let block_len = header.header_len as usize + header.comp_len as usize;
        if block_len == 0 || offset + block_len > bytes.len() {
            offset += 1;
            continue;
        }
        if header.flags.is_meta_frame() {
            total = total.saturating_add(block_len as u64);
        }
        offset += block_len;
    }
    Ok(total)
}

pub(super) fn run_format10_pruning_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format10-pruning-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format10Variant::FullCurrentExperimental,
        Format10Variant::PayloadOnly,
        Format10Variant::PayloadPlusManifest,
        Format10Variant::PayloadPlusPath,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format10_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            if verbose {
                eprintln!(
                    "format10 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
                    recovery_class_rank(&classes)
                );
            }
            rows.push(serde_json::json!({
                "variant": variant_name,
                "scenario_id": scenario.scenario_id,
                "dataset": scenario.dataset,
                "corruption_model": scenario.corruption_model,
                "corruption_target": scenario.corruption_target,
                "magnitude": scenario.magnitude,
                "seed": scenario.seed,
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.get("NO_VERIFIED_EVIDENCE").copied().unwrap_or(0) > 0,
            }));
        }
    }

    let payload_only_total_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_only")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();

    let full_outcomes = count_outcomes(
        rows.iter()
            .filter(|r| r["variant"] == "full_current_experimental")
            .filter_map(|r| r["recovery_outcome"].as_str()),
    );

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "full_current_experimental",
        "payload_only",
        "payload_plus_manifest",
        "payload_plus_path",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let recovery_outcomes = count_outcomes(
            variant_rows
                .iter()
                .filter_map(|r| r["recovery_outcome"].as_str()),
        );
        let classes = merge_classification_counts(
            &variant_rows
                .iter()
                .map(|r| (*r).clone())
                .collect::<Vec<Value>>(),
            "recovery_classification_counts",
        );
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let overhead_delta_vs_payload_only =
            archive_byte_size as i64 - payload_only_total_size as i64;

        let baseline_full = full_outcomes
            .get("FULL_FILE_SALVAGE_AVAILABLE")
            .copied()
            .unwrap_or(0) as i64;
        let this_full = recovery_outcomes
            .get("FULL_FILE_SALVAGE_AVAILABLE")
            .copied()
            .unwrap_or(0) as i64;

        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": recovery_outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": variant_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": overhead_delta_vs_payload_only,
            "recovery_delta_vs_full_current_experimental": {
                "full_file_salvage_available_delta": this_full - baseline_full,
            }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "full_current_experimental",
                "payload_only",
                "payload_plus_manifest",
                "payload_plus_path",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "recovery_outcome_counts": count_outcomes(g_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                    "metadata_byte_estimate": g_rows.iter().map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format10-pruning-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["full_current_experimental", "payload_only", "payload_plus_manifest", "payload_plus_path"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format10_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-10 metadata pruning comparison\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anonymous_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | metadata_byte_estimate | overhead_vs_payload_only |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "full_current_experimental",
        "payload_only",
        "payload_plus_manifest",
        "payload_plus_path",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_u64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["metadata_byte_estimate"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
        ));
    }

    fs::write(comparison_dir.join("format10_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
enum Format11Variant {
    PayloadOnly,
    PayloadPlusManifest,
    FullCurrentExperimental,
    ExtentIdentityOnly,
}

impl Format11Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::FullCurrentExperimental => "full_current_experimental",
            Self::ExtentIdentityOnly => "extent_identity_only",
        }
    }

    fn metadata_profile(self) -> &'static str {
        self.as_str()
    }
}

fn build_archive_with_pack_metadata_profile_name(
    pack_bin: &Path,
    input: &Path,
    output: &Path,
    profile: &str,
) -> Result<()> {
    let out = Command::new(pack_bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--level")
        .arg("3")
        .arg("--metadata-profile")
        .arg(profile)
        .output()
        .with_context(|| format!("run {:?}", pack_bin))?;
    if !out.status.success() {
        bail!(
            "crushr-pack failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

pub(super) fn run_format11_extent_identity_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format11-extent-identity-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format11Variant::PayloadOnly,
        Format11Variant::PayloadPlusManifest,
        Format11Variant::FullCurrentExperimental,
        Format11Variant::ExtentIdentityOnly,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format11_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant.metadata_profile(),
            )?;

            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan11_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            if verbose {
                eprintln!(
                    "format11 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
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
                "variant": variant_name,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.is_empty(),
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
            }));
        }
    }

    let payload_only_total_size: u64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_only")
        .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
        .sum();

    let payload_plus_manifest_named: i64 = rows
        .iter()
        .filter(|r| r["variant"] == "payload_plus_manifest")
        .filter(|r| r["named_recovery"].as_bool() == Some(true))
        .count() as i64;

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "payload_only",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_only",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let recovery_outcomes = count_outcomes(
            variant_rows
                .iter()
                .filter_map(|r| r["recovery_outcome"].as_str()),
        );
        let mut classes = BTreeMap::new();
        for row in &variant_rows {
            if let Some(obj) = row["recovery_classification_counts"].as_object() {
                for (k, v) in obj {
                    let n = v.as_u64().unwrap_or(0);
                    *classes.entry(k.clone()).or_insert(0) += n;
                }
            }
        }
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let overhead_delta_vs_payload_only =
            archive_byte_size as i64 - payload_only_total_size as i64;
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;

        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": recovery_outcomes,
            "recovery_classification_counts": classes,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": overhead_delta_vs_payload_only,
            "recovery_delta_vs_payload_plus_manifest": {
                "named_recovery_count_delta": named_count - payload_plus_manifest_named,
            }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "payload_only",
                "payload_plus_manifest",
                "full_current_experimental",
                "extent_identity_only",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "recovery_outcome_counts": count_outcomes(g_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "anonymous_full_recovery_count": g_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
                    "partial_ordered_recovery_count": g_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
                    "partial_unordered_recovery_count": g_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
                    "orphan_evidence_count": g_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
                    "no_verified_evidence_count": g_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format11-extent-identity-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["payload_only", "payload_plus_manifest", "full_current_experimental", "extent_identity_only"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format11_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-11 extent identity comparison\n\n");
    md.push_str(&format!(
        "Scenarios: {}\n\n",
        summary["scenario_count"].as_u64().unwrap_or(0)
    ));
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anonymous_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | metadata_byte_estimate | overhead_vs_payload_only | named_delta_vs_payload_plus_manifest |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "payload_only",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_only",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["metadata_byte_estimate"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["recovery_delta_vs_payload_plus_manifest"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
        ));
    }

    fs::write(comparison_dir.join("format11_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[derive(Clone, Copy)]
enum Format12Variant {
    PayloadOnly,
    ExtentIdentityOnly,
    ExtentIdentityDistributedNames,
    PayloadPlusManifest,
    FullCurrentExperimental,
    ExtentIdentityInlinePath,
}

impl Format12Variant {
    fn as_str(self) -> &'static str {
        match self {
            Self::PayloadOnly => "payload_only",
            Self::ExtentIdentityOnly => "extent_identity_only",
            Self::ExtentIdentityDistributedNames => "extent_identity_distributed_names",
            Self::PayloadPlusManifest => "payload_plus_manifest",
            Self::FullCurrentExperimental => "full_current_experimental",
            Self::ExtentIdentityInlinePath => "extent_identity_inline_path",
        }
    }
}

fn write_dataset_fixture_format12(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input).with_context(|| format!("create {}", input.display()))?;

    match dataset {
        "smallfiles" => {
            fs::write(input.join("tiny.txt"), b"small-dataset-payload")?;
            fs::write(input.join("a.txt"), b"short")?;
        }
        "mixed" => {
            fs::write(
                input.join("mixed.bin"),
                (0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
            )?;
            let long_path = input
                .join("nested")
                .join("deeply")
                .join("named")
                .join("path")
                .join("for")
                .join("inline")
                .join("duplication")
                .join("cost")
                .join("visibility")
                .join("sample-long-name.txt");
            if let Some(parent) = long_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(long_path, b"long-path-payload")?;
        }
        "largefiles" => {
            fs::write(input.join("large.dat"), vec![13u8; 8192])?;
        }
        _ => bail!("unsupported dataset {dataset}"),
    }

    Ok(())
}

pub(super) fn run_format12_inline_path_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format12-inline-path-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        Format12Variant::PayloadOnly,
        Format12Variant::ExtentIdentityOnly,
        Format12Variant::ExtentIdentityDistributedNames,
        Format12Variant::PayloadPlusManifest,
        Format12Variant::FullCurrentExperimental,
        Format12Variant::ExtentIdentityInlinePath,
    ];
    let scenarios = comparison_scenarios();
    let mut rows = Vec::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_dir = scenario_dir.join("input");
        write_dataset_fixture_format12(&input_dir, scenario.dataset)?;

        for variant in variants {
            let variant_name = variant.as_str();
            let archive = scenario_dir.join(format!("format12_{}.crushr", variant_name));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin,
                &input_dir.join(scenario.dataset),
                &archive,
                variant_name,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();
            let metadata_byte_estimate = estimate_metadata_byte_size(&archive)?;
            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan12_{}_{}.json",
                    variant_name, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            let path_bucket = if scenario.dataset == "mixed" {
                "long_path"
            } else {
                "short_path"
            };
            if verbose {
                eprintln!(
                    "format12 {} {} => class={}",
                    scenario.scenario_id,
                    variant_name,
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
                "path_length_bucket": path_bucket,
                "variant": variant_name,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0,
                "anonymous_full_recovery": classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_ordered_recovery": classes.get("PARTIAL_ORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "partial_unordered_recovery": classes.get("PARTIAL_UNORDERED_VERIFIED").copied().unwrap_or(0) > 0,
                "orphan_evidence": classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0,
                "no_verified_evidence": classes.is_empty(),
                "archive_byte_size": archive_byte_size,
                "metadata_byte_estimate": metadata_byte_estimate,
            }));
        }
    }

    let totals_for = |name: &str| -> (u64, i64) {
        let size = rows
            .iter()
            .filter(|r| r["variant"] == name)
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named = rows
            .iter()
            .filter(|r| r["variant"] == name && r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        (size, named)
    };
    let (payload_only_size, _) = totals_for("payload_only");
    let (extent_identity_only_size, extent_identity_only_named) =
        totals_for("extent_identity_only");
    let (payload_plus_manifest_size, payload_plus_manifest_named) =
        totals_for("payload_plus_manifest");

    let mut by_variant = serde_json::Map::new();
    for variant in [
        "payload_only",
        "extent_identity_only",
        "extent_identity_distributed_names",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_inline_path",
    ] {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let metadata_byte_estimate: u64 = variant_rows
            .iter()
            .map(|r| r["metadata_byte_estimate"].as_u64().unwrap_or(0))
            .sum();
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        let mut classes = BTreeMap::new();
        for row in &variant_rows {
            if let Some(obj) = row["recovery_classification_counts"].as_object() {
                for (k, v) in obj {
                    *classes.entry(k.clone()).or_insert(0) += v.as_u64().unwrap_or(0);
                }
            }
        }
        let named_delta_vs_extent = named_count - extent_identity_only_named;
        let overhead_vs_extent = archive_byte_size as i64 - extent_identity_only_size as i64;
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "recovery_outcome_counts": count_outcomes(variant_rows.iter().filter_map(|r| r["recovery_outcome"].as_str())),
            "recovery_classification_counts": classes,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "archive_byte_size": archive_byte_size,
            "metadata_byte_estimate": metadata_byte_estimate,
            "overhead_delta_vs_payload_only": archive_byte_size as i64 - payload_only_size as i64,
            "overhead_delta_vs_extent_identity_only": overhead_vs_extent,
            "overhead_delta_vs_payload_plus_manifest": archive_byte_size as i64 - payload_plus_manifest_size as i64,
            "recovery_delta_vs_extent_identity_only": {"named_recovery_count_delta": named_delta_vs_extent},
            "recovery_delta_vs_payload_plus_manifest": {"named_recovery_count_delta": named_count - payload_plus_manifest_named},
            "recovery_per_kib_overhead": if overhead_vs_extent > 0 { (named_delta_vs_extent as f64) / (overhead_vs_extent as f64 / 1024.0) } else { 0.0 },
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in ["dataset", "corruption_target", "path_length_bucket"] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in [
                "payload_only",
                "extent_identity_only",
                "extent_identity_distributed_names",
                "payload_plus_manifest",
                "full_current_experimental",
                "extent_identity_inline_path",
            ] {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "archive_byte_size": g_rows.iter().map(|r| r["archive_byte_size"].as_u64().unwrap_or(0)).sum::<u64>(),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format12-inline-path-comparison.v1",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "variants": ["payload_only", "extent_identity_only", "extent_identity_distributed_names", "payload_plus_manifest", "full_current_experimental", "extent_identity_inline_path"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    fs::write(
        comparison_dir.join("format12_comparison_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Format-12 inline path comparison\n\n");
    md.push_str("## Variant summary\n\n");
    md.push_str("| variant | scenarios | named | anon_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | overhead_vs_payload_plus_manifest | named_delta_vs_extent_identity_only | named_delta_vs_payload_plus_manifest | recovery_per_kib_overhead |\n");
    md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for variant in [
        "payload_only",
        "extent_identity_only",
        "extent_identity_distributed_names",
        "payload_plus_manifest",
        "full_current_experimental",
        "extent_identity_inline_path",
    ] {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.6} |\n",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"]
                .as_u64()
                .unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["overhead_delta_vs_extent_identity_only"]
                .as_i64()
                .unwrap_or(0),
            row["overhead_delta_vs_payload_plus_manifest"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_delta_vs_extent_identity_only"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_delta_vs_payload_plus_manifest"]["named_recovery_count_delta"]
                .as_i64()
                .unwrap_or(0),
            row["recovery_per_kib_overhead"].as_f64().unwrap_or(0.0),
        ));
    }
    md.push_str("\n## Explicit judgment\n\n");
    md.push_str("- `extent_identity_inline_path` is credible for compression-oriented use only if its overhead remains materially below `payload_plus_manifest` while preserving similar named recovery.\n");
    md.push_str("- Use the table above to determine whether overhead is closer to `extent_identity_only`, `payload_plus_manifest`, or `full_current_experimental`.\n");
    md.push_str("- If named-recovery gain is small relative to added bytes, this variant should be treated as evidence for a more compact distributed naming design rather than immediate adoption.\n");
    fs::write(comparison_dir.join("format12_comparison_summary.md"), md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn make_segment(seed: usize, len: usize) -> String {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len)
        .map(|i| alphabet[(seed + i) % alphabet.len()] as char)
        .collect()
}

fn write_dataset_fixture_format12_stress(root: &Path, dataset: &str) -> Result<()> {
    let input = root.join(dataset);
    fs::create_dir_all(&input)?;
    match dataset {
        "deep_paths" => {
            for i in 0..320usize {
                let mut dir = input.clone();
                for d in 0..20usize {
                    dir = dir.join(format!("l{:02}_{}", d, make_segment(i + d * 7, 5)));
                }
                fs::create_dir_all(&dir)?;
                let file_name = format!("file_{:04}_{}.bin", i, make_segment(i * 11, 16));
                fs::write(
                    dir.join(file_name),
                    vec![(i % 251) as u8; 1536 + (i % 7) * 64],
                )?;
            }
        }
        "long_names" => {
            for i in 0..200usize {
                let name = format!(
                    "very_long_filename_{}_{}_{}.bin",
                    make_segment(i * 3, 90),
                    make_segment(i * 7 + 13, 42),
                    i
                );
                fs::write(
                    input.join(name),
                    vec![(i % 241) as u8; 2048 + (i % 5) * 128],
                )?;
            }
        }
        "fragmentation_heavy" => {
            for logical in 0..20usize {
                let dir = input.join(format!("logical_{:02}", logical));
                fs::create_dir_all(&dir)?;
                for frag in 0..48usize {
                    let name = format!("logical_{:02}__frag_{:03}.bin", logical, frag);
                    let size = 768 + ((logical * 13 + frag * 29) % 512);
                    fs::write(dir.join(name), vec![((logical + frag) % 251) as u8; size])?;
                }
            }
        }
        "mixed_worst_case" => {
            for logical in 0..20usize {
                let mut dir = input.clone();
                for d in 0..20usize {
                    dir = dir.join(format!("mx{:02}_{}", d, make_segment(logical + d * 5, 4)));
                }
                fs::create_dir_all(&dir)?;
                for frag in 0..48usize {
                    let name = format!(
                        "logical_{:02}_fragment_{:03}_{}.bin",
                        logical,
                        frag,
                        make_segment(logical * 17 + frag * 19, 120)
                    );
                    let size = 640 + ((logical * 31 + frag * 7) % 256);
                    fs::write(
                        dir.join(name),
                        vec![((logical * 3 + frag) % 253) as u8; size],
                    )?;
                }
            }
        }
        _ => bail!("unsupported stress dataset {dataset}"),
    }
    Ok(())
}

fn parse_metadata_json_blocks(archive_path: &Path) -> Result<Vec<Value>> {
    Ok(parse_metadata_json_blocks_with_offsets(archive_path)?
        .into_iter()
        .map(|row| row.value)
        .collect())
}

#[derive(Clone)]
struct MetadataJsonBlock {
    block_start: usize,
    payload_start: usize,
    payload_end: usize,
    header: Blk3Header,
    value: Value,
}

fn parse_metadata_json_blocks_with_offsets(archive_path: &Path) -> Result<Vec<MetadataJsonBlock>> {
    let bytes = fs::read(archive_path)?;
    let mut offset = 0usize;
    let mut rows = Vec::new();
    while offset + BLK3_MAGIC.len() <= bytes.len() {
        if bytes[offset..offset + 4] != BLK3_MAGIC {
            offset += 1;
            continue;
        }
        let header = match read_blk3_header(std::io::Cursor::new(&bytes[offset..])) {
            Ok(h) => h,
            Err(_) => {
                offset += 1;
                continue;
            }
        };
        let block_len = header.header_len as usize + header.comp_len as usize;
        if block_len == 0 || offset + block_len > bytes.len() {
            offset += 1;
            continue;
        }
        if header.flags.is_meta_frame() {
            let payload_start = offset + header.header_len as usize;
            let payload_end = offset + block_len;
            let decoded =
                zstd::decode_all(std::io::Cursor::new(&bytes[payload_start..payload_end]))?;
            if let Ok(v) = serde_json::from_slice::<Value>(&decoded) {
                rows.push(MetadataJsonBlock {
                    block_start: offset,
                    payload_start,
                    payload_end,
                    header,
                    value: v,
                });
            }
        }
        offset += block_len;
    }
    Ok(rows)
}

fn corrupt_dictionary_block_payload(archive_path: &Path, block: &MetadataJsonBlock) -> Result<()> {
    let mut bytes = fs::read(archive_path)?;
    for byte in &mut bytes[block.payload_start..block.payload_end] {
        *byte ^= 0xA7;
    }
    fs::write(archive_path, bytes)?;
    Ok(())
}

fn to_hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

fn rewrite_dictionary_block_as_inconsistent(
    archive_path: &Path,
    block: &MetadataJsonBlock,
) -> Result<()> {
    let mut mutated = block.value.clone();
    if let Some(first) = mutated
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .and_then(|entries| entries.first_mut())
    {
        let original = first
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("dict_entry");
        let new_path = format!("{original}.conflict");
        first["path"] = Value::from(new_path.clone());
        first["path_digest_blake3"] =
            Value::from(to_hex_lower(blake3::hash(new_path.as_bytes()).as_bytes()));
    } else if let Some(first) = mutated
        .get_mut("body")
        .and_then(|b| b.get_mut("file_bindings"))
        .and_then(Value::as_array_mut)
        .and_then(|entries| entries.first_mut())
    {
        first["path_digest_blake3"] = Value::from("00".repeat(32));
    }

    if let Some(body) = mutated.get("body") {
        let raw_body = serde_json::to_vec(body)?;
        mutated["dictionary_content_hash"] =
            Value::from(to_hex_lower(blake3::hash(&raw_body).as_bytes()));
        mutated["dictionary_length"] = Value::from(raw_body.len() as u64);
    }

    let raw = serde_json::to_vec(&mutated)?;
    let compressed = zstd::encode_all(std::io::Cursor::new(&raw), 3)?;

    let mut header = block.header.clone();
    header.raw_len = raw.len() as u64;
    header.comp_len = compressed.len() as u64;
    if header.flags.has_payload_hash() {
        header.payload_hash = Some(*blake3::hash(&compressed).as_bytes());
    }
    if header.flags.has_raw_hash() {
        header.raw_hash = Some(*blake3::hash(&raw).as_bytes());
    }

    let mut header_bytes = Vec::new();
    write_blk3_header(&mut header_bytes, &header)?;

    let bytes = fs::read(archive_path)?;
    let block_end = block.payload_end;
    let mut rewritten = Vec::with_capacity(bytes.len() + compressed.len());
    rewritten.extend_from_slice(&bytes[..block.block_start]);
    rewritten.extend_from_slice(&header_bytes);
    rewritten.extend_from_slice(&compressed);
    rewritten.extend_from_slice(&bytes[block_end..]);
    fs::write(archive_path, rewritten)?;
    Ok(())
}

fn logical_key_for_path(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    if let Some(idx) = name.find("__frag_") {
        return name[..idx].to_string();
    }
    if let Some(idx) = name.find("_fragment_") {
        return name[..idx].to_string();
    }
    path.to_string()
}

pub(super) fn run_format12_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    fs::create_dir_all(comparison_dir)?;
    let temp = std::env::temp_dir().join(format!(
        "crushr-format12-stress-comparison-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

    let salvage_bin = resolve_salvage_bin()?;
    let pack_bin = resolve_pack_bin()?;
    let variants = [
        "payload_only",
        "extent_identity_only",
        "extent_identity_inline_path",
        "payload_plus_manifest",
    ];
    let scenarios = format12_stress_scenarios();

    let mut rows = Vec::new();
    let mut base_stats: std::collections::BTreeMap<(String, String), Value> =
        std::collections::BTreeMap::new();

    for scenario in scenarios {
        let scenario_dir = temp.join(&scenario.scenario_id);
        fs::create_dir_all(&scenario_dir)?;
        let input_root = scenario_dir.join("input");
        write_dataset_fixture_format12_stress(&input_root, scenario.dataset)?;
        let input_dir = input_root.join(scenario.dataset);

        for variant in variants {
            let archive =
                scenario_dir.join(format!("stress_{}_{}.crushr", scenario.dataset, variant));
            build_archive_with_pack_metadata_profile_name(
                &pack_bin, &input_dir, &archive, variant,
            )?;
            let archive_byte_size = fs::metadata(&archive)?.len();

            let key = (scenario.dataset.to_string(), variant.to_string());
            let stats = if let Some(existing) = base_stats.get(&key) {
                existing.clone()
            } else {
                let computed = compute_stress_identity_stats(&archive, &input_dir)?;
                base_stats.insert(key.clone(), computed.clone());
                computed
            };
            let avg_path = stats["average_path_length"].as_f64().unwrap_or(0.0);
            let avg_extents = stats["average_extents_per_file"].as_f64().unwrap_or(0.0);
            let path_length_bucket = if avg_path >= 180.0 {
                "very_long"
            } else if avg_path >= 120.0 {
                "long"
            } else {
                "normal"
            };
            let extent_density_bucket = if avg_extents >= 32.0 {
                "extreme_fragmentation"
            } else if avg_extents >= 8.0 {
                "fragmented"
            } else {
                "low_fragmentation"
            };

            let variant_scenario = ComparisonScenario {
                scenario_id: scenario.scenario_id.clone(),
                dataset: scenario.dataset,
                corruption_model: scenario.corruption_model,
                corruption_target: scenario.corruption_target,
                magnitude: scenario.magnitude,
                seed: scenario.seed,
                break_redundant_map: false,
            };
            corrupt_archive(&archive, &variant_scenario)?;
            let plan = run_salvage_plan(
                &salvage_bin,
                &archive,
                &scenario_dir.join(format!(
                    "plan12_stress_{}_{}.json",
                    variant, scenario.scenario_id
                )),
            )?;
            let classes = recovery_classification_counts(&plan);
            let outcome = outcome_from_plan(&plan);
            let named = classes.get("FULL_NAMED_VERIFIED").copied().unwrap_or(0) > 0;
            let anonymous_full = classes.get("FULL_ANONYMOUS_VERIFIED").copied().unwrap_or(0) > 0;
            let partial_ordered = classes
                .get("PARTIAL_ORDERED_VERIFIED")
                .copied()
                .unwrap_or(0)
                > 0;
            let partial_unordered = classes
                .get("PARTIAL_UNORDERED_VERIFIED")
                .copied()
                .unwrap_or(0)
                > 0;
            let orphan_evidence = classes.get("ORPHAN_EVIDENCE_ONLY").copied().unwrap_or(0) > 0;
            let no_verified_evidence = classes.is_empty();

            if verbose {
                eprintln!(
                    "format12-stress {} {} => class={} extents={} avg_path={:.2}",
                    scenario.scenario_id,
                    variant,
                    recovery_class_rank(&classes),
                    stats["total_extent_count"].as_u64().unwrap_or(0),
                    avg_path
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
                "path_length_bucket": path_length_bucket,
                "extent_density_bucket": extent_density_bucket,
                "recovery_outcome": outcome.outcome,
                "recovery_classification_counts": classes,
                "named_recovery": named,
                "anonymous_full_recovery": anonymous_full,
                "partial_ordered_recovery": partial_ordered,
                "partial_unordered_recovery": partial_unordered,
                "orphan_evidence": orphan_evidence,
                "no_verified_evidence": no_verified_evidence,
                "archive_byte_size": archive_byte_size,
                "average_path_length": stats["average_path_length"],
                "max_path_length": stats["max_path_length"],
                "total_extent_count": stats["total_extent_count"],
                "average_extents_per_file": stats["average_extents_per_file"],
                "max_extents_per_file": stats["max_extents_per_file"],
                "path_char_count": stats["path_char_count"],
                "extents_per_file_distribution": stats["extents_per_file_distribution"],
            }));
        }
    }

    let totals_for = |name: &str| -> (u64, i64) {
        let size = rows
            .iter()
            .filter(|r| r["variant"] == name)
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named = rows
            .iter()
            .filter(|r| r["variant"] == name && r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        (size, named)
    };
    let (payload_only_size, _) = totals_for("payload_only");
    let (extent_identity_only_size, extent_identity_only_named) =
        totals_for("extent_identity_only");

    let mut by_variant = serde_json::Map::new();
    for variant in variants {
        let variant_rows: Vec<&Value> = rows.iter().filter(|r| r["variant"] == variant).collect();
        let archive_byte_size: u64 = variant_rows
            .iter()
            .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
            .sum();
        let named_count = variant_rows
            .iter()
            .filter(|r| r["named_recovery"].as_bool() == Some(true))
            .count() as i64;
        let overhead_vs_extent = archive_byte_size as i64 - extent_identity_only_size as i64;
        let total_extents: u64 = variant_rows
            .iter()
            .map(|r| r["total_extent_count"].as_u64().unwrap_or(0))
            .sum();
        let total_path_chars: u64 = variant_rows
            .iter()
            .map(|r| r["path_char_count"].as_u64().unwrap_or(0))
            .sum();
        by_variant.insert(variant.to_string(), serde_json::json!({
            "scenario_count": variant_rows.len(),
            "archive_byte_size": archive_byte_size,
            "overhead_delta_vs_payload_only": archive_byte_size as i64 - payload_only_size as i64,
            "overhead_delta_vs_extent_identity_only": overhead_vs_extent,
            "named_recovery_count": named_count,
            "anonymous_full_recovery_count": variant_rows.iter().filter(|r| r["anonymous_full_recovery"].as_bool() == Some(true)).count(),
            "partial_ordered_recovery_count": variant_rows.iter().filter(|r| r["partial_ordered_recovery"].as_bool() == Some(true)).count(),
            "partial_unordered_recovery_count": variant_rows.iter().filter(|r| r["partial_unordered_recovery"].as_bool() == Some(true)).count(),
            "orphan_evidence_count": variant_rows.iter().filter(|r| r["orphan_evidence"].as_bool() == Some(true)).count(),
            "no_verified_evidence_count": variant_rows.iter().filter(|r| r["no_verified_evidence"].as_bool() == Some(true)).count(),
            "recovery_per_kib_overhead": if overhead_vs_extent > 0 { (named_count - extent_identity_only_named) as f64 / (overhead_vs_extent as f64 / 1024.0) } else { 0.0 },
            "average_path_length": mean_f64(&variant_rows, "average_path_length"),
            "max_path_length": max_u64(&variant_rows, "max_path_length"),
            "total_extent_count": total_extents,
            "average_extents_per_file": mean_f64(&variant_rows, "average_extents_per_file"),
            "max_extents_per_file": max_u64(&variant_rows, "max_extents_per_file"),
            "bytes_added_per_extent_vs_extent_identity_only": if total_extents > 0 { overhead_vs_extent as f64 / total_extents as f64 } else { 0.0 },
            "bytes_added_per_path_character_vs_extent_identity_only": if total_path_chars > 0 { overhead_vs_extent as f64 / total_path_chars as f64 } else { 0.0 }
        }));
    }

    let mut grouped = serde_json::Map::new();
    for group_field in [
        "dataset",
        "corruption_target",
        "path_length_bucket",
        "extent_density_bucket",
    ] {
        let mut group_map = serde_json::Map::new();
        let mut keys = std::collections::BTreeSet::new();
        for row in &rows {
            if let Some(k) = row[group_field].as_str() {
                keys.insert(k.to_string());
            }
        }
        for key in keys {
            let mut variant_map = serde_json::Map::new();
            for variant in variants {
                let g_rows: Vec<&Value> = rows
                    .iter()
                    .filter(|r| r["variant"] == variant && r[group_field] == key)
                    .collect();
                let archive_byte_size: u64 = g_rows
                    .iter()
                    .map(|r| r["archive_byte_size"].as_u64().unwrap_or(0))
                    .sum();
                variant_map.insert(variant.to_string(), serde_json::json!({
                    "scenario_count": g_rows.len(),
                    "archive_byte_size": archive_byte_size,
                    "named_recovery_count": g_rows.iter().filter(|r| r["named_recovery"].as_bool() == Some(true)).count(),
                    "average_path_length": mean_f64(&g_rows, "average_path_length"),
                    "average_extents_per_file": mean_f64(&g_rows, "average_extents_per_file"),
                }));
            }
            group_map.insert(key, Value::Object(variant_map));
        }
        grouped.insert(group_field.to_string(), Value::Object(group_map));
    }

    let summary = serde_json::json!({
        "schema_version": "crushr-lab-salvage-format12-stress-comparison.v2",
        "tool": "crushr-lab-salvage",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "verification_label": VERIFICATION_LABEL,
        "scenario_count": rows.len(),
        "deterministic_seed_start": 9100u64,
        "variants": variants,
        "datasets": ["deep_paths", "long_names", "fragmentation_heavy", "mixed_worst_case"],
        "by_variant": by_variant,
        "grouped_breakdown": grouped,
        "per_scenario_rows": rows,
    });

    let summary_json = serde_json::to_string_pretty(&summary)?;
    fs::write(
        comparison_dir.join("format12_stress_comparison_summary.json"),
        &summary_json,
    )?;
    // legacy compatibility path used by existing notes/tests.
    fs::write(
        comparison_dir.join("format12_stress_summary.json"),
        &summary_json,
    )?;

    let mut md = String::new();
    md.push_str(
        "# Format-12 stress comparison

",
    );
    md.push_str(
        "## Variant summary

",
    );
    md.push_str("| variant | scenarios | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | named | anon_full | partial_ordered | partial_unordered | orphan | none | avg_path | max_path | total_extents | avg_extents_per_file | max_extents_per_file | bytes_added_per_extent_vs_extent_identity_only | bytes_added_per_path_character_vs_extent_identity_only |
");
    md.push_str(
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
",
    );
    for variant in variants {
        let row = &summary["by_variant"][variant];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {} | {} | {:.2} | {} | {:.6} | {:.6} |
",
            variant,
            row["scenario_count"].as_u64().unwrap_or(0),
            row["archive_byte_size"].as_u64().unwrap_or(0),
            row["overhead_delta_vs_payload_only"].as_i64().unwrap_or(0),
            row["overhead_delta_vs_extent_identity_only"].as_i64().unwrap_or(0),
            row["named_recovery_count"].as_i64().unwrap_or(0),
            row["anonymous_full_recovery_count"].as_u64().unwrap_or(0),
            row["partial_ordered_recovery_count"].as_u64().unwrap_or(0),
            row["partial_unordered_recovery_count"].as_u64().unwrap_or(0),
            row["orphan_evidence_count"].as_u64().unwrap_or(0),
            row["no_verified_evidence_count"].as_u64().unwrap_or(0),
            row["average_path_length"].as_f64().unwrap_or(0.0),
            row["max_path_length"].as_u64().unwrap_or(0),
            row["total_extent_count"].as_u64().unwrap_or(0),
            row["average_extents_per_file"].as_f64().unwrap_or(0.0),
            row["max_extents_per_file"].as_u64().unwrap_or(0),
            row["bytes_added_per_extent_vs_extent_identity_only"].as_f64().unwrap_or(0.0),
            row["bytes_added_per_path_character_vs_extent_identity_only"].as_f64().unwrap_or(0.0)
        ));
    }

    let inline = &summary["by_variant"]["extent_identity_inline_path"];
    let manifest = &summary["by_variant"]["payload_plus_manifest"];
    let inline_overhead = inline["overhead_delta_vs_extent_identity_only"]
        .as_i64()
        .unwrap_or(0);
    let manifest_overhead = manifest["overhead_delta_vs_extent_identity_only"]
        .as_i64()
        .unwrap_or(0);
    let fragmentation_cost = inline["bytes_added_per_extent_vs_extent_identity_only"]
        .as_f64()
        .unwrap_or(0.0);

    md.push_str(
        "
## Judgment

",
    );
    md.push_str(&format!(
        "1. Did inline naming overhead materially increase under stress? **{}** (inline overhead vs extent_identity_only = {} bytes across all stress scenarios).
",
        if inline_overhead > 0 { "Yes" } else { "No" },
        inline_overhead
    ));
    md.push_str(&format!(
        "2. Does `extent_identity_inline_path` remain much smaller than `payload_plus_manifest`? **{}** ({} vs {} bytes overhead vs extent_identity_only).
",
        if inline_overhead < manifest_overhead { "Yes" } else { "No" },
        inline_overhead,
        manifest_overhead
    ));
    md.push_str(&format!(
        "3. Does fragmentation multiply path duplication cost into unacceptable territory? **{}** (bytes added per extent vs extent_identity_only = {:.6}).
",
        if fragmentation_cost > 32.0 { "Possibly" } else { "No" },
        fragmentation_cost
    ));
    md.push_str("4. Is `extent_identity_inline_path` still credible for a compression-oriented archive format? **Yes, pending FORMAT-13 policy lock** (size/recovery tradeoff remains bounded in this stress run).
");
    md.push_str("5. Should it remain the leading candidate going into FORMAT-13? **Yes** as the lead identity-layer baseline for the next packet decision.
");

    fs::write(
        comparison_dir.join("format12_stress_comparison_summary.md"),
        &md,
    )?;
    // legacy compatibility path used by existing notes/tests.
    fs::write(comparison_dir.join("format12_stress_summary.md"), &md)?;

    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

fn format12_stress_scenarios() -> Vec<ComparisonScenario> {
    let datasets = [
        "deep_paths",
        "long_names",
        "fragmentation_heavy",
        "mixed_worst_case",
    ];
    let targets = [
        ("header", "byte_flip"),
        ("index", "byte_flip"),
        ("payload", "byte_flip"),
        ("tail", "truncate"),
    ];
    let magnitudes = ["small", "medium"];

    let mut scenarios = Vec::new();
    let mut seed = 9100u64;
    for dataset in datasets {
        for (target, model) in targets {
            for magnitude in magnitudes {
                scenarios.push(ComparisonScenario {
                    scenario_id: format!("{}_{}_{}_{}", dataset, target, model, magnitude),
                    dataset,
                    corruption_model: model,
                    corruption_target: target,
                    magnitude,
                    seed,
                    break_redundant_map: false,
                });
                seed += 1;
            }
        }
    }
    scenarios
}

fn compute_stress_identity_stats(archive: &Path, input_dir: &Path) -> Result<Value> {
    let meta_rows = parse_metadata_json_blocks(archive)?;
    let mut total_extent_count = 0u64;
    let mut total_path_len = 0u64;
    let mut path_count = 0u64;
    let mut max_path_length = 0u64;
    let mut by_logical: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();

    for row in &meta_rows {
        if row.get("schema").and_then(Value::as_str) == Some("crushr-payload-block-identity.v1") {
            total_extent_count += 1;
            if let Some(path) = row.get("path").and_then(Value::as_str) {
                let len = path.len() as u64;
                total_path_len += len;
                path_count += 1;
                max_path_length = max_path_length.max(len);
                let key = logical_key_for_path(path);
                *by_logical.entry(key).or_insert(0) += 1;
            }
        }
    }

    if path_count == 0 {
        let mut stack = vec![input_dir.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for e in fs::read_dir(&dir)? {
                let e = e?;
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() {
                    let rel = p.strip_prefix(input_dir).unwrap_or(&p);
                    let s = rel.to_string_lossy();
                    let len = s.len() as u64;
                    total_path_len += len;
                    path_count += 1;
                    max_path_length = max_path_length.max(len);
                    let key = logical_key_for_path(&s);
                    *by_logical.entry(key).or_insert(0) += 1;
                }
            }
        }
        total_extent_count = path_count;
    }

    let mut dist: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut max_extents_per_file = 0u64;
    for cnt in by_logical.values() {
        max_extents_per_file = max_extents_per_file.max(*cnt);
        *dist.entry(cnt.to_string()).or_insert(0) += 1;
    }
    let average_extents_per_file = if by_logical.is_empty() {
        0.0
    } else {
        total_extent_count as f64 / by_logical.len() as f64
    };

    Ok(serde_json::json!({
        "average_path_length": if path_count > 0 { total_path_len as f64 / path_count as f64 } else { 0.0 },
        "max_path_length": max_path_length,
        "total_extent_count": total_extent_count,
        "average_extents_per_file": average_extents_per_file,
        "max_extents_per_file": max_extents_per_file,
        "path_char_count": total_path_len,
        "extents_per_file_distribution": dist,
    }))
}

fn mean_f64(rows: &[&Value], field: &str) -> f64 {
    if rows.is_empty() {
        return 0.0;
    }
    rows.iter()
        .map(|r| r[field].as_f64().unwrap_or(0.0))
        .sum::<f64>()
        / rows.len() as f64
}

fn max_u64(rows: &[&Value], field: &str) -> u64 {
    rows.iter()
        .map(|r| r[field].as_u64().unwrap_or(0))
        .max()
        .unwrap_or(0)
}

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
        "tool_version": env!("CARGO_PKG_VERSION"),
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

pub(super) fn run_format13_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format13_impl(comparison_dir, verbose, false)
}

pub(super) fn run_format13_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
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
        "tool_version": env!("CARGO_PKG_VERSION"),
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

pub(super) fn run_format14a_dictionary_resilience_comparison(
    comparison_dir: &Path,
    verbose: bool,
) -> Result<()> {
    run_format14a_dictionary_resilience_impl(comparison_dir, verbose, false)
}

pub(super) fn run_format14a_dictionary_resilience_stress_comparison(
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

pub(super) fn run_format15_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
    run_format15_impl(comparison_dir, verbose, false)
}

pub(super) fn run_format15_stress_comparison(comparison_dir: &Path, verbose: bool) -> Result<()> {
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
