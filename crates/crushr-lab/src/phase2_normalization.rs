use crate::phase2_domain::{ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass};
use crate::phase2_runner::RawRunRecord;
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
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    StructuredJson,
    StdoutStderr,
    Mixed,
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
    pub files_safe: Option<u64>,
    pub files_refused: Option<u64>,
    pub files_unknown: Option<u64>,
    pub normalization_notes: Vec<String>,
    pub evidence_strength: EvidenceStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationSummary {
    pub total_normalized_runs: usize,
    pub result_class_counts: BTreeMap<String, usize>,
    pub failure_stage_counts: BTreeMap<String, usize>,
    pub diagnostic_specificity_counts: BTreeMap<String, usize>,
    pub per_format_result_class_counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub runs_with_file_level_detail: usize,
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

    normalized.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));

    Ok(NormalizedCorpus {
        records: normalized,
    })
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

    let evidence_strength =
        if record.has_json_result && (!stdout.trim().is_empty() || !stderr.trim().is_empty()) {
            EvidenceStrength::Mixed
        } else if record.has_json_result {
            EvidenceStrength::StructuredJson
        } else {
            EvidenceStrength::StdoutStderr
        };

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

    let (files_safe, files_refused, files_unknown) =
        extract_file_level_counts(record, &diag_text, &mut notes);

    if !record.has_json_result {
        notes.push(
            "no structured per-file result artifact available; file-level counts are null"
                .to_string(),
        );
    }

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
        files_safe,
        files_refused,
        files_unknown,
        normalization_notes: notes,
        evidence_strength,
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

fn extract_file_level_counts(
    record: &RawRunRecord,
    _diag_text: &str,
    notes: &mut Vec<String>,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    if !record.has_json_result {
        return (None, None, None);
    }

    notes.push(
        "structured JSON artifact is metadata-oriented (crushr-info), not extraction outcomes; file-level counts remain null"
            .to_string(),
    );
    (None, None, None)
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn build_summary(records: &[NormalizedRunRecord]) -> NormalizationSummary {
    let mut result_class_counts = BTreeMap::<String, usize>::new();
    let mut failure_stage_counts = BTreeMap::<String, usize>::new();
    let mut diagnostic_specificity_counts = BTreeMap::<String, usize>::new();
    let mut per_format_result_class_counts = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut file_level_runs = 0usize;

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
            .entry(format_key)
            .or_default()
            .entry(rc_key)
            .or_insert(0) += 1;

        if record.files_safe.is_some()
            || record.files_refused.is_some()
            || record.files_unknown.is_some()
        {
            file_level_runs += 1;
        }
    }

    let mut per_format_notes = BTreeMap::new();
    per_format_notes.insert(
        "crushr".to_string(),
        "structured JSON exists, but current phase uses crushr-info structural probes (no extraction outcome file counts)".to_string(),
    );
    for format in ["zip", "tar+zstd", "tar+gz", "tar+xz"] {
        per_format_notes.insert(
            format.to_string(),
            "no structured per-file result artifact in execution corpus".to_string(),
        );
    }

    let mut mapping_report = BTreeMap::new();
    mapping_report.insert(
        "exit_code_zero".to_string(),
        "exit_code=0 => SUCCESS, failure_stage=NONE".to_string(),
    );
    mapping_report.insert(
        "pre_extract_markers".to_string(),
        "non-zero exit + structural markers => failure_stage=PRE_EXTRACT; result_class=STRUCTURAL_FAIL unless refusal markers also present".to_string(),
    );
    mapping_report.insert(
        "extraction_markers".to_string(),
        "non-zero exit + extraction markers => failure_stage=EXTRACTION; result_class=REFUSED or PARTIAL based on refusal markers".to_string(),
    );

    NormalizationSummary {
        total_normalized_runs: records.len(),
        result_class_counts,
        failure_stage_counts,
        diagnostic_specificity_counts,
        per_format_result_class_counts,
        runs_with_file_level_detail: file_level_runs,
        per_format_notes,
        mapping_report,
    }
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
            "files_safe",
            "files_refused",
            "files_unknown",
            "normalization_notes",
            "evidence_strength",
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
        "runs_with_file_level_detail",
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

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    fn trials_dir() -> PathBuf {
        workspace_root().join("PHASE2_RESEARCH/trials")
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
    fn normalized_output_is_sorted_by_scenario_id() {
        let corpus = normalize_from_trials(&trials_dir()).expect("normalize");
        let ids = corpus
            .records
            .iter()
            .map(|r| r.scenario_id.clone())
            .collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn normalization_shapes_validate() {
        let corpus = normalize_from_trials(&trials_dir()).expect("normalize");
        let summary = build_summary(&corpus.records);
        validate_normalized_results_shape(&serde_json::to_value(&corpus.records).unwrap())
            .expect("normalized results shape valid");
        validate_normalization_summary_shape(&serde_json::to_value(&summary).unwrap())
            .expect("summary shape valid");
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
        let corpus = normalize_from_trials(&trials_dir()).expect("normalize");

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
    }
}
