// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::Result;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::scan_blocks_v1,
};
use crushr_format::blk3::read_blk3_header;
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn unique_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nonce}"))
}

fn run_bin(bin: &str, args: &[&str]) -> std::process::Output {
    let bin_path = match bin {
        "crushr-pack" => Path::new(env!("CARGO_BIN_EXE_crushr-pack")),
        "crushr-extract" => Path::new(env!("CARGO_BIN_EXE_crushr-extract")),
        _ => panic!("unsupported binary in test: {bin}"),
    };
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

fn corrupt_first_block_payload_hash_only(archive: &Path) {
    let reader = FileReader {
        file: fs::File::open(archive).unwrap(),
    };
    let opened = open_archive_v1(&reader).unwrap();
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset).unwrap();
    let first = &blocks[0];
    let mut bytes = fs::read(archive).unwrap();
    let header_start = first.header_offset as usize;
    let header_end = first.payload_offset as usize;
    let header = read_blk3_header(Cursor::new(&bytes[header_start..header_end])).unwrap();

    let fixed_prefix = 4 + 2 + 2 + 4 + 4 + 4 + 8 + 8;
    assert!(
        header.payload_hash.is_some(),
        "expected payload hash in block"
    );
    let payload_hash_offset = header_start + fixed_prefix;
    bytes[payload_hash_offset] ^= 0x01;
    fs::write(archive, bytes).unwrap();
}

#[test]
fn recover_mode_writes_strict_structure_for_clean_archive() {
    let root = unique_dir("crushr-recover-clean");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("in");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("alpha.txt"), b"alpha\n").unwrap();

    let archive = root.join("clean.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[input.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    let out_dir = root.join("out");
    assert_ok(&run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
            "--recover",
        ],
    ));

    assert!(out_dir.join("canonical/alpha.txt").exists());
    assert!(out_dir.join("metadata_degraded").is_dir());
    assert!(out_dir.join("recovered_named").is_dir());
    assert!(out_dir.join("_crushr_recovery/anonymous").is_dir());
    assert!(out_dir.join("_crushr_recovery/manifest.json").is_file());

    let manifest: Value =
        serde_json::from_slice(&fs::read(out_dir.join("_crushr_recovery/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["schema_version"], "crushr-recovery-manifest.v1");
    assert_eq!(manifest["entries"].as_array().unwrap().len(), 0);
    let named_artifacts = fs::read_dir(out_dir.join("recovered_named"))
        .unwrap()
        .filter_map(|item| item.ok())
        .filter(|item| item.path().is_file())
        .count();
    let anonymous_artifacts = fs::read_dir(out_dir.join("_crushr_recovery/anonymous"))
        .unwrap()
        .filter_map(|item| item.ok())
        .filter(|item| item.path().is_file())
        .count();
    assert_eq!(
        named_artifacts, 0,
        "clean recover run emitted named artifacts"
    );
    assert_eq!(
        anonymous_artifacts, 0,
        "clean recover run emitted anonymous artifacts"
    );

    let stdout = String::from_utf8_lossy(
        &run_bin(
            "crushr-extract",
            &[
                archive.to_str().unwrap(),
                "-o",
                out_dir.to_str().unwrap(),
                "--recover",
                "--overwrite",
            ],
        )
        .stdout,
    )
    .to_string();
    for phase in [
        "archive open",
        "metadata scan",
        "canonical extraction",
        "recovery analysis",
        "recovery extraction",
        "manifest/report finalization",
    ] {
        assert!(
            stdout.contains(phase),
            "expected progress phase in output: {phase}\n{stdout}"
        );
    }
    assert!(stdout.contains("canonical"));
    assert!(stdout.contains("metadata_degraded"));
    assert!(stdout.contains("recovered_named"));
    assert!(stdout.contains("anonymous"));
    assert!(stdout.contains("unrecoverable"));
    assert!(stdout.contains("canonical extraction"));
    assert!(stdout.contains("recovery extraction"));
    assert!(!stdout.contains("recovered output is non-canonical"));
}

#[test]
fn recover_mode_emits_anonymous_artifact_and_manifest_for_damaged_archive() {
    let root = unique_dir("crushr-recover-damaged");
    fs::create_dir_all(&root).unwrap();

    let input = root.join("in");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("alpha.txt"), vec![b'A'; 256 * 1024]).unwrap();
    fs::write(input.join("beta.txt"), vec![b'B'; 256 * 1024]).unwrap();

    let archive = root.join("damaged.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[input.to_str().unwrap(), "-o", archive.to_str().unwrap()],
    ));

    corrupt_first_block_payload_hash_only(&archive);

    let out_dir = root.join("out");
    let out = run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
            "--recover",
        ],
    );
    assert!(
        out.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);

    let manifest: Value =
        serde_json::from_slice(&fs::read(out_dir.join("_crushr_recovery/manifest.json")).unwrap())
            .unwrap();
    let entries = manifest["entries"].as_array().unwrap();
    assert!(
        !entries.is_empty(),
        "expected at least one recovery manifest entry"
    );

    let anonymous = fs::read_dir(out_dir.join("_crushr_recovery/anonymous"))
        .unwrap()
        .filter_map(|item| item.ok())
        .filter(|item| item.path().is_file())
        .count();
    let named = fs::read_dir(out_dir.join("recovered_named"))
        .unwrap()
        .filter_map(|item| item.ok())
        .filter(|item| item.path().is_file())
        .count();
    assert!(
        anonymous + named >= 1,
        "expected recovered artifact (named or anonymous)"
    );

    let first = &entries[0];
    assert!(
        first["recovery_kind"] == "recovered_anonymous"
            || first["recovery_kind"] == "recovered_named"
            || first["recovery_kind"] == "metadata_degraded"
    );
    assert!(first["trust_class"].is_string());
    assert!(
        first["missing_metadata_classes"].is_null() || first["missing_metadata_classes"].is_array()
    );
    assert!(first["failed_metadata_classes"].is_array());
    let assigned = first["assigned_name"].as_str().unwrap();
    assert!(first["classification"]["kind"].is_string());
    assert!(first["classification"]["confidence"].is_string());
    assert!(first["classification"]["basis"].is_string());

    if first["recovery_kind"] == "recovered_anonymous" {
        assert!(assigned.starts_with("file_"));
        assert!(
            assigned.ends_with(".bin") || assigned.contains(".probable-") || assigned.contains('.'),
            "unexpected anonymous recovery name format: {assigned}"
        );
    } else {
        assert!(
            assigned.ends_with(".txt"),
            "unexpected named recovery path format: {assigned}"
        );
    }
    assert!(stdout.contains("recovered output is non-canonical"));
    assert!(stdout.contains("_crushr_recovery/manifest.json"));
}
