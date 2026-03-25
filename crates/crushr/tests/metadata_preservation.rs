// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

#![cfg(unix)]

use crushr::index_codec::{decode_index, encode_index};
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
};
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use std::ffi::OsStr;
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

struct FileReader {
    file: fs::File,
}

impl ReadAt for FileReader {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> anyhow::Result<usize> {
        use std::os::unix::fs::FileExt;
        Ok(self.file.read_at(buf, offset)?)
    }
}

impl Len for FileReader {
    fn len(&self) -> anyhow::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

fn mutate_index_in_place<F>(archive: &Path, mutate: F)
where
    F: FnOnce(&mut crushr::format::Index),
{
    let reader = FileReader {
        file: fs::File::open(archive).unwrap(),
    };
    let open = open_archive_v1(&reader).unwrap();
    let mut index = decode_index(&open.tail.idx3_bytes).unwrap();
    mutate(&mut index);
    let new_index = encode_index(&index);

    let mut bytes = fs::read(archive).unwrap();
    let idx_off = open.tail.footer.index_offset as usize;
    let idx_end = idx_off + new_index.len();
    bytes[idx_off..idx_end].copy_from_slice(&new_index);

    let footer_off = bytes.len() - FTR4_LEN;
    let mut footer = Ftr4::read_from(&bytes[footer_off..]).unwrap();
    footer.index_hash = *blake3::hash(&new_index).as_bytes();
    let footer = footer.finalize().unwrap();
    let mut footer_bytes = Vec::with_capacity(FTR4_LEN);
    footer.write_to(&mut footer_bytes).unwrap();
    bytes[footer_off..].copy_from_slice(&footer_bytes);
    fs::write(archive, bytes).unwrap();
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
    fs::hard_link(&file_path, input.join("nested/data-hardlink.txt")).unwrap();

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
    assert_eq!(
        extracted_meta.uid(),
        fs::metadata(&file_path).unwrap().uid()
    );
    assert_eq!(
        extracted_meta.gid(),
        fs::metadata(&file_path).unwrap().gid()
    );

    let restored_xattr = xattr::get(&extracted_file, xattr_name)
        .unwrap()
        .expect("xattr should be restored");
    assert_eq!(restored_xattr, xattr_value);

    let extracted_hardlink = output.join("nested/data-hardlink.txt");
    let hardlink_meta = fs::metadata(&extracted_hardlink).unwrap();
    assert_eq!(extracted_meta.ino(), hardlink_meta.ino());
    assert_eq!(extracted_meta.dev(), hardlink_meta.dev());
    assert_eq!(extracted_meta.nlink(), 2);

    let extracted_link = output.join("ln-data");
    let link_meta = fs::symlink_metadata(&extracted_link).unwrap();
    assert!(link_meta.file_type().is_symlink());
    assert_eq!(
        fs::read_link(&extracted_link).unwrap().to_string_lossy(),
        "nested/data.txt"
    );

    let empty_dir = output.join("nested/empty");
    assert!(empty_dir.is_dir());

    let info = Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-info")))
        .arg(archive)
        .output()
        .expect("run info");
    assert!(info.status.success());
    let info_out = String::from_utf8_lossy(&info.stdout);
    assert!(info_out.contains("Metadata"));
    assert!(info_out.contains("ownership"));
    assert!(info_out.contains("hard links"));
}

#[test]
fn extraction_warns_when_ownership_restore_is_not_permitted() {
    if nix_like_euid_is_root() {
        return;
    }

    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    let file_path = input.join("file.txt");
    fs::write(&file_path, b"payload").unwrap();

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
    mutate_index_in_place(&archive, |index| {
        for entry in &mut index.entries {
            entry.uid = 0;
            entry.gid = 0;
        }
    });

    let output = td.path().join("output");
    let out = Command::new(Path::new(env!("CARGO_BIN_EXE_crushr-extract")))
        .args([archive.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .output()
        .expect("run extract");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("WARNING[ownership-restore]"));
}

fn nix_like_euid_is_root() -> bool {
    let mut status = false;
    let output = Command::new(OsStr::new("id"))
        .arg("-u")
        .output()
        .expect("run id -u");
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        status = text.trim() == "0";
    }
    status
}
