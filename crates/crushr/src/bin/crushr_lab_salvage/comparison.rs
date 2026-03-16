use super::*;
use crate::runner::{resolve_pack_bin, resolve_salvage_bin};

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
    format05: &BTreeMap<String, u64>,
    format06: &BTreeMap<String, u64>,
    key: &str,
) -> i64 {
    format06.get(key).copied().unwrap_or(0) as i64 - format05.get(key).copied().unwrap_or(0) as i64
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
