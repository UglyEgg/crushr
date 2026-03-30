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
        "pack",
        "extract",
        "verify",
        "info",
        "about",
        "completion",
        "salvage",
        "lab",
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
fn completion_command_exists_and_emits_non_empty_output() {
    for shell in ["bash", "zsh", "fish"] {
        let out =
            run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args(["completion", shell]));
        assert!(
            out.status.success(),
            "completion command failed for {shell}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).expect("stdout utf8");
        assert!(
            !text.trim().is_empty(),
            "completion output should not be empty for {shell}"
        );
    }
}

#[test]
fn completion_includes_info_and_primary_command_surface() {
    let bash =
        run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args(["completion", "bash"]));

    for token in ["--list", "--entry", "--find", "--propagation"] {
        assert!(
            bash.contains(token),
            "bash completion missing info flag token: {token}"
        );
    }
    for token in ["extract", "verify", "pack", "about"] {
        assert!(
            bash.contains(token),
            "bash completion missing primary command token: {token}"
        );
    }
}

#[test]
fn man_command_exists_and_generates_expected_pages() {
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("man");
    let out = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "man",
        "--out-dir",
        out_dir.to_str().expect("utf8"),
    ]));
    assert!(
        out.status.success(),
        "man generation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    for page in [
        "crushr.1",
        "crushr-info.1",
        "crushr-extract.1",
        "crushr-verify.1",
        "crushr-pack.1",
        "crushr-about.1",
        "crushr-completion.1",
    ] {
        let path = out_dir.join(page);
        assert!(
            path.exists(),
            "missing generated man page: {}",
            path.display()
        );
        let bytes = std::fs::read(&path).expect("read page");
        assert!(!bytes.is_empty(), "generated man page is empty: {}", page);
    }
}

#[test]
fn man_generation_is_deterministic_across_runs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("man");
    let run_once = || {
        run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
            "man",
            "--out-dir",
            out_dir.to_str().expect("utf8"),
        ]))
    };

    let first = run_once();
    assert!(
        first.status.success(),
        "first man generation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let first_page = std::fs::read(out_dir.join("crushr.1")).expect("read first page");

    let second = run_once();
    assert!(
        second.status.success(),
        "second man generation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let second_page = std::fs::read(out_dir.join("crushr.1")).expect("read second page");

    assert_eq!(
        first_page, second_page,
        "man generation must be deterministic across runs"
    );
}

#[test]
fn pack_defaults_to_crs_extension_when_output_has_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&input_dir).expect("create input");
    std::fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive_without_ext = tmp.path().join("sample");
    let expected_archive = tmp.path().join("sample.crs");

    let out = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "pack",
        input_dir.to_str().expect("utf8"),
        "-o",
        archive_without_ext.to_str().expect("utf8"),
    ]));
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
fn root_cli_help_version_and_about_are_consistent() {
    let help = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--help"));
    assert!(help.contains("crushr <command> [args...]"));

    let version = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("--version"));
    assert!(!version.trim().is_empty());

    let about = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).arg("about"));
    assert!(about.contains("crushr"));
}

#[test]
fn undocumented_wrapper_argument_aliases_are_rejected() {
    for wrapper in [
        env!("CARGO_BIN_EXE_crushr"),
        env!("CARGO_BIN_EXE_crushr"),
        env!("CARGO_BIN_EXE_crushr"),
        env!("CARGO_BIN_EXE_crushr"),
    ] {
        let out = run(Command::new(Path::new(wrapper)).args(["placeholder", "--help"]));
        assert!(
            !out.status.success(),
            "unexpected help alias acceptance: {wrapper}"
        );
        let stdout = String::from_utf8(out.stdout).expect("stdout utf8");
        assert!(
            !stdout.contains("canonical equivalent:"),
            "unexpected wrapper help behavior leaked: {wrapper}"
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

    let pack = run(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "pack",
        input_dir.to_str().expect("utf8"),
        "-o",
        archive.to_str().expect("utf8"),
    ]));
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

#[test]
fn pack_profile_flag_emits_phase_breakdown_only_when_explicitly_requested() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&input_dir).expect("create input");
    std::fs::write(input_dir.join("a.txt"), b"alpha").expect("write file");
    let archive = tmp.path().join("sample.crs");

    let baseline = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "pack",
        input_dir.to_str().expect("utf8"),
        "-o",
        archive.to_str().expect("utf8"),
        "--silent",
    ]));
    assert!(
        !baseline.contains("Pack phases"),
        "pack phase breakdown must not print by default"
    );

    let archive_profiled = tmp.path().join("sample_profiled.crs");
    let profiled = run_ok(Command::new(Path::new(env!("CARGO_BIN_EXE_crushr"))).args([
        "pack",
        input_dir.to_str().expect("utf8"),
        "-o",
        archive_profiled.to_str().expect("utf8"),
        "--silent",
        "--profile-pack",
    ]));
    assert!(profiled.contains("Pack phases"));
    for phase in [
        "discovery",
        "metadata",
        "hashing",
        "compression",
        "emission",
        "finalization",
    ] {
        assert!(
            profiled.contains(phase),
            "missing profile phase '{phase}' in output: {profiled}"
        );
    }
}
