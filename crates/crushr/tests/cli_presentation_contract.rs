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
    assert!(first.contains("[VERIFIED]"));
    assert!(first.contains("failure_domains"));
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
    assert!(salvage_human.contains("verified_files"));
    assert!(salvage_human.contains("partial_files"));
    assert!(salvage_human.contains("rejected_or_unresolved_files"));
}

#[test]
fn root_help_lists_canonical_suite_and_demotes_legacy_surface() {
    let out = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));

    for token in ["pack", "extract", "verify", "info", "salvage", "lab"] {
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
    assert!(stdout.contains("[REFUSED]"));
    assert!(stdout.contains("failure_domains"));
    assert!(stdout.contains("archive structure validation failed"));
    assert!(!stdout.contains("parse FTR4"));
    assert!(!stdout.contains("bad footer magic"));
    assert!(!stderr.contains("parse FTR4"));
    assert!(!stderr.contains("bad footer magic"));
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
