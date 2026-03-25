// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("run command");
    if !out.status.success() {
        panic!(
            "command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn set_mtime(path: &Path, secs: i64) {
    let out = Command::new("touch")
        .args(["-m", "-d", &format!("@{secs}"), path.to_str().unwrap()])
        .output()
        .expect("run touch");
    assert!(
        out.status.success(),
        "touch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn mixed_tree_roundtrip_preserves_baseline_metadata_and_xattrs() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    let output = td.path().join("output");
    fs::create_dir_all(input.join("nested/empty")).unwrap();

    let file_path = input.join("nested/data.txt");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, b"payload").unwrap();

    symlink("nested/data.txt", input.join("ln-data")).unwrap();

    let mut file_perm = fs::metadata(&file_path).unwrap().permissions();
    file_perm.set_mode(0o640);
    fs::set_permissions(&file_path, file_perm).unwrap();

    let mtime = 1_701_234_567i64;
    set_mtime(&file_path, mtime);

    let xattr_name = "user.crushr.test";
    let xattr_value = b"xattr-preserved";
    if let Err(err) = xattr::set(&file_path, xattr_name, xattr_value) {
        let msg = err.to_string();
        if msg.contains("Operation not supported") || msg.contains("Not supported") {
            eprintln!("skipping xattr assertions on unsupported filesystem: {msg}");
            return;
        }
        panic!("unable to set test xattr: {msg}");
    }

    let archive = td.path().join("meta.crs");
    run(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-pack"))).args([
            input.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ]),
    );

    run(
        Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract"))).args([
            archive.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ]),
    );

    let extracted_file = output.join("nested/data.txt");
    let extracted_meta = fs::metadata(&extracted_file).unwrap();
    assert_eq!(extracted_meta.permissions().mode() & 0o777, 0o640);
    assert_eq!(extracted_meta.mtime(), mtime);

    let restored_xattr = xattr::get(&extracted_file, xattr_name)
        .unwrap()
        .expect("xattr should be restored");
    assert_eq!(restored_xattr, xattr_value);

    let extracted_link = output.join("ln-data");
    let link_meta = fs::symlink_metadata(&extracted_link).unwrap();
    assert!(link_meta.file_type().is_symlink());
    assert_eq!(
        fs::read_link(&extracted_link).unwrap().to_string_lossy(),
        "nested/data.txt"
    );

    let empty_dir = output.join("nested/empty");
    assert!(empty_dir.is_dir());
}
