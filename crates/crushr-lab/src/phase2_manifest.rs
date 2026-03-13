use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::phase2_domain::{
    locked_core_scenario_id, ArchiveFormat, CorruptionType, Dataset, Magnitude, TargetClass,
    LOCKED_CORE_SEEDS, PHASE2_SCENARIO_ID_FORMAT,
};

pub const PHASE2_MANIFEST_SCHEMA_PATH: &str =
    "schemas/crushr-lab-experiment-manifest.phase2.v1.schema.json";
pub const PHASE2_MANIFEST_SCHEMA_ID: &str =
    "https://crushr.dev/schemas/crushr-lab-experiment-manifest.phase2.v1.schema.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase2Scenario {
    pub scenario_id: String,
    pub dataset: Dataset,
    pub format: ArchiveFormat,
    pub corruption_type: CorruptionType,
    pub target_class: TargetClass,
    pub magnitude: Magnitude,
    pub magnitude_bytes: u64,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase2ExperimentManifest {
    pub schema_version: String,
    pub scenario_id_format: String,
    pub ordering: Vec<String>,
    pub datasets: Vec<Dataset>,
    pub formats: Vec<ArchiveFormat>,
    pub corruption_types: Vec<CorruptionType>,
    pub target_classes: Vec<TargetClass>,
    pub magnitudes: Vec<Magnitude>,
    pub seeds: Vec<u64>,
    pub scenarios: Vec<Phase2Scenario>,
}

pub fn enumerate_locked_core_scenarios() -> Vec<Phase2Scenario> {
    let mut scenarios = Vec::new();
    for dataset in Dataset::ordered_locked_core() {
        for format in ArchiveFormat::ordered_locked_core() {
            for corruption_type in CorruptionType::ordered_locked_core() {
                for target_class in TargetClass::ordered_locked_core() {
                    for magnitude in Magnitude::ordered_locked_core() {
                        for seed in LOCKED_CORE_SEEDS {
                            scenarios.push(Phase2Scenario {
                                scenario_id: locked_core_scenario_id(
                                    *dataset,
                                    *format,
                                    *corruption_type,
                                    *target_class,
                                    *magnitude,
                                    seed,
                                ),
                                dataset: *dataset,
                                format: *format,
                                corruption_type: *corruption_type,
                                target_class: *target_class,
                                magnitude: *magnitude,
                                magnitude_bytes: magnitude.bytes(),
                                seed,
                            });
                        }
                    }
                }
            }
        }
    }
    scenarios
}

impl Phase2ExperimentManifest {
    pub fn locked_core() -> Self {
        Self {
            schema_version: "phase2.v1".to_string(),
            scenario_id_format: PHASE2_SCENARIO_ID_FORMAT.to_string(),
            ordering: vec![
                "dataset".to_string(),
                "format".to_string(),
                "corruption_type".to_string(),
                "target_class".to_string(),
                "magnitude".to_string(),
                "seed".to_string(),
            ],
            datasets: Dataset::ordered_locked_core().to_vec(),
            formats: ArchiveFormat::ordered_locked_core().to_vec(),
            corruption_types: CorruptionType::ordered_locked_core().to_vec(),
            target_classes: TargetClass::ordered_locked_core().to_vec(),
            magnitudes: Magnitude::ordered_locked_core().to_vec(),
            seeds: LOCKED_CORE_SEEDS.to_vec(),
            scenarios: enumerate_locked_core_scenarios(),
        }
    }
}

