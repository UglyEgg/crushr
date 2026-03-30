// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_ok(cmd: &mut Command) -> String {
    let out = cmd.output().expect("run command");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("stdout utf8")
}

fn run_any(cmd: &mut Command) -> Output {
    cmd.output().expect("run command")
}

fn normalize_paths(text: String, tmp: &Path) -> String {
    text.replace(&tmp.display().to_string(), "<TMP>")
}

fn normalize_info_output(text: String, tmp: &Path) -> String {
    let normalized = normalize_paths(text, tmp);
    normalized
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("blake3") {
                "  blake3                 <DYNAMIC>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn verify_output_is_deterministic_and_uses_shared_status_words() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let first = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&archive),
    );
    let second = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&archive),
    );

    assert_eq!(first, second);
    assert!(first.contains("crushr  /  verify"));
    assert!(first.contains("Progress"));
    assert!(first.contains("archive open / header read"));
    assert!(first.contains("metadata/index scan"));
    assert!(first.contains("payload verification"));
    assert!(first.contains("manifest validation"));
    assert!(first.contains("final result/report"));
    assert!(first.contains("Verification"));
    assert!(first.contains("Result"));
}

#[test]
fn silent_mode_emits_one_line_summary_for_public_commands() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive = tmp.path().join("sample.crushr");
    let extract_out = tmp.path().join("extract");

    let pack_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(pack_out.lines().count(), 1);
    assert!(pack_out.contains("status=COMPLETE"));

    let extract_out_text = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("extract")
            .arg(&archive)
            .arg("-o")
            .arg(&extract_out)
            .arg("--silent"),
    );
    assert_eq!(extract_out_text.lines().count(), 1);
    assert!(extract_out_text.contains("status=COMPLETE"));

    let verify_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(verify_out.lines().count(), 1);
    assert!(verify_out.contains("status=VERIFIED"));

    let salvage_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("salvage")
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(salvage_out.lines().count(), 1);
    assert!(salvage_out.contains("status=DEGRADED"));

    let salvage_human = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .args(["salvage", archive.to_str().expect("utf8")]),
    );
    assert!(salvage_human.contains("Evidence"));
    assert!(salvage_human.contains("verified files"));
    assert!(salvage_human.contains("rejected/unresolved"));
}

#[test]
fn root_help_lists_canonical_suite_and_demotes_legacy_surface() {
    let out = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));

    for token in [
        "pack", "extract", "verify", "info", "about", "salvage", "lab",
    ] {
        assert!(
            out.contains(token),
            "root help missing canonical command token: {token}\n{out}"
        );
    }
    for legacy in [
        "\n  append",
        "\n  list",
        "\n  cat",
        "\n  dict-train",
        "\n  tune",
        "\n  completions",
    ] {
        assert!(
            !out.contains(legacy),
            "root help should demote legacy command: {legacy}\n{out}"
        );
    }
    assert!(!out.contains("mock chart"));
    assert!(!out.contains("Solid-block archive compressor"));
}

#[test]
fn verify_invalid_archive_uses_operator_surface_without_parser_leakage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let archive = tmp.path().join("bad.crushr");
    fs::write(&archive, vec![0u8; 4096]).expect("write invalid archive");

    let out = run_any(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&archive),
    );
    assert!(!out.status.success());

    let stdout = String::from_utf8(out.stdout).expect("stdout utf8");
    let stderr = String::from_utf8(out.stderr).expect("stderr utf8");
    assert!(stdout.contains("Failure domain"));
    assert!(stdout.contains("component"));
    assert!(stdout.contains("archive structure"));
    assert!(!stdout.contains("parse FTR4"));
    assert!(!stdout.contains("bad footer magic"));
    assert!(!stderr.contains("parse FTR4"));
    assert!(!stderr.contains("bad footer magic"));
}

