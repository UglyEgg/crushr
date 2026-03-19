use crate::phase2_domain::{ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass};
use crate::phase2_runner::{RawRunRecord, RecoveryAccounting};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_TRIALS_DIR: &str = "PHASE2_RESEARCH/trials";
const DEFAULT_OUTPUT_DIR: &str = "PHASE2_RESEARCH/results";

pub const NORMALIZED_RESULTS_SCHEMA_PATH: &str =
    "schemas/crushr-lab-phase2-normalized-results.v1.schema.json";
pub const NORMALIZED_RESULTS_SCHEMA_ID: &str =
    "https://crushr.dev/schemas/crushr-lab-phase2-normalized-results.v1.schema.json";
pub const NORMALIZED_SUMMARY_SCHEMA_PATH: &str =
    "schemas/crushr-lab-phase2-normalization-summary.v1.schema.json";
pub const NORMALIZED_SUMMARY_SCHEMA_ID: &str =
    "https://crushr.dev/schemas/crushr-lab-phase2-normalization-summary.v1.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResultClass {
    Success,
    Partial,
    Refused,
    StructuralFail,
    ToolError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureStage {
    None,
    PreExtract,
    Extraction,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticSpecificity {
    None,
    Generic,
    Structural,
    Precise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryEvidenceStrength {
    FilePresenceOnly,
    FileAndByteCounts,
    FileByteAndContentValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlastRadiusClass {
    None,
    Localized,
    PartialSet,
    Widespread,
    Total,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRunRecord {
    pub scenario_id: String,
    pub dataset: Dataset,
    pub format: ArchiveFormat,
    pub corruption_type: CorruptionType,
    pub target_class: TargetClass,
    pub magnitude: Magnitude,
    pub magnitude_bytes: u64,
    pub seed: u64,
    pub tool_kind: ArchiveFormat,
    pub exit_code: i32,
    pub has_json_result: bool,
    pub result_completeness: String,
    pub detected_pre_extract: bool,
    pub failure_stage: FailureStage,
    pub result_class: ResultClass,
    pub diagnostic_specificity: DiagnosticSpecificity,
    pub files_expected: u64,
    pub files_recovered: u64,
    pub files_missing: u64,
    pub bytes_expected: u64,
    pub bytes_recovered: u64,
    pub recovery_ratio_files: f64,
    pub recovery_ratio_bytes: f64,
    pub blast_radius_class: BlastRadiusClass,
    pub normalization_notes: Vec<String>,
    pub recovery_evidence_strength: RecoveryEvidenceStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationSummary {
    pub total_normalized_runs: usize,
    pub result_class_counts: BTreeMap<String, usize>,
    pub failure_stage_counts: BTreeMap<String, usize>,
    pub diagnostic_specificity_counts: BTreeMap<String, usize>,
    pub per_format_result_class_counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub runs_with_recovery_accounting: usize,
    pub recovery_evidence_strength_counts: BTreeMap<String, usize>,
    pub per_format_average_recovery_ratio_files: BTreeMap<String, f64>,
    pub per_format_average_recovery_ratio_bytes: BTreeMap<String, f64>,
    pub per_format_blast_radius_class_counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub per_corruption_type_average_recovery_ratio_files: BTreeMap<String, f64>,
    pub per_target_average_recovery_ratio_files: BTreeMap<String, f64>,
    pub per_format_notes: BTreeMap<String, String>,
    pub mapping_report: BTreeMap<String, String>,
}

pub fn run_phase2_normalization_cmd(raw_args: Vec<String>) -> Result<()> {
    let mut args = raw_args.into_iter();
    let root = crate::cli::workspace_root()?;
    let mut trials_dir = root.join(DEFAULT_TRIALS_DIR);
    let mut output_dir = root.join(DEFAULT_OUTPUT_DIR);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trials-dir" => {
                trials_dir = PathBuf::from(args.next().context("missing value for --trials-dir")?)
            }
            "--output-dir" => {
                output_dir = PathBuf::from(args.next().context("missing value for --output-dir")?)
            }
            _ => bail!("unsupported flag: {arg}"),
        }
    }

    let normalized = normalize_from_trials(&trials_dir)?;
    let summary = build_summary(&normalized.records);

    fs::create_dir_all(&output_dir)?;
    let normalized_path = output_dir.join("normalized_results.json");
    let summary_path = output_dir.join("normalization_summary.json");

    fs::write(
        &normalized_path,
        serde_json::to_vec_pretty(&normalized.records)?,
    )?;
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary)?)?;

    validate_normalized_results_shape(&serde_json::to_value(&normalized.records)?)?;
    validate_normalization_summary_shape(&serde_json::to_value(&summary)?)?;

    eprintln!("wrote {}", normalized_path.display());
    eprintln!("wrote {}", summary_path.display());
    eprintln!("normalized results schema: {NORMALIZED_RESULTS_SCHEMA_ID} ({NORMALIZED_RESULTS_SCHEMA_PATH})");
    eprintln!("normalization summary schema: {NORMALIZED_SUMMARY_SCHEMA_ID} ({NORMALIZED_SUMMARY_SCHEMA_PATH})");
    Ok(())
}

