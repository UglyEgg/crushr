// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crushr_format::{
    ftr4::{FTR4_LEN, Ftr4},
    ledger::LedgerBlob,
    tailframe::{assemble_tail_frame, parse_tail_frame},
};
use serde_json::{Value, json};
use std::fs;
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

fn build_archive(pack_bin: &Path, path: &Path) {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("data.txt"), b"redundant-map-payload").unwrap();
    run(Command::new(pack_bin).args([
        input.to_str().unwrap(),
        "-o",
        path.to_str().unwrap(),
        "--level",
        "3",
    ]));
}

fn read_tail(archive_bytes: &[u8]) -> (u64, Vec<u8>, Option<LedgerBlob>) {
    let footer_off = archive_bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&archive_bytes[footer_off..]).unwrap();
    let blocks_end = footer.blocks_end_offset;
    let tail = parse_tail_frame(&archive_bytes[blocks_end as usize..]).unwrap();
    (blocks_end, tail.idx3_bytes, tail.ldg1)
}

fn rewrite_tail(archive_path: &Path, idx3: Vec<u8>, ledger: Option<LedgerBlob>) {
    let bytes = fs::read(archive_path).unwrap();
    let footer_off = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&bytes[footer_off..]).unwrap();
    let blocks_end = footer.blocks_end_offset;
    let mut rewritten = bytes[..blocks_end as usize].to_vec();
    let tail = assemble_tail_frame(blocks_end, None, &idx3, ledger.as_ref()).unwrap();
    rewritten.extend_from_slice(&tail);
    fs::write(archive_path, rewritten).unwrap();
}

fn run_salvage(salvage_bin: &Path, archive: &Path, out: &Path) -> Value {
    run(Command::new(salvage_bin).args([
        archive.to_str().unwrap(),
        "--json-out",
        out.to_str().unwrap(),
    ]));
    serde_json::from_slice(&fs::read(out).unwrap()).unwrap()
}

#[test]
fn salvage_prefers_primary_when_index_valid() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["schema_version"], "crushr-salvage-plan.v3");
    assert_eq!(
        plan["redundant_map_analysis"]["status"], "not_used",
        "redundant map should not be used when primary index verifies"
    );
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "PRIMARY_INDEX_PATH"
    );
}

#[test]
fn salvage_uses_redundant_mapping_when_primary_index_is_damaged() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive);

    let bytes = fs::read(&archive).unwrap();
    let (_blocks_end, idx3, ledger) = read_tail(&bytes);
    let mut corrupted_idx = idx3.clone();
    corrupted_idx[4] ^= 0x7F;
    rewrite_tail(&archive, corrupted_idx, ledger);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["redundant_map_analysis"]["status"], "valid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "REDUNDANT_VERIFIED_MAP_PATH"
    );
}

#[test]
fn salvage_rejects_invalid_redundant_map_and_keeps_orphan_boundary() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive);

    let bytes = fs::read(&archive).unwrap();
    let (_blocks_end, idx3, _ledger) = read_tail(&bytes);
    let mut corrupted_idx = idx3.clone();
    corrupted_idx[4] ^= 0x7F;
    let bad_ledger = LedgerBlob::from_value(&json!({
        "schema": "crushr-redundant-file-map.v1",
        "files": [{
            "path": "data.txt",
            "size": 20,
            "extents": [{"block_id": 99, "file_offset": 0, "len": 20}]
        }]
    }))
    .unwrap();
    rewrite_tail(&archive, corrupted_idx, Some(bad_ledger));

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["redundant_map_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 0);
    assert_eq!(plan["summary"]["unmappable_files"], 1);
}

#[test]
fn salvage_backward_compatible_without_ledger() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive);
    let bytes = fs::read(&archive).unwrap();
    let (blocks_end, idx3, _ledger) = read_tail(&bytes);
    let mut rewritten = bytes[..blocks_end as usize].to_vec();
    let tail = assemble_tail_frame(blocks_end, None, &idx3, None).unwrap();
    rewritten.extend_from_slice(&tail);
    fs::write(&archive, rewritten).unwrap();

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["index_analysis"]["status"], "valid");
    assert_eq!(plan["redundant_map_analysis"]["status"], "not_used");
}

#[test]
fn salvage_is_deterministic_when_using_redundant_map() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive);

    let bytes = fs::read(&archive).unwrap();
    let (_blocks_end, idx3, ledger) = read_tail(&bytes);
    let mut corrupted_idx = idx3.clone();
    corrupted_idx[4] ^= 0x7F;
    rewrite_tail(&archive, corrupted_idx, ledger);

    let first = run_salvage(salvage_bin, &archive, &td.path().join("first.json"));
    let second = run_salvage(salvage_bin, &archive, &td.path().join("second.json"));

    assert_eq!(first["summary"], second["summary"]);
    assert_eq!(first["file_plans"], second["file_plans"]);
    assert_eq!(
        first["redundant_map_analysis"],
        second["redundant_map_analysis"]
    );
}
