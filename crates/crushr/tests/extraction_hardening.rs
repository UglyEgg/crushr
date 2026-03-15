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
        mode: 0,
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

fn symlink_entry(path: &str, target: &str) -> Entry {
    Entry {
        path: path.to_string(),
        kind: EntryKind::Symlink,
        mode: 0,
        mtime: 0,
        size: target.len() as u64,
        extents: vec![],
        link_target: Some(target.to_string()),
        xattrs: vec![],
    }
}

fn run_ok(cmd: &mut std::process::Command) {
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
fn legacy_extract_rejects_parent_traversal_and_stays_confined() {
    let td = TempDir::new().unwrap();
    let archive = td.path().join("legacy-bad.crs");
    build_archive(&archive, regular_entry("../nested/escape.txt", 5), b"hello");

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

    assert!(!out.status.success());
    assert!(!td.path().join("nested/escape.txt").exists());
}

#[test]
fn legacy_extract_rejects_symlink_entries_in_hardened_mode() {
    let td = TempDir::new().unwrap();
    let archive = td.path().join("legacy-symlink.crs");
    build_archive(&archive, symlink_entry("link", "target"), b"");

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

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("symlink extraction is disabled in hardened mode"));
}

#[test]
fn legacy_extract_accepts_safe_relative_path() {
    let td = TempDir::new().unwrap();
    let archive = td.path().join("legacy-safe.crs");
    build_archive(&archive, regular_entry("safe/dir/file.txt", 5), b"hello");

    let out_dir = td.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();

    run_ok(
        std::process::Command::new(env!("CARGO_BIN_EXE_crushr")).args([
            "extract",
            archive.to_str().unwrap(),
            "--all",
            "-o",
            out_dir.to_str().unwrap(),
        ]),
    );

    let extracted = out_dir.join("safe/dir/file.txt");
    assert!(extracted.starts_with(&out_dir));
    assert_eq!(fs::read(extracted).unwrap(), b"hello");
}