pub struct NormalizedCorpus {
    pub records: Vec<NormalizedRunRecord>,
}

pub fn normalize_from_trials(trials_dir: &Path) -> Result<NormalizedCorpus> {
    let raw_path = trials_dir.join("raw_run_records.json");
    let raw_records: Vec<RawRunRecord> = serde_json::from_slice(
        &fs::read(&raw_path).with_context(|| format!("reading {}", raw_path.display()))?,
    )
    .context("parsing raw_run_records.json")?;

    let mut normalized = Vec::with_capacity(raw_records.len());
    for record in raw_records {
        normalized.push(normalize_record(trials_dir, &record)?);
    }

    sort_normalized_records_by_scenario_id(&mut normalized);

    Ok(NormalizedCorpus {
        records: normalized,
    })
}

fn sort_normalized_records_by_scenario_id(records: &mut [NormalizedRunRecord]) {
    records.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
}

fn normalize_record(trials_dir: &Path, record: &RawRunRecord) -> Result<NormalizedRunRecord> {
    let stdout = fs::read_to_string(trials_dir.join(&record.stdout_path)).unwrap_or_default();
    let stderr = fs::read_to_string(trials_dir.join(&record.stderr_path)).unwrap_or_default();

    let mut notes = Vec::new();
    let diag_text = [stdout.as_str(), stderr.as_str()].join("\n");
    let diag_lc = diag_text.to_lowercase();

    let has_json_file = if let Some(path) = &record.json_result_path {
        trials_dir.join(path).is_file()
    } else {
        false
    };

    let recovery_evidence_strength = RecoveryEvidenceStrength::FileAndByteCounts;

    if record.has_json_result && !has_json_file {
        notes.push("has_json_result=true but json_result_path missing on disk".to_string());
    }

    let failure_stage = classify_failure_stage(record.exit_code, &diag_lc);
    let detected_pre_extract = matches!(failure_stage, FailureStage::PreExtract);
    let diagnostic_specificity = classify_diagnostic_specificity(&diag_lc);

    let refusal_signal = contains_any(
        &diag_lc,
        &[
            "refused",
            "refusal",
            "cannot inflate",
            "invalid compressed data to inflate",
            "skipping to next header",
            "expected ",
        ],
    );

    let result_class = classify_result_class(record, failure_stage, refusal_signal, has_json_file);

    let recovery = normalize_recovery_accounting(&record.recovery_accounting);
    let blast_radius_class = classify_blast_radius(recovery.recovery_ratio_files);

    if matches!(failure_stage, FailureStage::Unknown) && record.exit_code != 0 {
        notes.push(
            "non-zero exit with no deterministic stage markers; normalized to UNKNOWN stage"
                .to_string(),
        );
    }

    Ok(NormalizedRunRecord {
        scenario_id: record.scenario_id.clone(),
        dataset: record.dataset,
        format: record.format,
        corruption_type: record.corruption_type,
        target_class: record.target_class,
        magnitude: record.magnitude,
        magnitude_bytes: record.magnitude_bytes,
        seed: record.seed,
        tool_kind: record.tool_kind,
        exit_code: record.exit_code,
        has_json_result: record.has_json_result,
        result_completeness: match record.result_completeness {
            crate::phase2_runner::EvidenceQuality::StdoutOnly => "stdout_only".to_string(),
            crate::phase2_runner::EvidenceQuality::StdoutAndStderr => {
                "stdout_and_stderr".to_string()
            }
            crate::phase2_runner::EvidenceQuality::StructuredJsonResult => {
                "structured_json_result".to_string()
            }
        },
        detected_pre_extract,
        failure_stage,
        result_class,
        diagnostic_specificity,
        files_expected: recovery.files_expected,
        files_recovered: recovery.files_recovered,
        files_missing: recovery.files_missing,
        bytes_expected: recovery.bytes_expected,
        bytes_recovered: recovery.bytes_recovered,
        recovery_ratio_files: recovery.recovery_ratio_files,
        recovery_ratio_bytes: recovery.recovery_ratio_bytes,
        blast_radius_class,
        normalization_notes: notes,
        recovery_evidence_strength,
    })
}

