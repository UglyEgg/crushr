use crushr::index_codec::decode_index;
use crushr_format::ftr4::{Ftr4, FTR4_LEN};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

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

fn build_crushr_pack_bin() -> PathBuf {
    let root = workspace_root();
    run(Command::new("cargo")
        .args(["build", "-p", "crushr", "--bin", "crushr-pack"])
        .current_dir(&root));
    root.join("target/debug/crushr-pack")
}

fn read_index_entries(archive: &Path) -> Vec<crushr::format::Entry> {
    let bytes = fs::read(archive).expect("read archive bytes");
    assert!(bytes.len() >= FTR4_LEN, "archive too small for FTR4");

    let footer = Ftr4::read_from(&bytes[bytes.len() - FTR4_LEN..]).expect("parse footer");
    let start = footer.index_offset as usize;
    let end = (footer.index_offset + footer.index_len) as usize;
    let idx = decode_index(&bytes[start..end]).expect("decode index");
    idx.entries
}

#[test]
fn crushr_pack_repeated_runs_are_byte_identical() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(input.join("z-dir")).unwrap();
    fs::create_dir_all(input.join("a-dir")).unwrap();
    fs::write(input.join("z-dir/zzz.txt"), b"z").unwrap();
    fs::write(input.join("a-dir/aaa.txt"), b"a").unwrap();
    fs::write(input.join("root.txt"), b"root").unwrap();

    let bin = build_crushr_pack_bin();
    let one = td.path().join("one.crs");
    let two = td.path().join("two.crs");

    run(Command::new(&bin).args([
        input.to_str().unwrap(),
        "-o",
        one.to_str().unwrap(),
        "--level",
        "3",
    ]));
    run(Command::new(&bin).args([
        input.to_str().unwrap(),
        "-o",
        two.to_str().unwrap(),
        "--level",
        "3",
    ]));

    assert_eq!(fs::read(&one).unwrap(), fs::read(&two).unwrap());
}

#[test]
fn crushr_pack_index_order_and_metadata_are_normalized() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(input.join("b/sub")).unwrap();
    fs::create_dir_all(input.join("a")).unwrap();
    fs::write(input.join("b/sub/three.txt"), b"3").unwrap();
    fs::write(input.join("a/one.txt"), b"1").unwrap();
    fs::write(input.join("a/two.txt"), b"2").unwrap();

    let archive = td.path().join("out.crs");
    let bin = build_crushr_pack_bin();
    run(Command::new(&bin).args([
        input.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--level",
        "3",
    ]));

    let entries = read_index_entries(&archive);
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, vec!["a/one.txt", "a/two.txt", "b/sub/three.txt"]);

    for entry in entries {
        assert_eq!(entry.mode, 0, "mode should be normalized to 0");
        assert_eq!(entry.mtime, 0, "mtime should be normalized to 0");
        assert!(entry.xattrs.is_empty(), "xattrs should be empty");
    }
}
