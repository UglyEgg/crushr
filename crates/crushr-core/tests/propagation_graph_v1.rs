use serde_json::Value;
use std::collections::BTreeSet;
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

fn as_object<'a>(value: &'a Value, context: &str) -> &'a serde_json::Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object for {context}"))
}

fn as_array<'a>(value: &'a Value, context: &str) -> &'a [Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected array for {context}"))
}

fn assert_sorted_strings(values: &[Value], context: &str) {
    let mut got = Vec::new();
    for v in values {
        got.push(
            v.as_str()
                .unwrap_or_else(|| panic!("{context} must contain strings")),
        );
    }
    let mut sorted = got.clone();
    sorted.sort_unstable();
    assert_eq!(got, sorted, "{context} must be sorted");
}

fn assert_field_set(obj: &serde_json::Map<String, Value>, expected: &[&str], context: &str) {
    let actual = obj.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{context} field set mismatch");
}

fn assert_schema_shape(schema: &Value, instance: &Value) {
    let root = as_object(instance, "report");
    let required = as_array(&schema["required"], "schema.required");
    for field in required {
        let key = field.as_str().unwrap();
        assert!(root.contains_key(key), "missing required field: {key}");
    }
    assert_field_set(
        root,
        &[
            "report_version",
            "format_family",
            "report_kind",
            "corrupted_structure_nodes",
            "corrupted_blocks",
            "nodes",
            "edges",
            "per_file_impacts",
        ],
        "report",
    );

    assert_eq!(instance["report_version"], 1);
    assert_eq!(instance["format_family"], "minimal-v1");
    assert_eq!(instance["report_kind"], "corruption_propagation_graph");

    let structure_nodes = as_array(
        &instance["corrupted_structure_nodes"],
        "corrupted_structure_nodes",
    );
    assert_sorted_strings(structure_nodes, "corrupted_structure_nodes");
    let allowed_assumed =
        BTreeSet::from(["structure:ftr4", "structure:tail_frame", "structure:idx3"]);
    for value in structure_nodes {
        let s = value.as_str().unwrap();
        assert!(
            allowed_assumed.contains(s),
            "unexpected structure node: {s}"
        );
    }

    let corrupted_blocks = as_array(&instance["corrupted_blocks"], "corrupted_blocks");
    let mut last = None;
    for block in corrupted_blocks {
        let value = block.as_u64().expect("corrupted block must be integer");
        if let Some(prev) = last {
            assert!(
                prev < value,
                "corrupted_blocks must be strictly ascending and unique"
            );
        }
        last = Some(value);
    }

    let allowed_node_kinds = BTreeSet::from(["footer", "tail_frame", "index", "block", "file"]);
    for node in as_array(&instance["nodes"], "nodes") {
        let node = as_object(node, "node");
        assert_field_set(node, &["id", "kind"], "node");
        assert!(node.get("id").and_then(Value::as_str).is_some());
        let kind = node["kind"].as_str().unwrap();
        assert!(
            allowed_node_kinds.contains(kind),
            "unexpected node kind: {kind}"
        );
    }

    let allowed_edge_reasons = BTreeSet::from([
        "required_for_reachability",
        "required_for_index",
        "required_for_extraction",
        "required_data_block",
    ]);
    let edges = as_array(&instance["edges"], "edges");
    let edges_sorted = edges
        .iter()
        .map(|edge| {
            let edge = as_object(edge, "edge");
            assert_field_set(edge, &["from", "to", "reason"], "edge");
            let from = edge["from"].as_str().unwrap().to_string();
            let to = edge["to"].as_str().unwrap().to_string();
            let reason = edge["reason"].as_str().unwrap().to_string();
            assert!(allowed_edge_reasons.contains(reason.as_str()));
            (from, to, reason)
        })
        .collect::<Vec<_>>();
    let mut sorted = edges_sorted.clone();
    sorted.sort();
    assert_eq!(
        edges_sorted, sorted,
        "edges must be deterministically ordered"
    );

    let allowed_impact_reasons =
        BTreeSet::from(["corrupted_required_structure", "corrupted_required_block"]);
    let per_file = as_array(&instance["per_file_impacts"], "per_file_impacts");
    let mut file_paths = Vec::new();
    for item in per_file {
        let item = as_object(item, "per_file_impact");
        assert_field_set(
            item,
            &[
                "file_path",
                "required_nodes",
                "hypothetical_impacts_if_corrupted",
                "actual_impacts_from_current_corruption",
            ],
            "per_file_impact",
        );

        let file_path = item["file_path"].as_str().unwrap().to_string();
        file_paths.push(file_path);

        for key in [
            "required_nodes",
            "hypothetical_impacts_if_corrupted",
            "actual_impacts_from_current_corruption",
        ] {
            assert!(item[key].is_array(), "{key} must be array");
        }

        for required_node in as_array(&item["required_nodes"], "required_nodes") {
            assert!(required_node.as_str().is_some());
        }

        for causes_key in [
            "hypothetical_impacts_if_corrupted",
            "actual_impacts_from_current_corruption",
        ] {
            for cause in as_array(&item[causes_key], causes_key) {
                let cause = as_object(cause, causes_key);
                assert_field_set(cause, &["cause_node", "reason"], causes_key);
                assert!(cause.get("cause_node").and_then(Value::as_str).is_some());
                let reason = cause["reason"].as_str().unwrap();
                assert!(allowed_impact_reasons.contains(reason));
            }
        }
    }

    let mut sorted_paths = file_paths.clone();
    sorted_paths.sort_unstable();
    assert_eq!(
        file_paths, sorted_paths,
        "per_file_impacts must be path-sorted"
    );
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
    assert_eq!(report["corrupted_structure_nodes"], serde_json::json!([]));

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

    let safe: Vec<String> = extract_json["safe_files"]
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
    for file in &safe {
        assert!(
            !impacted.contains(file),
            "safe file should not be marked impacted: {file}"
        );
    }
    assert_eq!(report["corrupted_blocks"].as_array().unwrap().len(), 1);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn propagation_report_boundary_for_structural_open_failure_is_explicit() {
    ensure_bins_built();

    let root = unique_dir("crushr-propagation-structural-boundary");
    fs::create_dir_all(&root).unwrap();
    let source = root.join("one.txt");
    fs::write(&source, b"data\n").unwrap();

    let archive = root.join("one.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[source.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let mut bytes = fs::read(&archive).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    fs::write(&archive, bytes).unwrap();

    let info = run_bin(
        "crushr-info",
        &[
            archive.to_str().unwrap(),
            "--json",
            "--report",
            "propagation",
        ],
    );
    assert_ok(&info);
    let report = parse_json(&info);
    let nodes = report["corrupted_structure_nodes"].as_array().unwrap();
    assert!(
        nodes.iter().any(|v| v.as_str() == Some("structure:ftr4")
            || v.as_str() == Some("structure:tail_frame")
            || v.as_str() == Some("structure:idx3")),
        "expected at least one structural corruption node"
    );

    let extract_out_dir = root.join("out");
    let extract = run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            extract_out_dir.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(extract.status.code(), Some(2));

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
