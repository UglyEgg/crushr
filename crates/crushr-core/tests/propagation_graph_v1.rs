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

fn unique_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
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

fn parse_json(out: &std::process::Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap()
}

fn load_schema() -> Value {
    let path = workspace_root().join("schemas/crushr-propagation-graph.v1.schema.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn assert_schema_shape(schema: &Value, instance: &Value) {
    let obj = instance.as_object().expect("report object");
    for field in schema["required"].as_array().unwrap() {
        let key = field.as_str().unwrap();
        assert!(obj.contains_key(key), "missing required field: {key}");
    }
    assert_eq!(instance["report_version"], 1);
    assert_eq!(instance["format_family"], "minimal-v1");
    assert_eq!(instance["report_kind"], "corruption_propagation_graph");
}

#[test]
fn propagation_report_healthy_archive_has_deterministic_graph_shape() {
    ensure_bins_built();
    let schema = load_schema();

    let root = unique_dir("crushr-propagation-healthy");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.txt"), b"alpha\n").unwrap();
    fs::write(src.join("b.txt"), b"bravo\n").unwrap();

    let archive = root.join("ok.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let out = run_bin(
        "crushr-info",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--report",
            "propagation",
        ],
    );
    assert_ok(&out);

    let report = parse_json(&out);
    assert_schema_shape(&schema, &report);

    assert_eq!(report["nodes"][0]["id"], "structure:ftr4");
    assert_eq!(report["nodes"][1]["id"], "structure:tail_frame");
    assert_eq!(report["nodes"][2]["id"], "structure:idx3");
    assert_eq!(report["per_file_impacts"][0]["file_path"], "a.txt");
    assert_eq!(report["corrupted_blocks"], serde_json::json!([]));

    let edges = report["edges"].as_array().unwrap();
    let mut sorted = edges.clone();
    sorted.sort_by_key(|e| {
        (
            e["from"].as_str().unwrap().to_string(),
            e["to"].as_str().unwrap().to_string(),
            e["reason"].as_str().unwrap().to_string(),
        )
    });
    assert_eq!(edges, &sorted);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn propagation_report_matches_extract_refusal_for_single_corrupted_block() {
    ensure_bins_built();

    let root = unique_dir("crushr-propagation-corrupt");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("x.txt"), b"x data\n").unwrap();
    fs::write(src.join("y.txt"), b"y data\n").unwrap();

    let archive = root.join("two.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let mut bytes = fs::read(&archive).unwrap();
    let first = bytes.iter().position(|b| *b == b'x').unwrap();
    bytes[first] ^= 1;
    fs::write(&archive, bytes).unwrap();

    let report_out = run_bin(
        "crushr-info",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--report",
            "propagation",
        ],
    );
    assert_ok(&report_out);
    let report = parse_json(&report_out);

    let extract_out_dir = root.join("out");
    let extract_out = run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            extract_out_dir.to_str().unwrap(),
            "--json",
        ],
    );
    assert_ok(&extract_out);
    let extract_json = parse_json(&extract_out);

    let refused: Vec<String> = extract_json["refused_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["path"].as_str().unwrap().to_string())
        .collect();

    let impacted: Vec<String> = report["per_file_impacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            !f["actual_impacts_from_current_corruption"]
                .as_array()
                .unwrap()
                .is_empty()
        })
        .map(|f| f["file_path"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(impacted, refused);
    assert!(report["corrupted_blocks"].as_array().unwrap().len() == 1);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn propagation_report_json_is_byte_stable_for_same_archive() {
    ensure_bins_built();

    let root = unique_dir("crushr-propagation-determinism");
    let file = root.join("one.txt");
    fs::create_dir_all(&root).unwrap();
    fs::write(&file, b"deterministic\n").unwrap();
    let archive = root.join("one.crs");

    assert_ok(&run_bin(
        "crushr-pack",
        &[file.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let a = run_bin(
        "crushr-info",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--report",
            "propagation",
        ],
    );
    assert_ok(&a);
    let b = run_bin(
        "crushr-info",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--report",
            "propagation",
        ],
    );
    assert_ok(&b);

    assert_eq!(a.stdout, b.stdout);

    let _ = fs::remove_dir_all(&root);
}
