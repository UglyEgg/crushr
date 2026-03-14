use crate::phase2_domain::ArchiveFormat;
use crate::phase2_normalization::NormalizedRunRecord;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_INPUT_PATH: &str = "PHASE2_RESEARCH/results/normalized_results.json";
const DEFAULT_OUTPUT_DIR: &str = "PHASE2_RESEARCH/summaries";

pub const COMPARISON_TABLES_SCHEMA_PATH: &str =
    "schemas/crushr-lab-phase2-comparison-tables.v1.schema.json";
pub const COMPARISON_TABLES_SCHEMA_ID: &str =
    "https://crushr.dev/schemas/crushr-lab-phase2-comparison-tables.v1.schema.json";
pub const FORMAT_RANKINGS_SCHEMA_PATH: &str =
    "schemas/crushr-lab-phase2-format-rankings.v1.schema.json";
pub const FORMAT_RANKINGS_SCHEMA_ID: &str =
    "https://crushr.dev/schemas/crushr-lab-phase2-format-rankings.v1.schema.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatComparisonRow {
    pub format: ArchiveFormat,
    pub total_runs: usize,
    pub recovery_success_rate: f64,
    pub mean_recovery_ratio_files: f64,
    pub mean_recovery_ratio_bytes: f64,
    pub detection_rate: f64,
    pub blast_radius_distribution: BTreeMap<String, f64>,
    pub diagnostic_specificity_distribution: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonTables {
    pub rows: Vec<FormatComparisonRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub rank: usize,
    pub format: ArchiveFormat,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRankings {
    pub survivability: Vec<RankingEntry>,
    pub diagnostic_quality: Vec<RankingEntry>,
    pub corruption_containment: Vec<RankingEntry>,
}

#[derive(Debug, Default)]
struct FormatAccumulator {
    total_runs: usize,
    recovery_success_count: usize,
    recovery_ratio_files_sum: f64,
    recovery_ratio_bytes_sum: f64,
    detection_count: usize,
    blast_counts: BTreeMap<String, usize>,
    diagnostic_counts: BTreeMap<String, usize>,
}

pub fn run_phase2_comparison_cmd(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let root = crate::cli::workspace_root()?;
    let mut input_path = root.join(DEFAULT_INPUT_PATH);
    let mut output_dir = root.join(DEFAULT_OUTPUT_DIR);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                input_path = PathBuf::from(args.next().context("missing value for --input")?)
            }
            "--output-dir" => {
                output_dir = PathBuf::from(args.next().context("missing value for --output-dir")?)
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let records: Vec<NormalizedRunRecord> = serde_json::from_slice(
        &fs::read(&input_path).with_context(|| format!("reading {}", input_path.display()))?,
    )
    .with_context(|| format!("parsing {}", input_path.display()))?;

    let tables = build_comparison_tables(&records);
    let rankings = build_format_rankings(&tables);

    fs::create_dir_all(&output_dir)?;
    let tables_path = output_dir.join("comparison_tables.json");
    let rankings_path = output_dir.join("format_rankings.json");

    fs::write(&tables_path, serde_json::to_vec_pretty(&tables)?)?;
    fs::write(&rankings_path, serde_json::to_vec_pretty(&rankings)?)?;

    validate_comparison_tables_shape(&serde_json::to_value(&tables)?)?;
    validate_format_rankings_shape(&serde_json::to_value(&rankings)?)?;

    eprintln!("wrote {}", tables_path.display());
    eprintln!("wrote {}", rankings_path.display());
    eprintln!(
        "comparison tables schema: {COMPARISON_TABLES_SCHEMA_ID} ({COMPARISON_TABLES_SCHEMA_PATH})"
    );
    eprintln!(
        "format rankings schema: {FORMAT_RANKINGS_SCHEMA_ID} ({FORMAT_RANKINGS_SCHEMA_PATH})"
    );

    Ok(())
}

fn build_comparison_tables(records: &[NormalizedRunRecord]) -> ComparisonTables {
    let mut per_format: BTreeMap<String, FormatAccumulator> = BTreeMap::new();

    for record in records {
        let key = serde_json::to_string(&record.format)
            .expect("archive format serialization should succeed")
            .trim_matches('"')
            .to_string();
        let accumulator = per_format.entry(key).or_default();
        accumulator.total_runs += 1;
        if record.recovery_ratio_files > 0.0 {
            accumulator.recovery_success_count += 1;
        }
        accumulator.recovery_ratio_files_sum += record.recovery_ratio_files;
        accumulator.recovery_ratio_bytes_sum += record.recovery_ratio_bytes;
        if record.detected_pre_extract {
            accumulator.detection_count += 1;
        }

        let blast_key = serde_json::to_string(&record.blast_radius_class)
            .expect("blast radius class serialization should succeed")
            .trim_matches('"')
            .to_string();
        *accumulator.blast_counts.entry(blast_key).or_insert(0) += 1;

        let diag_key = serde_json::to_string(&record.diagnostic_specificity)
            .expect("diagnostic specificity serialization should succeed")
            .trim_matches('"')
            .to_string();
        *accumulator.diagnostic_counts.entry(diag_key).or_insert(0) += 1;
    }

    let mut rows = Vec::new();
    for (format, acc) in per_format {
        if acc.total_runs == 0 {
            continue;
        }

        rows.push(FormatComparisonRow {
            format: parse_format(&format),
            total_runs: acc.total_runs,
            recovery_success_rate: ratio(acc.recovery_success_count, acc.total_runs),
            mean_recovery_ratio_files: acc.recovery_ratio_files_sum / acc.total_runs as f64,
            mean_recovery_ratio_bytes: acc.recovery_ratio_bytes_sum / acc.total_runs as f64,
            detection_rate: ratio(acc.detection_count, acc.total_runs),
            blast_radius_distribution: normalize_distribution(acc.blast_counts, acc.total_runs),
            diagnostic_specificity_distribution: normalize_distribution(
                acc.diagnostic_counts,
                acc.total_runs,
            ),
        });
    }

    rows.sort_by(|a, b| format_name(a.format).cmp(format_name(b.format)));
    ComparisonTables { rows }
}

fn build_format_rankings(tables: &ComparisonTables) -> FormatRankings {
    let mut survivability: Vec<(ArchiveFormat, f64, f64, f64)> = tables
        .rows
        .iter()
        .map(|row| {
            (
                row.format,
                row.recovery_success_rate,
                row.mean_recovery_ratio_files,
                row.mean_recovery_ratio_bytes,
            )
        })
        .collect();
    survivability.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| b.2.total_cmp(&a.2))
            .then_with(|| b.3.total_cmp(&a.3))
            .then_with(|| format_name(a.0).cmp(format_name(b.0)))
    });

    let mut diagnostic: Vec<(ArchiveFormat, f64, f64, f64)> = tables
        .rows
        .iter()
        .map(|row| {
            let specificity_score =
                weighted_specificity_score(&row.diagnostic_specificity_distribution);
            let composite = (row.detection_rate + specificity_score) / 2.0;
            (row.format, composite, row.detection_rate, specificity_score)
        })
        .collect();
    diagnostic.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| b.2.total_cmp(&a.2))
            .then_with(|| b.3.total_cmp(&a.3))
            .then_with(|| format_name(a.0).cmp(format_name(b.0)))
    });

    let mut containment: Vec<(ArchiveFormat, f64, f64)> = tables
        .rows
        .iter()
        .map(|row| {
            let containment_score = weighted_containment_score(&row.blast_radius_distribution);
            (row.format, containment_score, row.mean_recovery_ratio_files)
        })
        .collect();
    containment.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| b.2.total_cmp(&a.2))
            .then_with(|| format_name(a.0).cmp(format_name(b.0)))
    });

    FormatRankings {
        survivability: to_ranked_entries(survivability.into_iter().map(|v| (v.0, v.1)).collect()),
        diagnostic_quality: to_ranked_entries(diagnostic.into_iter().map(|v| (v.0, v.1)).collect()),
        corruption_containment: to_ranked_entries(
            containment.into_iter().map(|v| (v.0, v.1)).collect(),
        ),
    }
}