#[test]
fn about_command_matches_locked_output_shape() {
    let out = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("about"));

    assert!(out.contains("crushr  /  about"));
    assert!(out.contains("Build"));
    assert!(out.contains("Behavior"));
    assert!(out.contains("Data Model"));
    assert!(out.contains("Built with"));
    assert!(out.contains("Support"));
    assert!(out.contains("pack"));
    assert!(out.contains("deterministic archive creation"));
    assert!(out.contains("extract"));
    assert!(out.contains("strict extraction (verification-gated)"));
    assert!(out.contains("verify"));
    assert!(out.contains("structural and integrity validation"));
    assert!(out.contains("salvage"));
    assert!(out.contains("research-mode recovery planning (non-canonical)"));
    assert!(out.contains("crushr info <archive> --json"));
    assert!(out.contains("crushr extract --verify <archive>"));
}

#[test]
fn canonical_help_commands_are_available() {
    let pack = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert!(pack.contains("crushr <command>"));

    let extract = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert!(extract.contains("extract"));

    let salvage = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert!(salvage.contains("salvage"));
}

#[test]
fn section_layout_matches_goldens() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    fs::write(input_dir.join("b.txt"), b"beta").expect("write file");
    let archive = tmp.path().join("sample.crushr");
    let bad_archive = tmp.path().join("bad.crushr");
    fs::write(&bad_archive, b"bad").expect("bad archive");

    let pack_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );
    let verify_ok_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&archive),
    );
    let verify_bad = run_any(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("verify")
            .arg(&bad_archive),
    );
    assert!(!verify_bad.status.success());
    let verify_bad_out = String::from_utf8(verify_bad.stdout).expect("stdout utf8");
    let info_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .args(["info", archive.to_str().expect("utf8")]),
    );
    let salvage_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .args(["salvage", archive.to_str().expect("utf8")]),
    );

    let base = Path::new("tests/golden");
    let expected_pack = fs::read_to_string(base.join("pack.txt")).expect("golden pack");
    let expected_verify_ok =
        fs::read_to_string(base.join("verify_success.txt")).expect("golden verify success");
    let expected_verify_failure =
        fs::read_to_string(base.join("verify_failure.txt")).expect("golden verify failure");
    let expected_info = fs::read_to_string(base.join("info_human.txt")).expect("golden info");
    let expected_salvage = fs::read_to_string(base.join("salvage.txt")).expect("golden salvage");

    assert_eq!(normalize_paths(pack_out, tmp.path()), expected_pack);
    assert_eq!(
        normalize_paths(verify_ok_out, tmp.path()),
        expected_verify_ok
    );
    assert_eq!(
        normalize_paths(verify_bad_out, tmp.path()),
        expected_verify_failure
    );
    assert_eq!(normalize_info_output(info_out, tmp.path()), expected_info);
    assert_eq!(normalize_paths(salvage_out, tmp.path()), expected_salvage);
}

#[test]
fn non_tty_output_has_no_motion_control_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    fs::write(input_dir.join("b.txt"), b"beta").expect("write file");
    let archive = tmp.path().join("sample.crushr");
    let extract_out = tmp.path().join("extract");
    let recover_out = tmp.path().join("recover");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    for output in [
        run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .env("CRUSHR_MOTION", "full")
                .arg("pack")
                .arg(&input_dir)
                .arg("-o")
                .arg(tmp.path().join("second.crushr")),
        ),
        run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .env("CRUSHR_MOTION", "full")
                .arg("verify")
                .arg(&archive),
        ),
        run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .env("CRUSHR_MOTION", "full")
                .arg("extract")
                .arg(&archive)
                .arg("-o")
                .arg(&extract_out),
        ),
        run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .env("CRUSHR_MOTION", "full")
                .arg("extract")
                .arg(&archive)
                .arg("-o")
                .arg(&recover_out)
                .arg("--recover"),
        ),
    ] {
        assert!(
            !output.contains('\r'),
            "non-tty output contained carriage return: {output:?}"
        );
        assert!(
            !output.contains("\u{1b}[2K"),
            "non-tty output contained clear-line ANSI: {output:?}"
        );
    }
}

