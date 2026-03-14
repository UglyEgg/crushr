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
    let out = run_bin("crushr-salvage", &[archive.to_str().unwrap()]);
    assert_ok(&out);
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn salvage_plan_json_validates_and_is_deterministic_for_clean_archive() {
    ensure_bins_built();
    let schema = load_schema("schemas/crushr-salvage-plan.v1.schema.json");

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

    let files = first["file_plans"].as_array().unwrap();
    assert!(!files.is_empty(), "expected at least one file plan");
    assert_eq!(files[0]["status"], Value::String("SALVAGEABLE".to_string()));

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

    // First BLK3 should be at offset 0 for this fixture.
    let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
    let uses_dict = flags | (1 << 2);
    bytes[6..8].copy_from_slice(&uses_dict.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());

    let mutated = root.join("dict-required-no-dct1.crs");
    fs::write(&mutated, bytes).unwrap();

    let plan = run_salvage_json(&mutated);

    let files = plan["file_plans"].as_array().unwrap();
    assert!(!files.is_empty());
    assert_eq!(files[0]["status"], "UNSALVAGEABLE");
    assert_eq!(
        files[0]["reason"],
        "required_dictionary_missing_or_unverified"
    );

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
