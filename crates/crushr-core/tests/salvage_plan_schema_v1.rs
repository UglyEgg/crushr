// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

static BUILD_ONCE: Once = Once::new();

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn ensure_bins_built() {
    BUILD_ONCE.call_once(|| {
        let out = Command::new("cargo")
            .current_dir(workspace_root())
            .args(["build", "-q", "-p", "crushr", "--bins"])
            .output()
            .expect("run cargo build");
        assert!(
            out.status.success(),
            "cargo build failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    });
}

fn run_bin(bin: &str, args: &[&str]) -> std::process::Output {
    let bin_path = workspace_root().join(format!("target/debug/{bin}"));
    Command::new(bin_path).args(args).output().unwrap()
}

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn load_schema(path: &str) -> JSONSchema {
    let abs = workspace_root().join(path);
    let bytes = fs::read(abs).unwrap();
    let schema: Value = serde_json::from_slice(&bytes).unwrap();
    JSONSchema::compile(&schema).unwrap()
}

fn unique_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

fn make_single_file_archive(root: &Path, name: &str, body: &[u8]) -> PathBuf {
    let src = root.join(name);
    fs::write(&src, body).unwrap();
    let archive = root.join(format!("{name}.crs"));
    let out = run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&out);
    archive
}

fn run_salvage_json(archive: &Path) -> Value {
    let out = run_bin("crushr-salvage", &[archive.to_str().unwrap(), "--json"]);
    assert_ok(&out);
    serde_json::from_slice(&out.stdout).unwrap()
}

fn run_salvage_json_with_export(archive: &Path, export_dir: &Path) -> Value {
    let out = run_bin(
        "crushr-salvage",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--export-fragments",
            export_dir.to_str().unwrap(),
        ],
    );
    assert_ok(&out);
    serde_json::from_slice(&out.stdout).unwrap()
}

fn find_candidate(plan: &Value, offset: u64) -> &Value {
    plan["block_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["scan_offset"].as_u64() == Some(offset))
        .expect("candidate at expected offset")
}

#[test]
fn salvage_plan_json_validates_and_is_deterministic_for_clean_archive() {
    ensure_bins_built();
    let schema = load_schema("schemas/crushr-salvage-plan.v3.schema.json");

    let root = unique_dir("crushr-salvage-clean");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "clean.txt", b"clean\n");

    let first = run_salvage_json(&archive);
    let second = run_salvage_json(&archive);

    assert_eq!(first, second, "salvage plan output must be deterministic");
    if let Err(errs) = schema.validate(&first) {
        let rendered = errs.map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
        panic!("schema validation failed:\n{rendered}\ninstance={first}");
    }

    let candidate = find_candidate(&first, 0);
    assert_eq!(candidate["content_verification_status"], "content_verified");
    assert_eq!(candidate["decompression_status"], "success");
    assert_eq!(candidate["raw_hash_status"], "verified");

    let files = first["file_plans"].as_array().unwrap();
    assert!(!files.is_empty(), "expected at least one file plan");
    assert_eq!(files[0]["status"], Value::String("SALVAGEABLE".to_string()));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn decompression_failure_is_recorded_deterministically() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-decode-fail");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "decode.txt", b"decode\n");
    let mut bytes = fs::read(&archive).unwrap();
    let header_len = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    bytes[header_len] ^= 0xff;

    let broken = root.join("decode-broken.crs");
    fs::write(&broken, bytes).unwrap();

    let plan = run_salvage_json(&broken);
    let candidate = find_candidate(&plan, 0);
    assert_eq!(candidate["decompression_status"], "failed");
    assert_eq!(
        candidate["content_verification_status"],
        "not_content_verified"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn raw_hash_mismatch_blocks_content_verification() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-raw-hash");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "raw.txt", b"raw\n");
    let mut bytes = fs::read(&archive).unwrap();
    bytes[68] ^= 0x01;

    let broken = root.join("raw-mismatch.crs");
    fs::write(&broken, bytes).unwrap();

    let plan = run_salvage_json(&broken);
    let candidate = find_candidate(&plan, 0);
    assert_eq!(candidate["decompression_status"], "success");
    assert_eq!(candidate["raw_hash_status"], "mismatch");
    assert_eq!(
        candidate["content_verification_status"],
        "not_content_verified"
    );

    let files = plan["file_plans"].as_array().unwrap();
    assert!(!files.is_empty());
    assert_eq!(files[0]["status"], "UNSALVAGEABLE");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn damaged_footer_keeps_candidates_but_does_not_invent_file_mappings() {
    ensure_bins_built();

    let root = unique_dir("crushr-salvage-damaged-footer");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "footer.txt", b"footer\n");
    let mut bytes = fs::read(&archive).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    let broken = root.join("broken-footer.crs");
    fs::write(&broken, bytes).unwrap();

    let plan = run_salvage_json(&broken);

    assert!(!plan["block_candidates"].as_array().unwrap().is_empty());
    assert_eq!(plan["index_analysis"]["status"], "unavailable");
    assert!(plan["file_plans"].as_array().unwrap().is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn invalid_idx3_yields_unmappable_orphan_only_plan() {
    ensure_bins_built();

    let root = unique_dir("crushr-salvage-invalid-idx3");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "idx.txt", b"idx\n");
    let mut bytes = fs::read(&archive).unwrap();
    let mut idx_pos = None;
    for i in 0..bytes.len().saturating_sub(3) {
        if &bytes[i..i + 4] == b"IDX3" {
            idx_pos = Some(i);
            break;
        }
    }
    let idx_pos = idx_pos.expect("IDX3 magic should exist");
    bytes[idx_pos] = b'X';

    let broken = root.join("broken-idx.crs");
    fs::write(&broken, bytes).unwrap();

    let plan = run_salvage_json(&broken);

    assert_ne!(plan["index_analysis"]["status"], "valid");
    assert!(plan["file_plans"].as_array().unwrap().is_empty());
    assert!(
        plan["orphan_candidate_summary"]["orphan_unmappable_candidates"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_dictionary_dependency_blocks_salvageability() {
    ensure_bins_built();

    let root = unique_dir("crushr-salvage-missing-dict");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "dict.txt", b"dict\n");
    let mut bytes = fs::read(&archive).unwrap();

    let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
    let uses_dict = flags | (1 << 2);
    bytes[6..8].copy_from_slice(&uses_dict.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());

    let mutated = root.join("dict-required-no-dct1.crs");
    fs::write(&mutated, bytes).unwrap();

    let plan = run_salvage_json(&mutated);

    let candidate = find_candidate(&plan, 0);
    assert_eq!(candidate["dictionary_dependency_status"], "missing");
    assert_eq!(
        candidate["content_verification_status"],
        "not_content_verified"
    );

    let files = plan["file_plans"].as_array().unwrap();
    assert!(!files.is_empty());
    assert_eq!(files[0]["status"], "UNSALVAGEABLE");
    assert_eq!(files[0]["reason"], "required_block_not_content_verified");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn candidate_and_file_ordering_are_stable() {
    ensure_bins_built();

    let root = unique_dir("crushr-salvage-ordering");
    fs::create_dir_all(&root).unwrap();

    let source = root.join("dir");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("z.txt"), b"z").unwrap();
    fs::write(source.join("a.txt"), b"a").unwrap();

    let archive = root.join("ordered.crs");
    let packed = run_bin(
        "crushr-pack",
        &[
            source.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ],
    );
    assert_ok(&packed);

    let plan = run_salvage_json(&archive);

    let candidates = plan["block_candidates"].as_array().unwrap();
    let offsets = candidates
        .iter()
        .map(|c| c["scan_offset"].as_u64().unwrap())
        .collect::<Vec<_>>();
    let mut sorted_offsets = offsets.clone();
    sorted_offsets.sort_unstable();
    assert_eq!(offsets, sorted_offsets);

    let files = plan["file_plans"].as_array().unwrap();
    let paths = files
        .iter()
        .map(|f| f["file_path"].as_str().unwrap())
        .collect::<Vec<_>>();
    let mut sorted_paths = paths.clone();
    sorted_paths.sort_unstable();
    assert_eq!(paths, sorted_paths);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn verified_block_exports_payload_and_sidecar() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-block");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "block.txt", b"block\n");
    let export_dir = root.join("out");
    let plan = run_salvage_json_with_export(&archive, &export_dir);

    assert!(export_dir.join("SALVAGE_RESEARCH_OUTPUT.txt").exists());
    let blocks = plan["exported_artifacts"]["exported_block_artifacts"]
        .as_array()
        .unwrap();
    assert!(!blocks.is_empty());

    let block_json = blocks
        .iter()
        .find(|v| v.as_str().unwrap().ends_with(".json"))
        .unwrap()
        .as_str()
        .unwrap();
    let sidecar: Value =
        serde_json::from_slice(&fs::read(export_dir.join(block_json)).unwrap()).unwrap();
    assert_eq!(sidecar["verification_label"], "UNVERIFIED_RESEARCH_OUTPUT");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn export_disabled_does_not_create_artifact_directory() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-disabled");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "disabled.txt", b"disabled\n");
    let _ = run_salvage_json(&archive);
    assert!(!root.join("out").exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn decompression_failure_block_is_not_exported() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-decode-fail");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "decode.txt", b"decode\n");
    let mut bytes = fs::read(&archive).unwrap();
    let header_len = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    bytes[header_len] ^= 0xff;
    let broken = root.join("decode-broken.crs");
    fs::write(&broken, bytes).unwrap();

    let export_dir = root.join("out");
    let plan = run_salvage_json_with_export(&broken, &export_dir);
    assert!(plan["exported_artifacts"]["exported_block_artifacts"]
        .as_array()
        .unwrap()
        .is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn missing_dictionary_dependency_block_is_not_exported() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-dict-fail");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "dict.txt", b"dict\n");
    let mut bytes = fs::read(&archive).unwrap();
    let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
    let uses_dict = flags | (1 << 2);
    bytes[6..8].copy_from_slice(&uses_dict.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    let mutated = root.join("dict-required-no-dct1.crs");
    fs::write(&mutated, bytes).unwrap();

    let export_dir = root.join("out");
    let plan = run_salvage_json_with_export(&mutated, &export_dir);
    assert!(plan["exported_artifacts"]["exported_block_artifacts"]
        .as_array()
        .unwrap()
        .is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn partial_fragment_export_has_extents_but_no_complete_file() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-partial");
    fs::create_dir_all(&root).unwrap();

    let source = root.join("src");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("a.txt"), b"a").unwrap();
    fs::write(source.join("b.txt"), b"b").unwrap();
    let archive = root.join("partial.crs");
    let packed = run_bin(
        "crushr-pack",
        &[source.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let mut bytes = fs::read(&archive).unwrap();
    let second_header = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(i, w)| if w == b"BLK3" { Some(i) } else { None })
        .nth(1)
        .unwrap();
    let header_len =
        u16::from_le_bytes([bytes[second_header + 4], bytes[second_header + 5]]) as usize;
    bytes[second_header + header_len] ^= 0xff;
    let broken = root.join("partial-broken.crs");
    fs::write(&broken, bytes).unwrap();

    let export_dir = root.join("out");
    let plan = run_salvage_json_with_export(&broken, &export_dir);
    assert!(!plan["exported_artifacts"]["exported_fragment_artifacts"]
        .as_array()
        .unwrap()
        .is_empty());
    let fulls = plan["exported_artifacts"]["exported_complete_file_artifacts"]
        .as_array()
        .unwrap();
    let file_count = plan["file_plans"].as_array().unwrap().len();
    assert!(fulls.len() < file_count);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn full_file_export_only_for_fully_verified_file() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-full");
    fs::create_dir_all(&root).unwrap();

    let archive = make_single_file_archive(&root, "full.txt", b"full\n");
    let export_dir = root.join("out");
    let plan = run_salvage_json_with_export(&archive, &export_dir);

    let fulls = plan["exported_artifacts"]["exported_complete_file_artifacts"]
        .as_array()
        .unwrap();
    assert_eq!(fulls.len(), 1);
    assert!(export_dir.join(fulls[0].as_str().unwrap()).exists());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn export_ordering_is_deterministic_across_runs() {
    ensure_bins_built();
    let root = unique_dir("crushr-salvage-export-order");
    fs::create_dir_all(&root).unwrap();

    let source = root.join("src");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("z.txt"), b"z").unwrap();
    fs::write(source.join("a.txt"), b"a").unwrap();
    let archive = root.join("ordered.crs");
    let packed = run_bin(
        "crushr-pack",
        &[source.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let out1 = root.join("out1");
    let out2 = root.join("out2");
    let plan1 = run_salvage_json_with_export(&archive, &out1);
    let plan2 = run_salvage_json_with_export(&archive, &out2);
    assert_eq!(
        plan1["exported_artifacts"], plan2["exported_artifacts"],
        "exported artifact references must be deterministic"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn salvage_schema_enums_cover_emitted_vocabulary() {
    let schema_path = workspace_root().join("schemas/crushr-salvage-plan.v3.schema.json");
    let schema: Value = serde_json::from_slice(&fs::read(schema_path).unwrap()).unwrap();

    let file_plan_props = &schema["properties"]["file_plans"]["items"]["properties"];
    let mapping = file_plan_props["mapping_provenance"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let recovery = file_plan_props["recovery_classification"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let failure_reasons = file_plan_props["failure_reasons"]["items"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();

    let candidate_reasons = schema["properties"]["block_candidates"]["items"]["properties"]
        ["content_verification_reasons"]["items"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();

    let expected_mapping = std::collections::BTreeSet::from([
        "PRIMARY_INDEX_PATH",
        "REDUNDANT_VERIFIED_MAP_PATH",
        "CHECKPOINT_MAP_PATH",
        "SELF_DESCRIBING_EXTENT_PATH",
        "FILE_MANIFEST_PATH",
        "FILE_IDENTITY_EXTENT_PATH",
        "FILE_IDENTITY_EXTENT_PATH_ANONYMOUS",
        "PAYLOAD_BLOCK_IDENTITY_PATH",
        "PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS",
    ]);
    let expected_recovery = std::collections::BTreeSet::from([
        "FULL_VERIFIED",
        "FULL_ANONYMOUS",
        "PARTIAL_ORDERED",
        "PARTIAL_UNORDERED",
        "ORPHAN_BLOCKS",
    ]);
    let expected_reasons = std::collections::BTreeSet::from([
        "all_required_checks_passed",
        "decompression_not_successful",
        "dictionary_dependency_unsatisfied",
        "header_invalid",
        "header_out_of_bounds",
        "header_prefix_out_of_bounds",
        "manifest_digest_not_verified",
        "manifest_expected_blocks_missing",
        "manifest_without_recoverable_extents",
        "no_required_blocks",
        "payload_block_identity_index_gap",
        "payload_block_identity_missing_required_block_coverage",
        "payload_hash_mismatch",
        "payload_out_of_bounds",
        "raw_hash_mismatch",
        "required_block_not_content_verified",
        "required_block_unmapped",
        "required_extent_out_of_bounds",
    ]);

    assert_eq!(mapping, expected_mapping);
    assert_eq!(recovery, expected_recovery);
    assert_eq!(failure_reasons, expected_reasons);
    assert_eq!(candidate_reasons, expected_reasons);
}