#[test]
fn info_list_tree_and_flat_are_deterministic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(input_dir.join("docs/nested")).expect("create dirs");
    fs::write(input_dir.join("alpha.txt"), b"alpha").expect("write file");
    fs::write(input_dir.join("docs/readme.md"), b"readme").expect("write file");
    fs::write(input_dir.join("docs/nested/deep.txt"), b"deep").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let tree_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .arg("--list"),
    );
    assert!(tree_out.contains("crushr  /  list"));
    assert!(tree_out.contains("├── docs/"));
    assert!(tree_out.contains("│   ├── nested/"));
    assert!(tree_out.contains("│   │   └── deep.txt"));
    assert!(tree_out.contains("│   └── readme.md"));
    assert!(tree_out.contains("└── alpha.txt"));

    let flat_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .arg("--list")
            .arg("--flat"),
    );
    assert!(flat_out.contains("  docs/\n"));
    assert!(flat_out.contains("  docs/nested/\n"));
    assert!(flat_out.contains("  alpha.txt\n"));
    assert!(flat_out.contains("  docs/nested/deep.txt\n"));
    assert!(flat_out.contains("  docs/readme.md\n"));
}

#[test]
fn info_list_degrades_honestly_when_listing_proof_is_unavailable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(input_dir.join("keep")).expect("create dirs");
    fs::write(input_dir.join("keep/provable.txt"), b"provable").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let mut archive_bytes = fs::read(&archive).expect("read archive");
    archive_bytes.truncate(archive_bytes.len().saturating_sub(8));
    fs::write(&archive, archive_bytes).expect("rewrite archive");

    let out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .arg("--list"),
    );

    assert!(out.contains("WARNING:"));
    assert!(out.contains("IDX3 could not be proven"));
    assert!(out.contains("crushr extract --recover"));
    assert!(out.contains("(no provable paths)"));
    assert!(out.contains("status                 DEGRADED"));
}

#[test]
fn info_list_surfaces_profile_context_across_preservation_variants() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"a").expect("write");

    for (profile, expected_profile) in [
        ("full", "profile                full"),
        ("basic", "profile                basic"),
        ("payload-only", "profile                payload-only"),
    ] {
        let archive = tmp.path().join(format!("{profile}.crushr"));
        run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .arg("pack")
                .arg(&input_dir)
                .arg("-o")
                .arg(&archive)
                .arg("--preservation")
                .arg(profile),
        );

        let out = run_ok(
            Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
                .arg("info")
                .arg(&archive)
                .arg("--list"),
        );
        assert!(
            out.contains(expected_profile),
            "{profile} profile missing\n{out}"
        );
        assert!(out.contains("scope                  regular files (metadata/index proven)"));
    }
}

#[test]
fn info_entry_reports_truth_surface_for_exact_path_and_not_found() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(input_dir.join("src")).expect("create dirs");
    fs::write(input_dir.join("src/main.rs"), b"fn main(){}\n").expect("write file");
    fs::write(input_dir.join("src/lib.rs"), b"pub fn x(){}\n").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--entry", "src/main.rs"]),
    );
    assert!(out.contains("crushr  /  entry"));
    assert!(out.contains("logical path"));
    assert!(out.contains("src/main.rs"));
    assert!(out.contains("trust class"));
    assert!(out.contains("canonical"));
    assert!(out.contains("payload verified"));
    assert!(out.contains("true"));
    assert!(out.contains("payload blake3"));
    assert!(out.contains("logical range"));
    assert!(out.contains("identity source"));
    assert!(out.contains("canonical_index"));
    assert!(out.contains("strict extraction supported"));

    let not_found = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--entry", "missing.txt"]),
    );
    assert!(not_found.contains("entry not found"));
}