fn to_ranked_entries(items: Vec<(ArchiveFormat, f64)>) -> Vec<RankingEntry> {
    items
        .into_iter()
        .enumerate()
        .map(|(index, (format, score))| RankingEntry {
            rank: index + 1,
            format,
            score,
        })
        .collect()
}

fn normalize_distribution(counts: BTreeMap<String, usize>, total: usize) -> BTreeMap<String, f64> {
    counts
        .into_iter()
        .map(|(key, value)| (key, ratio(value, total)))
        .collect()
}

fn ratio(numer: usize, denom: usize) -> f64 {
    if denom == 0 {
        0.0
    } else {
        numer as f64 / denom as f64
    }
}

fn weighted_specificity_score(distribution: &BTreeMap<String, f64>) -> f64 {
    let mut score = 0.0;
    for (label, ratio) in distribution {
        let weight = match label.as_str() {
            "NONE" => 0.0,
            "GENERIC" => 1.0,
            "STRUCTURAL" => 2.0,
            "PRECISE" => 3.0,
            _ => 0.0,
        };
        score += ratio * weight;
    }
    score / 3.0
}

fn weighted_containment_score(distribution: &BTreeMap<String, f64>) -> f64 {
    let mut score = 0.0;
    for (label, ratio) in distribution {
        let weight = match label.as_str() {
            "NONE" => 1.0,
            "LOCALIZED" => 0.75,
            "PARTIAL_SET" => 0.5,
            "WIDESPREAD" => 0.25,
            "TOTAL" => 0.0,
            _ => 0.0,
        };
        score += ratio * weight;
    }
    score
}