fn classify_result_class(
    record: &RawRunRecord,
    failure_stage: FailureStage,
    refusal_signal: bool,
    has_json_file: bool,
) -> ResultClass {
    if record.exit_code == -1 {
        return ResultClass::ToolError;
    }
    if record.has_json_result && !has_json_file {
        return ResultClass::ToolError;
    }
    if record.exit_code == 0 {
        return ResultClass::Success;
    }

    if refusal_signal && matches!(failure_stage, FailureStage::Extraction) {
        return ResultClass::Refused;
    }
    if refusal_signal {
        return ResultClass::Partial;
    }
    if matches!(failure_stage, FailureStage::PreExtract) {
        return ResultClass::StructuralFail;
    }
    if matches!(failure_stage, FailureStage::Extraction) {
        return ResultClass::Partial;
    }
    ResultClass::ToolError
}

fn classify_failure_stage(exit_code: i32, diag_lc: &str) -> FailureStage {
    if exit_code == 0 {
        return FailureStage::None;
    }

    if contains_any(
        diag_lc,
        &[
            "bad footer magic",
            "bad magic",
            "hash mismatch",
            "missing end signature",
            "not a zip file",
            "not in gzip format",
            "file format not recognized",
            "unsupported format",
            "unknown header",
            "this does not look like a tar archive",
            "premature end",
            "unexpected end of file",
            "unexpected end of input",
        ],
    ) {
        return FailureStage::PreExtract;
    }

    if contains_any(
        diag_lc,
        &[
            "invalid compressed data to inflate",
            "skipping to next header",
            "bad zipfile offset",
            "crc error",
            "length error",
            "format violated",
            "filename too long",
        ],
    ) {
        return FailureStage::Extraction;
    }

    FailureStage::Unknown
}

fn classify_diagnostic_specificity(diag_lc: &str) -> DiagnosticSpecificity {
    if diag_lc.trim().is_empty() {
        return DiagnosticSpecificity::None;
    }

    if contains_any(
        diag_lc,
        &[
            "payload/", "bin/", "cfg/", "file #", "inflate", "idx3", "ftr4", "header",
        ],
    ) {
        return DiagnosticSpecificity::Precise;
    }

    if contains_any(
        diag_lc,
        &[
            "magic",
            "archive",
            "format",
            "checksum",
            "crc",
            "corrupt",
            "hash mismatch",
            "premature end",
            "unexpected end",
            "footer",
            "index",
        ],
    ) {
        return DiagnosticSpecificity::Structural;
    }

    DiagnosticSpecificity::Generic
}

fn normalize_recovery_accounting(accounting: &RecoveryAccounting) -> RecoveryAccounting {
    RecoveryAccounting {
        files_expected: accounting.files_expected,
        files_recovered: accounting.files_recovered.min(accounting.files_expected),
        files_missing: accounting.files_missing.min(
            accounting
                .files_expected
                .saturating_sub(accounting.files_recovered.min(accounting.files_expected)),
        ),
        bytes_expected: accounting.bytes_expected,
        bytes_recovered: accounting.bytes_recovered.min(accounting.bytes_expected),
        recovery_ratio_files: ratio_clamped(accounting.files_recovered, accounting.files_expected),
        recovery_ratio_bytes: ratio_clamped(accounting.bytes_recovered, accounting.bytes_expected),
    }
}

fn classify_blast_radius(recovery_ratio_files: f64) -> BlastRadiusClass {
    if recovery_ratio_files >= 1.0 {
        BlastRadiusClass::None
    } else if recovery_ratio_files >= 0.9 {
        BlastRadiusClass::Localized
    } else if recovery_ratio_files >= 0.5 {
        BlastRadiusClass::PartialSet
    } else if recovery_ratio_files > 0.0 {
        BlastRadiusClass::Widespread
    } else {
        BlastRadiusClass::Total
    }
}

