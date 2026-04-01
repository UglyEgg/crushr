// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result};
use crushr::introspection::{profile_entry, profile_find};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Deserialize, Serialize)]
struct ArchiveSpec {
    dataset_name: String,
    class_name: String,
    archive_path: String,
    dataset_path: String,
    query: String,
    entry_path: String,
}

#[derive(Serialize)]
struct Stats {
    samples_ms: Vec<f64>,
    mean_ms: f64,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

#[derive(Serialize)]
struct OpProfile {
    cold: crushr::introspection::HotspotBreakdownMs,
    repeated_total_ms: Stats,
    repeated_archive_open_read_ms: Stats,
    repeated_index_decode_parse_ms: Stats,
    repeated_summary_index_prep_ms: Stats,
    repeated_traversal_ms: Stats,
    repeated_result_materialization_ms: Stats,
}

#[derive(Serialize)]
struct ArchiveHotspot {
    dataset_name: String,
    class_name: String,
    archive_path: String,
    query: String,
    entry_path: String,
    operations: ArchiveOps,
}

#[derive(Serialize)]
struct ArchiveOps {
    find: OpProfile,
    entry: OpProfile,
}

#[derive(Serialize)]
struct HotspotReport {
    runs: usize,
    mode: String,
    archives: Vec<ArchiveHotspot>,
}

fn stats(values: &[f64]) -> Stats {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    let median = if len % 2 == 1 {
        sorted[len / 2]
    } else {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    };
    let mean = sorted.iter().sum::<f64>() / len as f64;
    Stats {
        samples_ms: values
            .iter()
            .map(|v| (v * 1000.0).round() / 1000.0)
            .collect(),
        mean_ms: (mean * 1000.0).round() / 1000.0,
        median_ms: (median * 1000.0).round() / 1000.0,
        min_ms: (sorted[0] * 1000.0).round() / 1000.0,
        max_ms: (sorted[len - 1] * 1000.0).round() / 1000.0,
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut specs_path = None;
    let mut out_path = None;
    let mut runs = 5usize;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--specs" => specs_path = args.next(),
            "--out" => out_path = args.next(),
            "--runs" => {
                let raw = args.next().context("missing value for --runs")?;
                runs = raw.parse::<usize>().context("invalid --runs value")?;
            }
            _ => anyhow::bail!("unknown argument: {arg}"),
        }
    }

    let specs_path = specs_path.context("missing --specs path")?;
    let out_path = out_path.context("missing --out path")?;

    let specs: Vec<ArchiveSpec> =
        serde_json::from_str(&fs::read_to_string(&specs_path).context("read specs file")?)
            .context("parse specs JSON")?;

    let mut archives = Vec::new();
    for spec in specs {
        let cold_find = profile_find(&spec.archive_path, &spec.query, None)?;
        let cold_entry = profile_entry(&spec.archive_path, &spec.entry_path)?;

        let mut find_total = Vec::with_capacity(runs);
        let mut find_open = Vec::with_capacity(runs);
        let mut find_decode = Vec::with_capacity(runs);
        let mut find_prep = Vec::with_capacity(runs);
        let mut find_traversal = Vec::with_capacity(runs);
        let mut find_materialization = Vec::with_capacity(runs);

        let mut entry_total = Vec::with_capacity(runs);
        let mut entry_open = Vec::with_capacity(runs);
        let mut entry_decode = Vec::with_capacity(runs);
        let mut entry_prep = Vec::with_capacity(runs);
        let mut entry_traversal = Vec::with_capacity(runs);
        let mut entry_materialization = Vec::with_capacity(runs);

        for _ in 0..runs {
            let find = profile_find(&spec.archive_path, &spec.query, None)?;
            find_total.push(find.timings_ms.total);
            find_open.push(find.timings_ms.archive_open_read);
            find_decode.push(find.timings_ms.index_decode_parse);
            find_prep.push(find.timings_ms.summary_index_prep);
            find_traversal.push(find.timings_ms.traversal);
            find_materialization.push(find.timings_ms.result_materialization);

            let entry = profile_entry(&spec.archive_path, &spec.entry_path)?;
            entry_total.push(entry.timings_ms.total);
            entry_open.push(entry.timings_ms.archive_open_read);
            entry_decode.push(entry.timings_ms.index_decode_parse);
            entry_prep.push(entry.timings_ms.summary_index_prep);
            entry_traversal.push(entry.timings_ms.traversal);
            entry_materialization.push(entry.timings_ms.result_materialization);
        }

        archives.push(ArchiveHotspot {
            dataset_name: spec.dataset_name,
            class_name: spec.class_name,
            archive_path: spec.archive_path,
            query: spec.query,
            entry_path: spec.entry_path,
            operations: ArchiveOps {
                find: OpProfile {
                    cold: cold_find.timings_ms,
                    repeated_total_ms: stats(&find_total),
                    repeated_archive_open_read_ms: stats(&find_open),
                    repeated_index_decode_parse_ms: stats(&find_decode),
                    repeated_summary_index_prep_ms: stats(&find_prep),
                    repeated_traversal_ms: stats(&find_traversal),
                    repeated_result_materialization_ms: stats(&find_materialization),
                },
                entry: OpProfile {
                    cold: cold_entry.timings_ms,
                    repeated_total_ms: stats(&entry_total),
                    repeated_archive_open_read_ms: stats(&entry_open),
                    repeated_index_decode_parse_ms: stats(&entry_decode),
                    repeated_summary_index_prep_ms: stats(&entry_prep),
                    repeated_traversal_ms: stats(&entry_traversal),
                    repeated_result_materialization_ms: stats(&entry_materialization),
                },
            },
        });
    }

    let out = HotspotReport {
        runs,
        mode: "cli_hotspot".to_string(),
        archives,
    };
    fs::write(out_path, serde_json::to_string_pretty(&out)? + "\n")
        .context("write hotspot output")?;
    Ok(())
}
