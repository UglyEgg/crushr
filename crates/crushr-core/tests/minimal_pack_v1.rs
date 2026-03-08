use anyhow::Result;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
};
use crushr_format::ftr4::FTR4_LEN;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

static BUILD_ONCE: Once = Once::new();

struct FileReader {
    file: fs::File,
}

impl ReadAt for FileReader {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        use std::os::unix::fs::FileExt;
        Ok(self.file.read_at(buf, offset)?)
    }
}

impl Len for FileReader {
    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn unique_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

fn ensure_bins_built() {
    BUILD_ONCE.call_once(|| {
        let out = Command::new("cargo")
            .current_dir(workspace_root())
            .args(["build", "-q", "-p", "crushr", "--bins"])
            .output()
            .expect("run cargo build");
        assert!(
            out.status.success(),
            "cargo build failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    });
}

fn run_bin(bin: &str, args: &[&str]) -> std::process::Output {
    let bin_path = workspace_root().join(format!("target/debug/{bin}"));
    Command::new(bin_path).args(args).output().unwrap()
}

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn pack_single_file_produces_readable_v1_archive() {
    ensure_bins_built();

    let root = unique_dir("crushr-pack-v1-single");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("hello.txt");
    fs::write(&input, b"hello minimal v1\n").unwrap();
    let archive = root.join("single.crs");

    let out = run_bin(
        "crushr-pack",
        &[
            input.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ],
    );
    assert_ok(&out);

    let reader = FileReader {
        file: fs::File::open(&archive).unwrap(),
    };
    let opened = open_archive_v1(&reader).unwrap();
    assert!(opened.tail.idx3_bytes.starts_with(b"IDX3"));

    let info = run_bin("crushr-info", &[archive.to_str().unwrap(), "--json"]);
    assert_ok(&info);
    let info_json: serde_json::Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(info_json["tool"], "crushr-info");

    let fsck = run_bin("crushr-fsck", &[archive.to_str().unwrap(), "--json"]);
    assert_ok(&fsck);
    let fsck_json: serde_json::Value = serde_json::from_slice(&fsck.stdout).unwrap();
    assert_eq!(fsck_json["tool"], "crushr-fsck");
    assert_eq!(fsck_json["payload"]["verify"]["status"], "ok");

    let bytes = fs::read(&archive).unwrap();
    let footer_start = bytes.len() - FTR4_LEN;
    assert_eq!(&bytes[footer_start..footer_start + 4], b"FTR4");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn pack_directory_and_output_is_deterministic() {
    ensure_bins_built();

    let root = unique_dir("crushr-pack-v1-dir");
    let src = root.join("src");
    fs::create_dir_all(src.join("nested")).unwrap();

    fs::write(src.join("a.txt"), b"alpha\n").unwrap();
    fs::write(src.join("nested/b.txt"), b"bravo\n").unwrap();

    let archive_a = root.join("tree-a.crs");
    let archive_b = root.join("tree-b.crs");

    let out_a = run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive_a.to_str().unwrap()],
    );
    assert_ok(&out_a);

    let out_b = run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive_b.to_str().unwrap()],
    );
    assert_ok(&out_b);

    let bytes_a = fs::read(&archive_a).unwrap();
    let bytes_b = fs::read(&archive_b).unwrap();
    assert_eq!(bytes_a, bytes_b);

    let info = run_bin("crushr-info", &[archive_a.to_str().unwrap(), "--json"]);
    assert_ok(&info);

    let fsck = run_bin("crushr-fsck", &[archive_a.to_str().unwrap(), "--json"]);
    assert_ok(&fsck);

    let _ = fs::remove_dir_all(&root);
}
