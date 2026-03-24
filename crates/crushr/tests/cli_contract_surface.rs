// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::path::Path;
use std::process::{Command, Output};

fn run(cmd: &mut Command) -> Output {
    cmd.output().expect("run command")
}

fn run_ok(cmd: &mut Command) -> String {
    let out = run(cmd);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("stdout utf8")
}

#[test]
fn canonical_command_surface_is_locked() {
    let help = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert!(
        !help.contains('\u{1b}'),
        "help output should not emit ANSI escape codes in non-tty mode"
    );

    for command in [
        "pack", "extract", "verify", "info", "about", "salvage", "lab",
    ] {
        assert!(help.contains(command), "missing command {command}\n{help}");
    }

    for legacy in [
        "append",
        "list",
        "cat",
        "dict-train",
        "tune",
        "completions",
        "fsck",
    ] {
        assert!(
            !help.contains(&format!("\n  {legacy}")),
            "legacy command leaked in help: {legacy}\n{help}"
        );

        let out = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg(legacy));
        assert_eq!(out.status.code(), Some(2));
        let stderr = String::from_utf8(out.stderr).expect("stderr utf8");
        assert!(
            stderr.contains("unknown command"),
            "expected unknown command for {legacy}"
        );
    }
}

#[test]
fn pack_defaults_to_crs_extension_when_output_has_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&input_dir).expect("create input");
    std::fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive_without_ext = tmp.path().join("sample");
    let expected_archive = tmp.path().join("sample.crs");

    let out = run(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack"))).args([
            input_dir.to_str().expect("utf8"),
            "-o",
            archive_without_ext.to_str().expect("utf8"),
        ]),
    );
    assert!(out.status.success(), "pack failed");
    assert!(
        expected_archive.exists(),
        "expected .crs archive to be created"
    );
    assert!(
        !archive_without_ext.exists(),
        "output path without extension should not be used directly"
    );
}

#[test]
fn wrappers_expose_consistent_help_version_and_about() {
    let version = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--version"));

    for (wrapper, canonical_usage) in [
        (
            env!("CARGO_BIN_EXE_crushr-pack"),
            "canonical equivalent: crushr pack",
        ),
        (
            env!("CARGO_BIN_EXE_crushr-extract"),
            "canonical equivalent: crushr extract",
        ),
        (
            env!("CARGO_BIN_EXE_crushr-info"),
            "canonical equivalent: crushr info",
        ),
        (
            env!("CARGO_BIN_EXE_crushr-salvage"),
            "canonical equivalent: crushr salvage",
        ),
    ] {
        let help = run_ok(Command::new(Path::new(wrapper)).arg("--help"));
        assert!(help.contains("wrapper over canonical crushr CLI"));
        assert!(help.contains(canonical_usage));

        let wrapper_version = run_ok(Command::new(Path::new(wrapper)).arg("--version"));
        assert_eq!(wrapper_version, version, "version mismatch for {wrapper}");

        let wrapper_about = run_ok(Command::new(Path::new(wrapper)).arg("about"));
        let root_about = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("about"));
        assert_eq!(wrapper_about, root_about, "about mismatch for {wrapper}");
    }
}

#[test]
fn undocumented_wrapper_argument_aliases_are_rejected() {
    for wrapper in [
        env!("CARGO_BIN_EXE_crushr-pack"),
        env!("CARGO_BIN_EXE_crushr-extract"),
        env!("CARGO_BIN_EXE_crushr-info"),
        env!("CARGO_BIN_EXE_crushr-salvage"),
    ] {
        let out = run(Command::new(Path::new(wrapper)).args(["placeholder", "--help"]));
        assert!(
            !out.status.success(),
            "wrapper accepted hidden help alias position: {wrapper}"
        );
        let stdout = String::from_utf8(out.stdout).expect("stdout utf8");
        assert!(
            !stdout.contains("wrapper over canonical crushr CLI"),
            "wrapper leaked hidden help alias behavior: {wrapper}"
        );
    }
}

#[test]
fn exit_code_handling_is_consistent_for_root_cli() {
    let no_args = run(&mut Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))));
    assert_eq!(no_args.status.code(), Some(1));

    let help = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert_eq!(help.status.code(), Some(0));

    let version = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--version"));
    assert_eq!(version.status.code(), Some(0));

    let about_bad = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args(["about", "x"]));
    assert_eq!(about_bad.status.code(), Some(2));

    let verify_missing_archive =
        run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("verify"));
    assert_eq!(verify_missing_archive.status.code(), Some(2));
}

#[test]
fn shared_flags_json_and_silent_are_consistent_when_combined() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&input_dir).expect("create input");
    std::fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive = tmp.path().join("sample.crushr");

    let pack = run(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack"))).args([
            input_dir.to_str().expect("utf8"),
            "-o",
            archive.to_str().expect("utf8"),
        ]),
    );
    assert!(pack.status.success(), "pack failed");

    let verify_json_silent = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "verify",
        archive.to_str().expect("utf8"),
        "--json",
        "--silent",
    ]));
    assert!(verify_json_silent.trim_start().starts_with('{'));
    assert!(!verify_json_silent.contains("status=VERIFIED"));

    let salvage_json_silent = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "salvage",
        archive.to_str().expect("utf8"),
        "--json",
        "--silent",
    ]));
    assert!(salvage_json_silent.trim_start().starts_with('{'));
    assert!(!salvage_json_silent.contains("status=PARTIAL"));
}
