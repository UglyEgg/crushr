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

#[test]
fn verify_output_is_deterministic_and_uses_shared_status_words() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    fs::create_dir_all(&input_dir).expect("create input");
    fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack")))
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );

    let first = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
            .arg(&archive),
    );
    let second = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
            .arg(&archive),
    );

    assert_eq!(first, second);
    assert!(first.contains("crushr-extract  /  verify"));
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
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack")))
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(pack_out.lines().count(), 1);
    assert!(pack_out.contains("status=COMPLETE"));

    let extract_out_text = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg(&archive)
            .arg("-o")
            .arg(&extract_out)
            .arg("--silent"),
    );
    assert_eq!(extract_out_text.lines().count(), 1);
    assert!(extract_out_text.contains("status=COMPLETE"));

    let verify_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(verify_out.lines().count(), 1);
    assert!(verify_out.contains("status=VERIFIED"));

    let salvage_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-salvage")))
            .arg(&archive)
            .arg("--silent"),
    );
    assert_eq!(salvage_out.lines().count(), 1);
    assert!(salvage_out.contains("status=PARTIAL"));

    let salvage_human =
        run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-salvage"))).arg(&archive));
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
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
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

    assert!(out.contains("crushr / about"));
    assert!(out.contains("Build"));
    assert!(out.contains("Behavior"));
    assert!(out.contains("Data Model"));
    assert!(out.contains("Built with"));
    assert!(out.contains("Support"));
    assert!(out.contains("pack             deterministic archive creation"));
    assert!(out.contains("extract          strict extraction (verification-gated)"));
    assert!(out.contains("verify           structural and integrity validation"));
    assert!(out.contains("salvage          research-mode recovery planning (non-canonical)"));
    assert!(out.contains("crushr info <archive> --json"));
    assert!(out.contains("crushr extract --verify <archive>"));
}

#[test]
fn canonical_help_commands_are_available() {
    let pack = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack"))).arg("--help"));
    assert!(pack.contains("usage: crushr-pack"));

    let extract =
        run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract"))).arg("--help"));
    assert!(extract.contains("usage: crushr-extract"));

    let salvage =
        run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-salvage"))).arg("--help"));
    assert!(salvage.contains("usage: crushr-salvage"));
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
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack")))
            .arg(&input_dir)
            .arg("-o")
            .arg(&archive),
    );
    let verify_ok_out = run_ok(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
            .arg(&archive),
    );
    let verify_bad = run_any(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
            .arg("--verify")
            .arg(&bad_archive),
    );
    assert!(!verify_bad.status.success());
    let verify_bad_out = String::from_utf8(verify_bad.stdout).expect("stdout utf8");
    let info_out = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-info"))).arg(&archive));
    let salvage_out =
        run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-salvage"))).arg(&archive));

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
    assert_eq!(normalize_paths(info_out, tmp.path()), expected_info);
    assert_eq!(normalize_paths(salvage_out, tmp.path()), expected_salvage);
}