fn ratio_clamped(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64).clamp(0.0, 1.0)
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn build_summary(records: &[NormalizedRunRecord]) -> NormalizationSummary {
    let mut result_class_counts = BTreeMap::<String, usize>::new();
    let mut failure_stage_counts = BTreeMap::<String, usize>::new();
    let mut diagnostic_specificity_counts = BTreeMap::<String, usize>::new();
    let mut per_format_result_class_counts = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut recovery_evidence_strength_counts = BTreeMap::<String, usize>::new();
    let mut per_format_blast_radius_class_counts =
        BTreeMap::<String, BTreeMap<String, usize>>::new();

    let mut format_file_ratio_sum = BTreeMap::<String, (f64, usize)>::new();
    let mut format_byte_ratio_sum = BTreeMap::<String, (f64, usize)>::new();
    let mut corruption_ratio_sum = BTreeMap::<String, (f64, usize)>::new();
    let mut target_ratio_sum = BTreeMap::<String, (f64, usize)>::new();

    for record in records {
        *result_class_counts
            .entry(result_class_label(record.result_class).to_string())
            .or_insert(0) += 1;
        *failure_stage_counts
            .entry(failure_stage_label(record.failure_stage).to_string())
            .or_insert(0) += 1;
        *diagnostic_specificity_counts
            .entry(diagnostic_label(record.diagnostic_specificity).to_string())
            .or_insert(0) += 1;

        let format_key = archive_format_label(record.format).to_string();
        let rc_key = result_class_label(record.result_class).to_string();
        *per_format_result_class_counts
            .entry(format_key.clone())
            .or_default()
            .entry(rc_key)
            .or_insert(0) += 1;

        *recovery_evidence_strength_counts
            .entry(recovery_evidence_strength_label(record.recovery_evidence_strength).to_string())
            .or_insert(0) += 1;

        *per_format_blast_radius_class_counts
            .entry(format_key.clone())
            .or_default()
            .entry(blast_radius_label(record.blast_radius_class).to_string())
            .or_insert(0) += 1;

        let e = format_file_ratio_sum
            .entry(format_key.clone())
            .or_insert((0.0, 0));
        e.0 += record.recovery_ratio_files;
        e.1 += 1;

        let e = format_byte_ratio_sum.entry(format_key).or_insert((0.0, 0));
        e.0 += record.recovery_ratio_bytes;
        e.1 += 1;

        let corr = record.corruption_type.slug().to_string();
        let e = corruption_ratio_sum.entry(corr).or_insert((0.0, 0));
        e.0 += record.recovery_ratio_files;
        e.1 += 1;

        let tgt = record.target_class.slug().to_string();
        let e = target_ratio_sum.entry(tgt).or_insert((0.0, 0));
        e.0 += record.recovery_ratio_files;
        e.1 += 1;
    }

    let per_format_average_recovery_ratio_files = average_map(&format_file_ratio_sum);
    let per_format_average_recovery_ratio_bytes = average_map(&format_byte_ratio_sum);
    let per_corruption_type_average_recovery_ratio_files = average_map(&corruption_ratio_sum);
    let per_target_average_recovery_ratio_files = average_map(&target_ratio_sum);

    let mut per_format_notes = BTreeMap::new();
    for format in ["crushr", "zip", "tar+zstd", "tar+gz", "tar+xz"] {
        per_format_notes.insert(
            format.to_string(),
            "recovery accounting is computed from extracted file presence and byte counts; content checksums are not yet validated".to_string(),
        );
    }

    let mut mapping_report = BTreeMap::new();
    mapping_report.insert(
        "recovered_file_rule".to_string(),
        "a file counts as recovered only when a same-relative-path regular file exists in extraction output; zero-byte files count as recovered when expected size is zero".to_string(),
    );
    mapping_report.insert(
        "recovered_byte_rule".to_string(),
        "bytes_recovered sums min(actual_size, expected_size) for each recovered file; missing files contribute zero bytes".to_string(),
    );
    mapping_report.insert(
        "blast_radius_thresholds".to_string(),
        "NONE=1.0, LOCALIZED=[0.9,1.0), PARTIAL_SET=[0.5,0.9), WIDESPREAD=(0,0.5), TOTAL=0.0 using recovery_ratio_files".to_string(),
    );

    NormalizationSummary {
        total_normalized_runs: records.len(),
        result_class_counts,
        failure_stage_counts,
        diagnostic_specificity_counts,
        per_format_result_class_counts,
        runs_with_recovery_accounting: records.len(),
        recovery_evidence_strength_counts,
        per_format_average_recovery_ratio_files,
        per_format_average_recovery_ratio_bytes,
        per_format_blast_radius_class_counts,
        per_corruption_type_average_recovery_ratio_files,
        per_target_average_recovery_ratio_files,
        per_format_notes,
        mapping_report,
    }
}

fn average_map(source: &BTreeMap<String, (f64, usize)>) -> BTreeMap<String, f64> {
    source
        .iter()
        .map(|(k, (sum, count))| {
            (
                k.clone(),
                if *count == 0 {
                    0.0
                } else {
                    sum / *count as f64
                },
            )
        })
        .collect()
}

fn archive_format_label(value: ArchiveFormat) -> &'static str {
    match value {
        ArchiveFormat::Crushr => "crushr",
        ArchiveFormat::Zip => "zip",
        ArchiveFormat::TarZstd => "tar+zstd",
        ArchiveFormat::TarGz => "tar+gz",
        ArchiveFormat::TarXz => "tar+xz",
    }
}

fn result_class_label(value: ResultClass) -> &'static str {
    match value {
        ResultClass::Success => "SUCCESS",
        ResultClass::Partial => "PARTIAL",
        ResultClass::Refused => "REFUSED",
        ResultClass::StructuralFail => "STRUCTURAL_FAIL",
        ResultClass::ToolError => "TOOL_ERROR",
    }
}

