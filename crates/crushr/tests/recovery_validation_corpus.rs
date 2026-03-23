// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::Result;
use crushr::index_codec::{decode_index, encode_index};
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::scan_blocks_v1,
};
use crushr_format::blk3::read_blk3_header;
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct FileReader {
    file: fs::File,
}

type BlockSpans = Vec<crushr_core::verify::BlockSpanV1>;
type PathBlockMap = BTreeMap<String, Vec<u32>>;

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

fn run_bin(bin: &str, args: &[&str]) -> Output {
    let bin_path = match bin {
        "crushr-pack" => Path::new(env!("CARGO_BIN_EXE_crushr-pack")),
        "crushr-extract" => Path::new(env!("CARGO_BIN_EXE_crushr-extract")),
        _ => panic!("unsupported binary in test: {bin}"),
    };
    Command::new(bin_path).args(args).output().unwrap()
}

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join("crushr-recovery-corpus")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove old test root");
    }
    fs::create_dir_all(&root).expect("create test root");
    root
}

fn assert_ok(out: &Output) {
    assert!(
        out.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn assert_fail(out: &Output) {
    assert!(
        !out.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write_fixture_corpus(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    fs::create_dir_all(dir.join("docs/nested")).unwrap();
    fs::create_dir_all(dir.join("a")).unwrap();
    fs::create_dir_all(dir.join("b")).unwrap();
    fs::create_dir_all(dir.join("empty_dir")).unwrap();

    fs::write(dir.join("readme.txt"), b"crushr recovery corpus\n").unwrap();
    fs::write(dir.join("config.json"), b"{\"name\":\"crushr\",\"v\":1}\n").unwrap();
    fs::write(dir.join("layout.xml"), b"<root><item id=\"1\"/></root>\n").unwrap();
    fs::write(
        dir.join("index.html"),
        b"<!doctype html><html><body>ok</body></html>\n",
    )
    .unwrap();

    fs::write(
        dir.join("image.png"),
        b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01",
    )
    .unwrap();
    fs::write(dir.join("doc.pdf"), b"%PDF-1.4\n1 0 obj<</Type/Catalog>>\n").unwrap();
    let mut sqlite = vec![0u8; 128];
    sqlite[..16].copy_from_slice(b"SQLite format 3\0");
    fs::write(dir.join("data.sqlite"), sqlite).unwrap();
    fs::write(
        dir.join("bundle.zip"),
        b"PK\x03\x04zip-content-without-office-markers",
    )
    .unwrap();

    fs::write(
        dir.join("report.docx"),
        b"PK\x03\x04[Content_Types].xml word/document.xml",
    )
    .unwrap();
    fs::write(
        dir.join("table.xlsx"),
        b"PK\x03\x04[Content_Types].xml xl/workbook.xml",
    )
    .unwrap();
    fs::write(
        dir.join("note.odt"),
        b"PK\x03\x04mimetypeapplication/vnd.oasis.opendocument.text",
    )
    .unwrap();

    fs::write(dir.join("docs/nested/repeat.txt"), b"nested repeat\n").unwrap();
    fs::write(dir.join("a/repeat.txt"), b"a repeat\n").unwrap();
    fs::write(dir.join("b/repeat.txt"), b"b repeat\n").unwrap();
}

fn build_split_recovery_corpus(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("canon.txt"), b"canonical file\n").unwrap();
    fs::write(dir.join("named_payload.txt"), vec![b'N'; 120_000]).unwrap();

    let mut high = vec![0u8; 90_000];
    high[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    high[8..16].copy_from_slice(b"IHDRDATA");
    fs::write(dir.join("anon_high.bin"), high).unwrap();

    let mut medium = vec![0u8; 90_000];
    medium[..4].copy_from_slice(b"KDMV");
    medium[4..35].copy_from_slice(b"vmdk deterministic corpus bytes");
    fs::write(dir.join("anon_medium.bin"), medium).unwrap();

    let mut low = vec![0u8; 90_000];
    for (idx, byte) in low.iter_mut().enumerate() {
        *byte = ((idx * 17 + 31) % 251) as u8;
    }
    fs::write(dir.join("anon_low.bin"), low).unwrap();

    fs::write(dir.join("gone.bin"), vec![b'G'; 80_000]).unwrap();
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
    assert_eq!(
        new_index.len(),
        open.tail.idx3_bytes.len(),
        "index rewrite must preserve byte length"
    );

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

fn open_blocks_and_index(archive: &Path) -> Result<(BlockSpans, PathBlockMap)> {
    let reader = FileReader {
        file: fs::File::open(archive)?,
    };
    let open = open_archive_v1(&reader)?;
    let blocks = scan_blocks_v1(&reader, open.tail.footer.blocks_end_offset)?;
    let index = decode_index(&open.tail.idx3_bytes)?;
    let map = index
        .entries
        .into_iter()
        .map(|entry| {
            (
                entry.path,
                entry
                    .extents
                    .into_iter()
                    .map(|extent| extent.block_id)
                    .collect(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    Ok((blocks, map))
}

fn flip_block_payload_hash_bit(archive: &Path, block_id: u32) {
    let (blocks, _) = open_blocks_and_index(archive).expect("open archive for hash flip");
    let span = blocks
        .iter()
        .find(|b| b.block_id == block_id)
        .expect("block for hash flip");

    let mut bytes = fs::read(archive).unwrap();
    let header_start = span.header_offset as usize;
    let header_end = span.payload_offset as usize;
    let header = read_blk3_header(Cursor::new(&bytes[header_start..header_end])).unwrap();
    assert!(
        header.payload_hash.is_some(),
        "expected payload hash in block"
    );

    let fixed_prefix = 4 + 2 + 2 + 4 + 4 + 4 + 8 + 8;
    let payload_hash_offset = header_start + fixed_prefix;
    bytes[payload_hash_offset] ^= 0x01;
    fs::write(archive, bytes).unwrap();
}

fn corrupt_index_bytes(archive: &Path) {
    let reader = FileReader {
        file: fs::File::open(archive).unwrap(),
    };
    let open = open_archive_v1(&reader).unwrap();
    let mut bytes = fs::read(archive).unwrap();
    let index_offset = open.tail.footer.index_offset as usize;
    bytes[index_offset + 8] ^= 0x80;
    fs::write(archive, bytes).unwrap();
}

fn truncate_tail(archive: &Path, tail_bytes: usize) {
    let mut bytes = fs::read(archive).unwrap();
    let new_len = bytes.len().saturating_sub(tail_bytes);
    bytes.truncate(new_len);
    fs::write(archive, bytes).unwrap();
}

fn read_manifest(out_dir: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(out_dir.join("_crushr_recovery/manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest")
}

fn run_strict(archive: &Path, out_dir: &Path) -> Output {
    run_bin(
        "crushr-extract",
        &[archive.to_str().unwrap(), "-o", out_dir.to_str().unwrap()],
    )
}

fn run_recover(archive: &Path, out_dir: &Path) -> Output {
    run_bin(
        "crushr-extract",
        &[
            archive.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
            "--recover",
        ],
    )
}

#[test]
fn deterministic_recovery_validation_corpus_covers_strict_and_recover_modes() {
    let root = test_root("validation-suite");

    let source = root.join("source");
    write_fixture_corpus(&source);

    let clean_archive = root.join("clean.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[
            source.to_str().unwrap(),
            "-o",
            clean_archive.to_str().unwrap(),
        ],
    ));

    let strict_clean_out = root.join("strict-clean");
    assert_ok(&run_strict(&clean_archive, &strict_clean_out));
    assert!(strict_clean_out.join("report.docx").is_file());
    assert!(strict_clean_out.join("table.xlsx").is_file());
    assert!(strict_clean_out.join("note.odt").is_file());

    let recover_clean_out = root.join("recover-clean");
    assert_ok(&run_recover(&clean_archive, &recover_clean_out));
    let clean_manifest = read_manifest(&recover_clean_out);
    assert_eq!(clean_manifest["entries"].as_array().unwrap().len(), 0);
    assert!(recover_clean_out.join("canonical/readme.txt").is_file());

    let trunc_archive = root.join("tail-truncated.crs");
    fs::copy(&clean_archive, &trunc_archive).unwrap();
    truncate_tail(&trunc_archive, 128);
    assert_fail(&run_strict(
        &trunc_archive,
        &root.join("strict-tail-truncated"),
    ));
    assert_fail(&run_recover(
        &trunc_archive,
        &root.join("recover-tail-truncated"),
    ));

    let index_archive = root.join("index-damaged.crs");
    fs::copy(&clean_archive, &index_archive).unwrap();
    corrupt_index_bytes(&index_archive);
    assert_fail(&run_strict(
        &index_archive,
        &root.join("strict-index-damaged"),
    ));
    assert_fail(&run_recover(
        &index_archive,
        &root.join("recover-index-damaged"),
    ));

    let hash_archive = root.join("payload-hash-damaged.crs");
    fs::copy(&clean_archive, &hash_archive).unwrap();
    let (_, clean_map) = open_blocks_and_index(&hash_archive).unwrap();
    let named_block = clean_map
        .get("readme.txt")
        .and_then(|ids| ids.first())
        .copied()
        .expect("readme block id");
    flip_block_payload_hash_bit(&hash_archive, named_block);
    let strict_hash = run_strict(&hash_archive, &root.join("strict-payload-hash-damaged"));
    assert_ok(&strict_hash);
    let strict_hash_stdout = String::from_utf8_lossy(&strict_hash.stdout);
    assert!(strict_hash_stdout.contains("refused files"));
    assert!(strict_hash_stdout.contains("DEGRADED"));

    let recover_hash_out = root.join("recover-payload-hash-damaged");
    assert_ok(&run_recover(&hash_archive, &recover_hash_out));
    let hash_manifest = read_manifest(&recover_hash_out);
    let hash_entries = hash_manifest["entries"].as_array().unwrap();
    assert_eq!(hash_entries.len(), 1);
    assert_eq!(hash_entries[0]["recovery_kind"], "recovered_named");
    assert_eq!(hash_entries[0]["classification"]["confidence"], "medium");
    assert_eq!(
        hash_entries[0]["original_identity"]["path_status"],
        "untrusted"
    );
    assert!(
        recover_hash_out
            .join("recovered_named/readme.txt")
            .is_file(),
        "expected named recovered file"
    );

    let split_source = root.join("split-source");
    build_split_recovery_corpus(&split_source);
    let split_archive = root.join("mixed.crs");
    assert_ok(&run_bin(
        "crushr-pack",
        &[
            split_source.to_str().unwrap(),
            "-o",
            split_archive.to_str().unwrap(),
        ],
    ));

    mutate_index_in_place(&split_archive, |index| {
        for entry in &mut index.entries {
            if matches!(
                entry.path.as_str(),
                "anon_high.bin" | "anon_medium.bin" | "anon_low.bin"
            ) {
                entry.size += 1;
            }
            if entry.path == "gone.bin" {
                entry.size += 1;
                entry.extents[0].len += 1;
            }
        }
    });

    let (_, split_map) = open_blocks_and_index(&split_archive).unwrap();
    flip_block_payload_hash_bit(
        &split_archive,
        split_map["named_payload.txt"].first().copied().unwrap(),
    );
    flip_block_payload_hash_bit(&split_archive, split_map["anon_high.bin"][0]);
    flip_block_payload_hash_bit(&split_archive, split_map["anon_medium.bin"][0]);
    flip_block_payload_hash_bit(&split_archive, split_map["anon_low.bin"][0]);
    flip_block_payload_hash_bit(&split_archive, split_map["gone.bin"][0]);

    let strict_mixed = run_strict(&split_archive, &root.join("strict-mixed"));
    assert_ok(&strict_mixed);
    let strict_mixed_stdout = String::from_utf8_lossy(&strict_mixed.stdout);
    assert!(strict_mixed_stdout.contains("refused files"));
    assert!(strict_mixed_stdout.contains("DEGRADED"));

    let recover_mixed_out = root.join("recover-mixed");
    assert_ok(&run_recover(&split_archive, &recover_mixed_out));

    assert!(recover_mixed_out.join("canonical/canon.txt").is_file());
    assert!(
        recover_mixed_out
            .join("recovered_named/named_payload.txt")
            .is_file()
    );
    assert!(
        recover_mixed_out
            .join("_crushr_recovery/anonymous/file_000001.png")
            .is_file()
    );
    assert!(
        recover_mixed_out
            .join("_crushr_recovery/anonymous/file_000002.bin")
            .is_file()
    );
    assert!(
        recover_mixed_out
            .join("_crushr_recovery/anonymous/file_000003.probable-vmdk.bin")
            .is_file()
    );

    let mixed_manifest = read_manifest(&recover_mixed_out);
    let entries = mixed_manifest["entries"].as_array().unwrap();
    assert_eq!(
        entries.len(),
        5,
        "expected 5 non-canonical entries in manifest"
    );

    let mut by_assigned = BTreeMap::new();
    for entry in entries {
        let key = entry["assigned_name"]
            .as_str()
            .unwrap_or("<none>")
            .to_string();
        by_assigned.insert(key, entry.clone());
    }

    assert_eq!(
        by_assigned["named_payload.txt"]["recovery_kind"],
        "recovered_named"
    );
    assert_eq!(
        by_assigned["file_000001.png"]["classification"]["confidence"],
        "high"
    );
    assert_eq!(
        by_assigned["file_000002.bin"]["classification"]["confidence"],
        "low"
    );
    assert_eq!(
        by_assigned["file_000003.probable-vmdk.bin"]["classification"]["confidence"],
        "medium"
    );

    let unrecoverable = entries
        .iter()
        .find(|entry| entry["recovery_kind"] == "unrecoverable")
        .expect("unrecoverable entry");
    assert_eq!(unrecoverable["assigned_name"], Value::Null);
    assert_eq!(unrecoverable["size"], 0);
    assert_eq!(unrecoverable["hash"], Value::Null);
    assert_eq!(unrecoverable["classification"]["kind"], "bin");
    assert_eq!(unrecoverable["classification"]["confidence"], "low");

    for entry in entries {
        let kind = entry["recovery_kind"].as_str().unwrap();
        match kind {
            "recovered_named" => {
                let path = recover_mixed_out.join("recovered_named").join(
                    entry["assigned_name"]
                        .as_str()
                        .expect("named entry has path"),
                );
                assert!(path.is_file(), "manifest named entry not on disk: {path:?}");
            }
            "recovered_anonymous" => {
                let path = recover_mixed_out.join("_crushr_recovery/anonymous").join(
                    entry["assigned_name"]
                        .as_str()
                        .expect("anonymous entry has path"),
                );
                assert!(
                    path.is_file(),
                    "manifest anonymous entry not on disk: {path:?}"
                );
                assert_eq!(entry["original_identity"]["path_status"], "lost");
                assert_eq!(entry["original_identity"]["name_status"], "lost");
            }
            "unrecoverable" => {
                assert_eq!(entry["assigned_name"], Value::Null);
            }
            other => panic!("unexpected recovery kind: {other}"),
        }
    }
}