pub fn validate_manifest_shape(manifest: &Value) -> Result<()> {
    let object = manifest
        .as_object()
        .context("manifest root must be a JSON object")?;

    for required in [
        "$schema",
        "schema_version",
        "scenario_id_format",
        "ordering",
        "datasets",
        "formats",
        "corruption_types",
        "target_classes",
        "magnitudes",
        "seeds",
        "scenarios",
    ] {
        if !object.contains_key(required) {
            bail!("manifest missing required field `{required}`");
        }
    }

    let scenarios = object
        .get("scenarios")
        .and_then(Value::as_array)
        .context("manifest.scenarios must be an array")?;
    if scenarios.len() != 2700 {
        bail!("manifest.scenarios must contain exactly 2700 entries");
    }

    let expected_ids = enumerate_locked_core_scenarios()
        .into_iter()
        .map(|s| s.scenario_id)
        .collect::<Vec<_>>();

    for (index, scenario) in scenarios.iter().enumerate() {
        let item = scenario
            .as_object()
            .context("manifest scenario entries must be objects")?;
        let scenario_id = item
            .get("scenario_id")
            .and_then(Value::as_str)
            .context("scenario.scenario_id must be a string")?;
        if scenario_id != expected_ids[index] {
            bail!("scenario ordering differs from locked core enumeration order");
        }

        for key in [
            "dataset",
            "format",
            "corruption_type",
            "target_class",
            "magnitude",
            "magnitude_bytes",
            "seed",
        ] {
            if !item.contains_key(key) {
                bail!("scenario missing required field `{key}`");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn locked_core_manifest_expands_to_2700_scenarios() {
        let manifest = Phase2ExperimentManifest::locked_core();
        assert_eq!(manifest.scenarios.len(), 2700);
    }

    #[test]
    fn locked_core_enumeration_order_is_deterministic() {
        let manifest = Phase2ExperimentManifest::locked_core();
        assert_eq!(
            manifest.scenarios.first().map(|s| s.scenario_id.as_str()),
            Some("p2-core-smallfiles-crushr-bit_flip-header-1B-1337")
        );
        assert_eq!(
            manifest.scenarios.last().map(|s| s.scenario_id.as_str()),
            Some("p2-core-largefiles-tar_xz-tail_damage-tail-4KB-65535")
        );
    }

    #[test]
    fn scenario_ids_are_deterministic_and_unique() {
        let manifest = Phase2ExperimentManifest::locked_core();
        let mut ids = HashSet::new();
        for scenario in &manifest.scenarios {
            assert!(scenario.scenario_id.starts_with("p2-core-"));
            assert!(ids.insert(scenario.scenario_id.clone()));
        }
        assert_eq!(ids.len(), 2700);
    }

    #[test]
    fn scenario_fields_are_stable_for_known_sample() {
        let manifest = Phase2ExperimentManifest::locked_core();
        let sample = manifest
            .scenarios
            .iter()
            .find(|s| s.scenario_id == "p2-core-mixed-tar_zstd-zero_fill-payload-256B-2600")
            .expect("sample scenario exists");

        assert_eq!(sample.dataset, Dataset::Mixed);
        assert_eq!(sample.format, ArchiveFormat::TarZstd);
        assert_eq!(sample.corruption_type, CorruptionType::ZeroFill);
        assert_eq!(sample.target_class, TargetClass::Payload);
        assert_eq!(sample.magnitude, Magnitude::TwoHundredFiftySixBytes);
        assert_eq!(sample.magnitude_bytes, 256);
        assert_eq!(sample.seed, 2600);
    }

    #[test]
    fn locked_core_manifest_matches_schema_shape() {
        let schema_path = workspace_root().join(PHASE2_MANIFEST_SCHEMA_PATH);
        let schema_json: Value = serde_json::from_slice(&fs::read(schema_path).unwrap()).unwrap();
        assert_eq!(schema_json["$id"], PHASE2_MANIFEST_SCHEMA_ID);

        let mut manifest = serde_json::to_value(Phase2ExperimentManifest::locked_core()).unwrap();
        manifest.as_object_mut().unwrap().insert(
            "$schema".to_string(),
            Value::String(PHASE2_MANIFEST_SCHEMA_ID.to_string()),
        );

        validate_manifest_shape(&manifest).expect("manifest matches phase2 schema shape");
    }
}