fn failure_stage_label(value: FailureStage) -> &'static str {
    match value {
        FailureStage::None => "NONE",
        FailureStage::PreExtract => "PRE_EXTRACT",
        FailureStage::Extraction => "EXTRACTION",
        FailureStage::Unknown => "UNKNOWN",
    }
}

fn recovery_evidence_strength_label(value: RecoveryEvidenceStrength) -> &'static str {
    match value {
        RecoveryEvidenceStrength::FilePresenceOnly => "FILE_PRESENCE_ONLY",
        RecoveryEvidenceStrength::FileAndByteCounts => "FILE_AND_BYTE_COUNTS",
        RecoveryEvidenceStrength::FileByteAndContentValidation => {
            "FILE_BYTE_AND_CONTENT_VALIDATION"
        }
    }
}

fn blast_radius_label(value: BlastRadiusClass) -> &'static str {
    match value {
        BlastRadiusClass::None => "NONE",
        BlastRadiusClass::Localized => "LOCALIZED",
        BlastRadiusClass::PartialSet => "PARTIAL_SET",
        BlastRadiusClass::Widespread => "WIDESPREAD",
        BlastRadiusClass::Total => "TOTAL",
    }
}

fn diagnostic_label(value: DiagnosticSpecificity) -> &'static str {
    match value {
        DiagnosticSpecificity::None => "NONE",
        DiagnosticSpecificity::Generic => "GENERIC",
        DiagnosticSpecificity::Structural => "STRUCTURAL",
        DiagnosticSpecificity::Precise => "PRECISE",
    }
}

pub fn validate_normalized_results_shape(value: &Value) -> Result<()> {
    let records = value
        .as_array()
        .context("normalized results root must be an array")?;
    for record in records {
        let obj = record
            .as_object()
            .context("normalized run record must be an object")?;
        for field in [
            "scenario_id",
            "dataset",
            "format",
            "corruption_type",
            "target_class",
            "magnitude",
            "magnitude_bytes",
            "seed",
            "tool_kind",
            "exit_code",
            "has_json_result",
            "result_completeness",
            "detected_pre_extract",
            "failure_stage",
            "result_class",
            "diagnostic_specificity",
            "files_expected",
            "files_recovered",
            "files_missing",
            "bytes_expected",
            "bytes_recovered",
            "recovery_ratio_files",
            "recovery_ratio_bytes",
            "blast_radius_class",
            "normalization_notes",
            "recovery_evidence_strength",
        ] {
            if !obj.contains_key(field) {
                bail!("normalized run record missing required field `{field}`");
            }
        }
    }
    Ok(())
}

