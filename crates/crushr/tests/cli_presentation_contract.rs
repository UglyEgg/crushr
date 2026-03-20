// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fs;
use std::path::Path;
use std::process::Command;

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
