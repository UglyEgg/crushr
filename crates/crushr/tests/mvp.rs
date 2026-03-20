// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fs;
use tempfile::TempDir;

fn run(cmd: &mut std::process::Command) {
    let out = cmd.output().expect("run");
    if !out.status.success() {
        panic!(
            "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn root_extract_roundtrip_from_canonical_archive() {
    let td = TempDir::new().unwrap();
    let in_dir = td.path().join("in");
    let out_dir = td.path().join("out");
    fs::create_dir_all(in_dir.join("sub")).unwrap();

    fs::write(in_dir.join("a.txt"), b"hello\n").unwrap();
    fs::write(in_dir.join("sub/b.json"), br#"{"k":"v","n":1}"#).unwrap();

    let archive = td.path().join("test.crushr");
    let pack_bin = std::path::Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let root_bin = std::path::Path::new(env!("CARGO_BIN_EXE_crushr"));

    run(std::process::Command::new(pack_bin).args([
        in_dir.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--level",
        "3",
    ]));

    fs::create_dir_all(&out_dir).unwrap();
    run(std::process::Command::new(root_bin).args([
        "extract",
        archive.to_str().unwrap(),
        "--all",
        "-o",
        out_dir.to_str().unwrap(),
    ]));

    assert_eq!(fs::read(out_dir.join("a.txt")).unwrap(), b"hello\n");
    assert_eq!(
        fs::read(out_dir.join("sub/b.json")).unwrap(),
        br#"{"k":"v","n":1}"#
    );
}

#[test]
fn canonical_crushr_extract_roundtrip_via_crushr_pack_archive() {
    let td = TempDir::new().unwrap();
    let in_dir = td.path().join("canonical-in");
    let out_dir = td.path().join("canonical-out");
    fs::create_dir_all(in_dir.join("sub")).unwrap();

    fs::write(in_dir.join("a.txt"), b"hello\n").unwrap();
    fs::write(in_dir.join("sub/b.json"), br#"{"k":"v","n":1}"#).unwrap();

    let archive = td.path().join("canonical-test.crushr");
    let pack_bin = std::path::Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let extract_bin = std::path::Path::new(env!("CARGO_BIN_EXE_crushr-extract"));

    run(std::process::Command::new(pack_bin).args([
        in_dir.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--level",
        "3",
    ]));

    fs::create_dir_all(&out_dir).unwrap();
    run(std::process::Command::new(extract_bin).args([
        archive.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ]));

    assert_eq!(fs::read(out_dir.join("a.txt")).unwrap(), b"hello\n");
    assert_eq!(
        fs::read(out_dir.join("sub/b.json")).unwrap(),
        br#"{"k":"v","n":1}"#
    );
}