#[test]
fn info_entry_and_find_json_are_deterministic_and_find_is_sorted() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(input_dir.join("src/deep")).expect("create dirs");
    fs::write(input_dir.join("src/main.rs"), b"main").expect("write file");
    fs::write(input_dir.join("src/lib.rs"), b"lib").expect("write file");
    fs::write(input_dir.join("src/deep/mod.rs"), b"mod").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let entry_json = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--entry", "src/main.rs", "--json"]),
    );
    let entry_value: serde_json::Value = serde_json::from_str(&entry_json).expect("json");
    assert_eq!(entry_value["path"], "src/main.rs");
    assert_eq!(entry_value["trust_class"], "canonical");
    assert!(entry_value["payload_verified"].is_boolean());
    assert!(entry_value["metadata_complete"].is_boolean());
    assert!(entry_value["extent_count"].is_u64());
    assert!(entry_value["size_bytes"].is_u64());
    assert!(entry_value["payload_blake3"].is_string());
    assert!(entry_value["logical_range"]["start"].is_u64());
    assert!(entry_value["logical_range"]["end"].is_u64());
    assert_eq!(entry_value["identity_source"], "canonical_index");
    assert!(entry_value["strict_extraction_supported"].is_boolean());

    let not_found_json = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--entry", "src/nope.rs", "--json"]),
    );
    let not_found_value: serde_json::Value =
        serde_json::from_str(&not_found_json).expect("json not found");
    assert_eq!(not_found_value["found"], false);
    assert_eq!(not_found_value["path"], "src/nope.rs");

    let find_json = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--find", ".rs", "--json"]),
    );
    let find_value: serde_json::Value = serde_json::from_str(&find_json).expect("find json");
    let rows = find_value.as_array().expect("array rows");
    let paths: Vec<&str> = rows
        .iter()
        .map(|row| row["path"].as_str().expect("path"))
        .collect();
    assert_eq!(paths, vec!["src/deep/mod.rs", "src/lib.rs", "src/main.rs"]);
    for row in rows {
        assert_eq!(row["trust_class"], "canonical");
    }

    let no_match_json = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--find", "zzz", "--json"]),
    );
    let no_match_value: serde_json::Value = serde_json::from_str(&no_match_json).expect("json");
    assert_eq!(no_match_value.as_array().expect("empty array").len(), 0);
}

#[test]
fn info_find_human_and_info_entry_do_not_create_output_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    let probe_dir = tmp.path().join("probe");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"a").expect("write");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .current_dir(tmp.path())
            .arg("info")
            .arg(&archive)
            .args(["--find", "a"]),
    );
    assert!(out.contains("crushr  /  find"));
    assert!(out.contains("a.txt"));
    assert!(!probe_dir.exists());

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .current_dir(tmp.path())
            .arg("info")
            .arg(&archive)
            .args(["--entry", "a.txt"]),
    );
    assert!(!probe_dir.exists());
}

#[test]
fn info_propagation_human_is_default_and_has_operator_sections() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write");
    fs::write(input_dir.join("b.txt"), b"beta").expect("write");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let mut bytes = fs::read(&archive).expect("read archive");
    let first_payload_byte = bytes
        .iter()
        .position(|byte| *byte == b'a')
        .expect("find payload byte");
    bytes[first_payload_byte] ^= 1;
    fs::write(&archive, bytes).expect("rewrite archive");

    let human = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .arg("--propagation"),
    );
    assert!(human.contains("crushr  /  propagation"));
    assert!(human.contains("Archive"));
    assert!(human.contains("Detected corruption"));
    assert!(human.contains("Impact summary"));
    assert!(human.contains("Entry impacts"));
    assert!(human.contains("canonical extraction blocked"));
    assert!(human.contains("supported trust classes"));
    assert!(human.contains("unrecoverable"));
    assert!(!human.contains("\"report_version\""));

    let human_second = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .arg("--propagation"),
    );
    assert_eq!(human, human_second);

    let json = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--propagation", "--json"]),
    );
    let json_value: serde_json::Value = serde_json::from_str(&json).expect("json");
    assert_eq!(json_value["report_kind"], "corruption_propagation_graph");
    assert!(json_value["entry_impacts"].is_array());
}

#[test]
fn info_report_propagation_surface_is_retired() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"a").expect("write");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("pack")
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let out = run_any(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr")))
            .arg("info")
            .arg(&archive)
            .args(["--report", "propagation"]),
    );
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("stderr utf8");
    assert!(stderr.contains("--report is retired; use --propagation"));
}