pub fn validate_normalization_summary_shape(value: &Value) -> Result<()> {
    let obj = value
        .as_object()
        .context("normalization summary root must be an object")?;
    for field in [
        "total_normalized_runs",
        "result_class_counts",
        "failure_stage_counts",
        "diagnostic_specificity_counts",
        "per_format_result_class_counts",
        "runs_with_recovery_accounting",
        "recovery_evidence_strength_counts",
        "per_format_average_recovery_ratio_files",
        "per_format_average_recovery_ratio_bytes",
        "per_format_blast_radius_class_counts",
        "per_corruption_type_average_recovery_ratio_files",
        "per_target_average_recovery_ratio_files",
        "per_format_notes",
        "mapping_report",
    ] {
        if !obj.contains_key(field) {
            bail!("normalization summary missing required field `{field}`");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::phase2_domain::{ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass};
    use crate::phase2_runner::{
        EvidenceQuality, InvocationStatus, RawRunRecord, RecoveryAccounting, ResultArtifacts,
        RunContextPaths, ToolVersionObservation, ToolVersionStatus,
    };

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    fn create_sample_trials_dir() -> PathBuf {
        let unique = format!(
            "phase2-normalize-contract-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_nanos()
        );
        let trials = std::env::temp_dir().join(unique);
        fs::create_dir_all(trials.join("raw/scenario")).expect("create test trials dir");

        let mut b_record = sample_raw_record();
        b_record.scenario_id = "p2-core-smallfiles-crushr-bit_flip-header-1B-2000".to_string();
        b_record.seed = 2000;
        b_record.stdout_path = "raw/scenario/b_stdout.txt".to_string();
        b_record.stderr_path = "raw/scenario/b_stderr.txt".to_string();
        b_record.json_result_path = Some("raw/scenario/b_result.json".to_string());

        let mut a_record = sample_raw_record();
        a_record.scenario_id = "p2-core-smallfiles-crushr-bit_flip-header-1B-1000".to_string();
        a_record.seed = 1000;
        a_record.stdout_path = "raw/scenario/a_stdout.txt".to_string();
        a_record.stderr_path = "raw/scenario/a_stderr.txt".to_string();
        a_record.json_result_path = Some("raw/scenario/a_result.json".to_string());

        fs::write(
            trials.join("raw_run_records.json"),
            serde_json::to_vec_pretty(&vec![b_record, a_record]).expect("serialize raw records"),
        )
        .expect("write raw records");
        fs::write(trials.join("raw/scenario/a_stdout.txt"), "").expect("write a stdout");
        fs::write(trials.join("raw/scenario/a_stderr.txt"), "").expect("write a stderr");
        fs::write(trials.join("raw/scenario/a_result.json"), "{}").expect("write a result");
        fs::write(trials.join("raw/scenario/b_stdout.txt"), "").expect("write b stdout");
        fs::write(trials.join("raw/scenario/b_stderr.txt"), "").expect("write b stderr");
        fs::write(trials.join("raw/scenario/b_result.json"), "{}").expect("write b result");

        trials
    }

    fn sample_raw_record() -> RawRunRecord {
        RawRunRecord {
            scenario_id: "p2-core-smallfiles-crushr-bit_flip-header-1B-1337".to_string(),
            dataset: Dataset::Smallfiles,
            format: ArchiveFormat::Crushr,
            corruption_type: CorruptionType::BitFlip,
            target_class: TargetClass::Header,
            magnitude: Magnitude::OneByte,
            magnitude_bytes: 1,
            seed: 1337,
            source_archive_path: "baselines/crushr/smallfiles.crs".to_string(),
            corrupted_archive_path: "raw/scenario/corrupt.crs".to_string(),
            tool_kind: ArchiveFormat::Crushr,
            executable: "crushr-extract".to_string(),
            argv: vec!["--json".to_string()],
            cwd: None,
            exit_code: 0,
            stdout_path: "raw/scenario/stdout.txt".to_string(),
            stderr_path: "raw/scenario/stderr.txt".to_string(),
            json_result_path: Some("raw/scenario/result.json".to_string()),
            has_json_result: true,
            invocation_status: InvocationStatus::Completed,
            stage_classification: None,
            tool_version: ToolVersionObservation {
                status: ToolVersionStatus::Detected,
                version: Some("v".to_string()),
                detail: None,
            },
            result_artifacts: ResultArtifacts {
                stdout_path: "raw/scenario/stdout.txt".to_string(),
                stderr_path: "raw/scenario/stderr.txt".to_string(),
                json_result_path: Some("raw/scenario/result.json".to_string()),
            },
            result_completeness: EvidenceQuality::StructuredJsonResult,
            run_context_paths: RunContextPaths {
                source_archive_path: "baselines/crushr/smallfiles.crs".to_string(),
                corrupted_archive_path: "raw/scenario/corrupt.crs".to_string(),
                corruption_log_path: "raw/scenario/corruption_provenance.json".to_string(),
            },
            extraction_output_dir: "raw/scenario/extracted".to_string(),
            recovery_report_path: "raw/scenario/recovery_report.json".to_string(),
            recovery_accounting: RecoveryAccounting {
                files_expected: 10,
                files_recovered: 10,
                files_missing: 0,
                bytes_expected: 1000,
                bytes_recovered: 1000,
                recovery_ratio_files: 1.0,
                recovery_ratio_bytes: 1.0,
            },
        }
    }

    #[test]
    fn classifies_structural_failure_from_crushr_parse_error() {
        let diag = "parse FTR4: bad footer magic".to_lowercase();
        assert_eq!(classify_failure_stage(2, &diag), FailureStage::PreExtract);
        assert_eq!(
            classify_diagnostic_specificity(&diag),
            DiagnosticSpecificity::Precise
        );
    }

    #[test]
    fn classifies_refusal_for_extraction_stage_zip_signal() {
        let diag =
            "error: invalid compressed data to inflate payload/large_text.txt".to_lowercase();
        let stage = classify_failure_stage(3, &diag);
        assert_eq!(stage, FailureStage::Extraction);
        assert_eq!(
            classify_diagnostic_specificity(&diag),
            DiagnosticSpecificity::Precise
        );
    }

    #[test]
    fn diagnostic_specificity_mapping_is_deterministic() {
        assert_eq!(
            classify_diagnostic_specificity(""),
            DiagnosticSpecificity::None
        );
        assert_eq!(
            classify_diagnostic_specificity("archive checksum mismatch"),
            DiagnosticSpecificity::Structural
        );
        assert_eq!(
            classify_diagnostic_specificity("warning happened"),
            DiagnosticSpecificity::Generic
        );
    }

    #[test]
    fn blast_radius_thresholds_are_deterministic() {
        assert_eq!(classify_blast_radius(1.0), BlastRadiusClass::None);
        assert_eq!(classify_blast_radius(0.95), BlastRadiusClass::Localized);
        assert_eq!(classify_blast_radius(0.75), BlastRadiusClass::PartialSet);
        assert_eq!(classify_blast_radius(0.25), BlastRadiusClass::Widespread);
        assert_eq!(classify_blast_radius(0.0), BlastRadiusClass::Total);
    }

    #[test]
    fn normalize_record_uses_recovery_accounting_full_partial_zero() {
        let tmp = std::env::temp_dir().join("crushr_norm_recovery_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("raw/scenario")).expect("mkdir");
        fs::write(tmp.join("raw/scenario/stdout.txt"), "{}").expect("stdout");
        fs::write(tmp.join("raw/scenario/stderr.txt"), "").expect("stderr");
        fs::write(tmp.join("raw/scenario/result.json"), "{}").expect("json");

        let mut full = sample_raw_record();
        let full_norm = normalize_record(&tmp, &full).expect("full");
        assert_eq!(full_norm.files_recovered, 10);
        assert_eq!(full_norm.bytes_recovered, 1000);
        assert_eq!(full_norm.blast_radius_class, BlastRadiusClass::None);
        assert_eq!(
            full_norm.recovery_evidence_strength,
            RecoveryEvidenceStrength::FileAndByteCounts
        );

        full.recovery_accounting.files_recovered = 4;
        full.recovery_accounting.files_missing = 6;
        full.recovery_accounting.bytes_recovered = 450;
        full.recovery_accounting.recovery_ratio_files = 0.4;
        full.recovery_accounting.recovery_ratio_bytes = 0.45;
        let partial_norm = normalize_record(&tmp, &full).expect("partial");
        assert_eq!(
            partial_norm.blast_radius_class,
            BlastRadiusClass::Widespread
        );

        full.recovery_accounting.files_recovered = 0;
        full.recovery_accounting.files_missing = 10;
        full.recovery_accounting.bytes_recovered = 0;
        full.recovery_accounting.recovery_ratio_files = 0.0;
        full.recovery_accounting.recovery_ratio_bytes = 0.0;
        let zero_norm = normalize_record(&tmp, &full).expect("zero");
        assert_eq!(zero_norm.blast_radius_class, BlastRadiusClass::Total);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn summary_includes_recovery_aggregates() {
        let records = vec![
            NormalizedRunRecord {
                scenario_id: "a".to_string(),
                dataset: Dataset::Smallfiles,
                format: ArchiveFormat::Crushr,
                corruption_type: CorruptionType::BitFlip,
                target_class: TargetClass::Header,
                magnitude: Magnitude::OneByte,
                magnitude_bytes: 1,
                seed: 1,
                tool_kind: ArchiveFormat::Crushr,
                exit_code: 0,
                has_json_result: true,
                result_completeness: "structured_json_result".to_string(),
                detected_pre_extract: false,
                failure_stage: FailureStage::None,
                result_class: ResultClass::Success,
                diagnostic_specificity: DiagnosticSpecificity::Precise,
                files_expected: 10,
                files_recovered: 10,
                files_missing: 0,
                bytes_expected: 100,
                bytes_recovered: 100,
                recovery_ratio_files: 1.0,
                recovery_ratio_bytes: 1.0,
                blast_radius_class: BlastRadiusClass::None,
                normalization_notes: vec![],
                recovery_evidence_strength: RecoveryEvidenceStrength::FileAndByteCounts,
            },
            NormalizedRunRecord {
                scenario_id: "b".to_string(),
                dataset: Dataset::Smallfiles,
                format: ArchiveFormat::Crushr,
                corruption_type: CorruptionType::BitFlip,
                target_class: TargetClass::Header,
                magnitude: Magnitude::OneByte,
                magnitude_bytes: 1,
                seed: 2,
                tool_kind: ArchiveFormat::Crushr,
                exit_code: 1,
                has_json_result: true,
                result_completeness: "structured_json_result".to_string(),
                detected_pre_extract: false,
                failure_stage: FailureStage::Extraction,
                result_class: ResultClass::Partial,
                diagnostic_specificity: DiagnosticSpecificity::Structural,
                files_expected: 10,
                files_recovered: 5,
                files_missing: 5,
                bytes_expected: 100,
                bytes_recovered: 60,
                recovery_ratio_files: 0.5,
                recovery_ratio_bytes: 0.6,
                blast_radius_class: BlastRadiusClass::PartialSet,
                normalization_notes: vec![],
                recovery_evidence_strength: RecoveryEvidenceStrength::FileAndByteCounts,
            },
        ];
        let summary = build_summary(&records);
        assert_eq!(summary.runs_with_recovery_accounting, 2);
        assert_eq!(
            summary
                .per_format_average_recovery_ratio_files
                .get("crushr"),
            Some(&0.75)
        );
        assert_eq!(
            summary
                .recovery_evidence_strength_counts
                .get("FILE_AND_BYTE_COUNTS"),
            Some(&2)
        );
    }

    #[test]
    fn normalized_output_is_sorted_by_scenario_id() {
        let trials = create_sample_trials_dir();
        let corpus = normalize_from_trials(&trials).expect("normalize");
        let ids = corpus
            .records
            .iter()
            .map(|r| r.scenario_id.clone())
            .collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
        let _ = fs::remove_dir_all(&trials);
    }

    #[test]
    fn normalization_shapes_validate() {
        let trials = create_sample_trials_dir();
        let corpus = normalize_from_trials(&trials).expect("normalize");
        let summary = build_summary(&corpus.records);
        validate_normalized_results_shape(&serde_json::to_value(&corpus.records).unwrap())
            .expect("normalized results shape valid");
        validate_normalization_summary_shape(&serde_json::to_value(&summary).unwrap())
            .expect("summary shape valid");
        let _ = fs::remove_dir_all(&trials);
    }

    #[test]
    fn normalized_schemas_have_expected_ids() {
        let root = workspace_root();
        let results_schema: Value = serde_json::from_slice(
            &fs::read(root.join(NORMALIZED_RESULTS_SCHEMA_PATH)).expect("read results schema"),
        )
        .expect("parse results schema");
        assert_eq!(results_schema["$id"], NORMALIZED_RESULTS_SCHEMA_ID);

        let summary_schema: Value = serde_json::from_slice(
            &fs::read(root.join(NORMALIZED_SUMMARY_SCHEMA_PATH)).expect("read summary schema"),
        )
        .expect("parse summary schema");
        assert_eq!(summary_schema["$id"], NORMALIZED_SUMMARY_SCHEMA_ID);
    }

    #[test]
    fn normalizes_known_crushr_and_comparator_cases() {
        let unique = format!(
            "phase2-normalize-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_nanos()
        );
        let trials = std::env::temp_dir().join(unique);
        fs::create_dir_all(trials.join("raw/scenario")).expect("create test trials dir");

        let mut crushr = sample_raw_record();
        crushr.scenario_id = "p2-core-smallfiles-crushr-bit_flip-header-1B-1337".to_string();
        crushr.stdout_path = "raw/scenario/crushr_stdout.txt".to_string();
        crushr.stderr_path = "raw/scenario/crushr_stderr.txt".to_string();
        crushr.json_result_path = Some("raw/scenario/crushr_result.json".to_string());
        crushr.has_json_result = true;
        crushr.exit_code = 0;

        let mut zip = sample_raw_record();
        zip.scenario_id = "p2-core-smallfiles-zip-bit_flip-header-1B-1337".to_string();
        zip.format = ArchiveFormat::Zip;
        zip.tool_kind = ArchiveFormat::Zip;
        zip.stdout_path = "raw/scenario/zip_stdout.txt".to_string();
        zip.stderr_path = "raw/scenario/zip_stderr.txt".to_string();
        zip.json_result_path = None;
        zip.has_json_result = false;
        zip.exit_code = 2;

        fs::write(trials.join(&crushr.stdout_path), "ok").expect("write crushr stdout");
        fs::write(trials.join(&crushr.stderr_path), "").expect("write crushr stderr");
        fs::write(
            trials.join(crushr.json_result_path.as_ref().expect("crushr json path")),
            "{}",
        )
        .expect("write crushr result");

        fs::write(trials.join(&zip.stdout_path), "").expect("write zip stdout");
        fs::write(
            trials.join(&zip.stderr_path),
            "End-of-central-directory signature not found",
        )
        .expect("write zip stderr");

        fs::write(
            trials.join("raw_run_records.json"),
            serde_json::to_vec_pretty(&vec![crushr, zip]).expect("serialize records"),
        )
        .expect("write raw records");

        let corpus = normalize_from_trials(&trials).expect("normalize");

        let crushr = corpus
            .records
            .iter()
            .find(|r| r.scenario_id == "p2-core-smallfiles-crushr-bit_flip-header-1B-1337")
            .expect("crushr sample");
        assert_eq!(crushr.result_class, ResultClass::Success);
        assert!(crushr.has_json_result);

        let zip = corpus
            .records
            .iter()
            .find(|r| r.scenario_id == "p2-core-smallfiles-zip-bit_flip-header-1B-1337")
            .expect("zip sample");
        assert!(!zip.has_json_result);
        assert_ne!(zip.result_class, ResultClass::Success);

        fs::remove_dir_all(&trials).expect("cleanup test trials dir");
    }
}
