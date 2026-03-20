// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crushr::format::{Entry, EntryKind, Extent, Index, BLK_MAGIC_V2, CODEC_ZSTD, FTR_MAGIC_V2};
use crushr::index_codec::encode_index;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn build_archive(archive: &Path, entry: Entry, payload: &[u8]) {
    let mut f = fs::File::create(archive).expect("create archive");

    let comp = zstd::encode_all(payload, 3).expect("compress payload");
    f.write_all(BLK_MAGIC_V2).expect("write BLK magic");
    f.write_all(&CODEC_ZSTD.to_le_bytes()).expect("write codec");
    f.write_all(&(payload.len() as u64).to_le_bytes())
        .expect("write raw len");
    f.write_all(&(comp.len() as u64).to_le_bytes())
        .expect("write comp len");
    f.write_all(&comp).expect("write block payload");

    let blocks_end_offset = 4 + 4 + 8 + 8 + comp.len() as u64;
    let index = Index {
        entries: vec![entry],
    };
    let idx_bytes = encode_index(&index);
    let index_offset = blocks_end_offset;
    let index_len = idx_bytes.len() as u64;
    f.write_all(&idx_bytes).expect("write index");

    let index_hash = blake3::hash(&idx_bytes);
    f.write_all(FTR_MAGIC_V2).expect("write footer magic");
    f.write_all(&blocks_end_offset.to_le_bytes())
        .expect("write blocks_end_offset");
    f.write_all(&index_offset.to_le_bytes())
        .expect("write index_offset");
    f.write_all(&index_len.to_le_bytes())
        .expect("write index_len");
    f.write_all(index_hash.as_bytes())
        .expect("write index hash");
}

fn regular_entry(path: &str, len: u64) -> Entry {
    Entry {
        path: path.to_string(),
        kind: EntryKind::Regular,
        mode: 0o644,
        mtime: 0,
        size: len,
        extents: vec![Extent {
            block_id: 0,
            offset: 0,
            len,
        }],
        link_target: None,
        xattrs: vec![],
    }
}

#[test]
fn canonical_extractor_rejects_parent_traversal() {
    let td = TempDir::new().unwrap();
    let archive = td.path().join("bad-parent.crs");
    build_archive(&archive, regular_entry("../../outside.txt", 5), b"hello");

    let out_dir = td.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_crushr-extract"))
        .args([archive.to_str().unwrap(), "-o", out_dir.to_str().unwrap()])
        .output()
        .expect("run crushr-extract");

    assert!(!out.status.success());
}

#[test]
fn canonical_extractor_rejects_absolute_path() {
    let td = TempDir::new().unwrap();
    let archive = td.path().join("bad-absolute.crs");
    build_archive(&archive, regular_entry("/tmp/pwned", 5), b"hello");

    let out_dir = td.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_crushr-extract"))
        .args([archive.to_str().unwrap(), "-o", out_dir.to_str().unwrap()])
        .output()
        .expect("run crushr-extract");

    assert!(!out.status.success());
}

#[test]
fn root_crushr_extract_delegates_to_strict_for_all_entries() {
    let td = TempDir::new().unwrap();
    let in_dir = td.path().join("in");
    fs::create_dir_all(in_dir.join("safe/dir")).unwrap();
    fs::write(in_dir.join("safe/dir/file.txt"), b"hello").unwrap();

    let archive = td.path().join("root-all.crushr");
    let pack = std::process::Command::new(env!("CARGO_BIN_EXE_crushr-pack"))
        .args([
            in_dir.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run crushr-pack");
    assert!(
        pack.status.success(),
        "{}",
        String::from_utf8_lossy(&pack.stderr)
    );

    let out_dir = td.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_crushr"))
        .args([
            "extract",
            archive.to_str().unwrap(),
            "--all",
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("run crushr extract");

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read(out_dir.join("safe/dir/file.txt")).unwrap(),
        b"hello"
    );
}

#[test]
fn root_crushr_extract_delegates_to_strict_for_path_filtered_mode() {
    let td = TempDir::new().unwrap();
    let in_dir = td.path().join("in");
    fs::create_dir_all(in_dir.join("safe/dir")).unwrap();
    fs::write(in_dir.join("safe/dir/file.txt"), b"hello").unwrap();

    let archive = td.path().join("root-filtered.crushr");
    let pack = std::process::Command::new(env!("CARGO_BIN_EXE_crushr-pack"))
        .args([
            in_dir.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run crushr-pack");
    assert!(
        pack.status.success(),
        "{}",
        String::from_utf8_lossy(&pack.stderr)
    );

    let out_dir = td.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_crushr"))
        .args([
            "extract",
            archive.to_str().unwrap(),
            "safe/dir/file.txt",
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("run crushr extract");

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read(out_dir.join("safe/dir/file.txt")).unwrap(),
        b"hello"
    );
}