fn parse_format(raw: &str) -> ArchiveFormat {
    match raw {
        "crushr" => ArchiveFormat::Crushr,
        "zip" => ArchiveFormat::Zip,
        "tar+zstd" => ArchiveFormat::TarZstd,
        "tar+gz" => ArchiveFormat::TarGz,
        "tar+xz" => ArchiveFormat::TarXz,
        _ => panic!("unexpected archive format: {raw}"),
    }
}

fn format_name(format: ArchiveFormat) -> &'static str {
    match format {
        ArchiveFormat::Crushr => "crushr",
        ArchiveFormat::Zip => "zip",
        ArchiveFormat::TarZstd => "tar+zstd",
        ArchiveFormat::TarGz => "tar+gz",
        ArchiveFormat::TarXz => "tar+xz",
    }
}

pub fn validate_comparison_tables_shape(value: &Value) -> Result<()> {
    let root = value
        .as_object()
        .context("comparison tables root must be an object")?;

    let rows = root
        .get("rows")
        .and_then(Value::as_array)
        .context("comparison tables `rows` must be an array")?;

    for row in rows {
        let obj = row
            .as_object()
            .context("comparison table row must be an object")?;
        for field in [
            "format",
            "total_runs",
            "recovery_success_rate",
            "mean_recovery_ratio_files",
            "mean_recovery_ratio_bytes",
            "detection_rate",
            "blast_radius_distribution",
            "diagnostic_specificity_distribution",
        ] {
            if !obj.contains_key(field) {
                bail!("comparison table row missing required field `{field}`");
            }
        }
    }

    validate_ratio_maps(rows, "blast_radius_distribution")?;
    validate_ratio_maps(rows, "diagnostic_specificity_distribution")?;

    Ok(())
}

fn validate_ratio_maps(rows: &[Value], field_name: &str) -> Result<()> {
    for row in rows {
        let obj = row
            .as_object()
            .context("comparison table row must be an object")?;
        let dist = obj
            .get(field_name)
            .and_then(Value::as_object)
            .with_context(|| format!("comparison table field `{field_name}` must be an object"))?;
        for (key, value) in dist {
            let ratio = value
                .as_f64()
                .with_context(|| format!("distribution `{field_name}.{key}` must be numeric"))?;
            if !(0.0..=1.0).contains(&ratio) {
                bail!("distribution `{field_name}.{key}` must be in [0,1]");
            }
        }
    }
    Ok(())
}

pub fn validate_format_rankings_shape(value: &Value) -> Result<()> {
    let root = value
        .as_object()
        .context("format rankings root must be an object")?;

    for field in [
        "survivability",
        "diagnostic_quality",
        "corruption_containment",
    ] {
        let entries = root
            .get(field)
            .and_then(Value::as_array)
            .with_context(|| format!("format rankings `{field}` must be an array"))?;

        for entry in entries {
            let obj = entry
                .as_object()
                .context("ranking entry must be an object")?;
            for required in ["rank", "format", "score"] {
                if !obj.contains_key(required) {
                    bail!("ranking entry missing required field `{required}`");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_outputs_validate_against_shape_contracts() {
        let root = crate::cli::workspace_root().expect("workspace root");
        let records: Vec<NormalizedRunRecord> = serde_json::from_slice(
            &fs::read(root.join("PHASE2_RESEARCH/results/normalized_results.json"))
                .expect("read normalized results"),
        )
        .expect("parse normalized results");

        let tables = build_comparison_tables(&records);
        let rankings = build_format_rankings(&tables);

        validate_comparison_tables_shape(&serde_json::to_value(&tables).expect("tables to value"))
            .expect("comparison tables shape valid");
        validate_format_rankings_shape(
            &serde_json::to_value(&rankings).expect("rankings to value"),
        )
        .expect("format rankings shape valid");
    }

    #[test]
    fn schema_files_match_expected_ids() {
        let root = crate::cli::workspace_root().expect("workspace root");

        let tables_schema: Value = serde_json::from_slice(
            &fs::read(root.join(COMPARISON_TABLES_SCHEMA_PATH)).expect("read comparison schema"),
        )
        .expect("parse comparison schema");
        assert_eq!(tables_schema["$id"], COMPARISON_TABLES_SCHEMA_ID);

        let rankings_schema: Value = serde_json::from_slice(
            &fs::read(root.join(FORMAT_RANKINGS_SCHEMA_PATH)).expect("read rankings schema"),
        )
        .expect("parse rankings schema");
        assert_eq!(rankings_schema["$id"], FORMAT_RANKINGS_SCHEMA_ID);
    }
}
