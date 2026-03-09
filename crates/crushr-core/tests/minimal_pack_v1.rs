use anyhow::Result;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::scan_blocks_v1,
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

fn assert_err_code_2(out: &std::process::Output) {
    assert_eq!(
        out.status.code(),
        Some(2),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn assert_err_code_3(out: &std::process::Output) {
    assert_eq!(
        out.status.code(),
        Some(3),
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

#[test]
fn fsck_reports_corrupted_block_for_payload_byte_flip() {
    ensure_bins_built();

    let root = unique_dir("crushr-pack-v1-corrupt-payload");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("hello.txt");
    fs::write(
        &input,
        b"hello minimal v1 payload corruption
",
    )
    .unwrap();
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
    let _opened = open_archive_v1(&reader).unwrap();

    let mut bytes = fs::read(&archive).unwrap();
    // First block payload byte: [BLK3 header(64)] + payload starts at offset 64.
    bytes[64] ^= 0x01;
    let corrupt_archive = root.join("single-corrupt.crs");
    fs::write(&corrupt_archive, bytes).unwrap();

    let fsck = run_bin(
        "crushr-fsck",
        &[corrupt_archive.to_str().unwrap(), "--json"],
    );
    assert_ok(&fsck);
    let fsck_json: serde_json::Value = serde_json::from_slice(&fsck.stdout).unwrap();
    assert_eq!(fsck_json["tool"], "crushr-fsck");
    assert_eq!(fsck_json["payload"]["verify"]["status"], "ok");
    assert_eq!(
        fsck_json["payload"]["blast_radius"]["impact"]["corrupted_blocks"],
        serde_json::json!([0])
    );
    assert!(
        fsck_json["payload"]["blast_radius"]["impact"]["affected_files"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // Deterministic for identical archive bytes + corruption input.
    let fsck_again = run_bin(
        "crushr-fsck",
        &[corrupt_archive.to_str().unwrap(), "--json"],
    );
    assert_ok(&fsck_again);
    assert_eq!(fsck.stdout, fsck_again.stdout);

    // Tail/footer corruption still fails structurally (exit code 2).
    let mut tail_corrupt = fs::read(&archive).unwrap();
    let last = tail_corrupt.len() - 1;
    tail_corrupt[last] ^= 0x01;
    let footer_corrupt_archive = root.join("single-footer-corrupt.crs");
    fs::write(&footer_corrupt_archive, tail_corrupt).unwrap();

    let fsck_tail = run_bin(
        "crushr-fsck",
        &[footer_corrupt_archive.to_str().unwrap(), "--json"],
    );
    assert_eq!(fsck_tail.status.code(), Some(2));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn extract_single_file_roundtrip() {
    ensure_bins_built();

    let root = unique_dir("crushr-extract-v1-single");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("single.txt");
    fs::write(&input, b"extract me\n").unwrap();
    let archive = root.join("single.crs");
    let out_dir = root.join("out");

    let packed = run_bin(
        "crushr-pack",
        &[input.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let extracted_success = run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
            "--refusal-exit",
            "success",
            "--json",
        ],
    );
    assert_ok(&extracted_success);
    let extracted_success_json: serde_json::Value =
        serde_json::from_slice(&extracted_success.stdout).unwrap();
    assert_eq!(extracted_success_json["overall_status"], "success");
    assert_eq!(
        extracted_success_json["refused_files"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        extracted_success_json["extracted_files"]
            .as_array()
            .unwrap(),
        &[serde_json::json!("single.txt")]
    );
    assert_eq!(
        fs::read(out_dir.join("single.txt")).unwrap(),
        b"extract me\n"
    );

    let out_dir_partial = root.join("out-partial");
    let extracted_partial = run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            out_dir_partial.to_str().unwrap(),
            "--refusal-exit",
            "partial-failure",
            "--json",
        ],
    );
    assert_ok(&extracted_partial);
    let extracted_partial_json: serde_json::Value =
        serde_json::from_slice(&extracted_partial.stdout).unwrap();
    assert_eq!(extracted_partial_json["overall_status"], "success");
    assert_eq!(
        fs::read(out_dir_partial.join("single.txt")).unwrap(),
        b"extract me\n"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn extract_tiny_directory_roundtrip() {
    ensure_bins_built();

    let root = unique_dir("crushr-extract-v1-dir");
    let src = root.join("src");
    fs::create_dir_all(src.join("nested")).unwrap();

    fs::write(src.join("a.txt"), b"alpha\n").unwrap();
    fs::write(src.join("nested/b.txt"), b"bravo\n").unwrap();

    let archive = root.join("tree.crs");
    let out_dir = root.join("out");

    let packed = run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let extracted = run_bin(
        "crushr-extract",
        &[archive.to_str().unwrap(), "-o", out_dir.to_str().unwrap()],
    );
    assert_ok(&extracted);

    assert_eq!(fs::read(out_dir.join("a.txt")).unwrap(), b"alpha\n");
    assert_eq!(fs::read(out_dir.join("nested/b.txt")).unwrap(), b"bravo\n");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn extract_refuses_affected_file_and_keeps_unaffected_file() {
    ensure_bins_built();

    let root = unique_dir("crushr-extract-v1-corrupt");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.txt"), b"alpha\n").unwrap();
    fs::write(src.join("b.txt"), b"bravo\n").unwrap();

    let archive = root.join("tree.crs");
    let packed = run_bin(
        "crushr-pack",
        &[src.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let reader = FileReader {
        file: fs::File::open(&archive).unwrap(),
    };
    let opened = open_archive_v1(&reader).unwrap();
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset).unwrap();

    let mut corrupted = fs::read(&archive).unwrap();
    let payload_start = blocks[0].payload_offset as usize;
    corrupted[payload_start] ^= 0x01;
    let corrupt_archive = root.join("tree-corrupt.crs");
    fs::write(&corrupt_archive, corrupted).unwrap();

    let out_dir_success_a = root.join("out-success-a");
    let extracted_success_a = run_bin(
        "crushr-extract",
        &[
            corrupt_archive.to_str().unwrap(),
            "-o",
            out_dir_success_a.to_str().unwrap(),
            "--refusal-exit",
            "success",
            "--json",
        ],
    );
    assert_ok(&extracted_success_a);
    let extracted_success_a_json: serde_json::Value =
        serde_json::from_slice(&extracted_success_a.stdout).unwrap();
    assert_eq!(
        extracted_success_a_json["overall_status"],
        "partial_refusal"
    );
    assert_eq!(
        extracted_success_a_json["extracted_files"]
            .as_array()
            .unwrap(),
        &[serde_json::json!("b.txt")]
    );
    assert_eq!(
        extracted_success_a_json["refused_files"]
            .as_array()
            .unwrap(),
        &[serde_json::json!({"path": "a.txt", "reason": "corrupted_required_blocks"})]
    );
    assert!(!out_dir_success_a.join("a.txt").exists());
    assert_eq!(
        fs::read(out_dir_success_a.join("b.txt")).unwrap(),
        b"bravo\n"
    );

    let out_dir_success_b = root.join("out-success-b");
    let extracted_success_b = run_bin(
        "crushr-extract",
        &[
            corrupt_archive.to_str().unwrap(),
            "-o",
            out_dir_success_b.to_str().unwrap(),
            "--refusal-exit",
            "success",
            "--json",
        ],
    );
    assert_ok(&extracted_success_b);
    assert_eq!(extracted_success_a.stderr, extracted_success_b.stderr);
    assert_eq!(extracted_success_a.stdout, extracted_success_b.stdout);

    let out_dir_partial_a = root.join("out-partial-a");
    let extracted_partial_a = run_bin(
        "crushr-extract",
        &[
            corrupt_archive.to_str().unwrap(),
            "-o",
            out_dir_partial_a.to_str().unwrap(),
            "--refusal-exit",
            "partial-failure",
            "--json",
        ],
    );
    assert_err_code_3(&extracted_partial_a);
    let extracted_partial_a_json: serde_json::Value =
        serde_json::from_slice(&extracted_partial_a.stdout).unwrap();
    assert_eq!(
        extracted_partial_a_json["overall_status"],
        "partial_refusal"
    );
    assert!(!out_dir_partial_a.join("a.txt").exists());
    assert_eq!(
        fs::read(out_dir_partial_a.join("b.txt")).unwrap(),
        b"bravo\n"
    );

    let out_dir_partial_b = root.join("out-partial-b");
    let extracted_partial_b = run_bin(
        "crushr-extract",
        &[
            corrupt_archive.to_str().unwrap(),
            "-o",
            out_dir_partial_b.to_str().unwrap(),
            "--refusal-exit",
            "partial-failure",
            "--json",
        ],
    );
    assert_err_code_3(&extracted_partial_b);
    assert_eq!(extracted_partial_a.stderr, extracted_partial_b.stderr);
    assert_eq!(extracted_partial_a.stdout, extracted_partial_b.stdout);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn extract_fails_for_invalid_footer() {
    ensure_bins_built();

    let root = unique_dir("crushr-extract-v1-footer");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("single.txt");
    fs::write(&input, b"footer-test\n").unwrap();
    let archive = root.join("single.crs");

    let packed = run_bin(
        "crushr-pack",
        &[input.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    );
    assert_ok(&packed);

    let mut bytes = fs::read(&archive).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    let broken = root.join("single-broken.crs");
    fs::write(&broken, bytes).unwrap();

    let out_dir_success = root.join("out-success");
    let extracted_success = run_bin(
        "crushr-extract",
        &[
            broken.to_str().unwrap(),
            "-o",
            out_dir_success.to_str().unwrap(),
            "--refusal-exit",
            "success",
            "--json",
        ],
    );
    assert_err_code_2(&extracted_success);
    let extracted_success_json: serde_json::Value =
        serde_json::from_slice(&extracted_success.stdout).unwrap();
    assert_eq!(extracted_success_json["overall_status"], "error");

    let out_dir_partial = root.join("out-partial");
    let extracted_partial = run_bin(
        "crushr-extract",
        &[
            broken.to_str().unwrap(),
            "-o",
            out_dir_partial.to_str().unwrap(),
            "--refusal-exit",
            "partial-failure",
            "--json",
        ],
    );
    assert_err_code_2(&extracted_partial);
    assert_eq!(extracted_success.stdout, extracted_partial.stdout);

    let _ = fs::remove_dir_all(&root);
}
