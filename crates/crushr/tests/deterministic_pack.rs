// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crushr::index_codec::decode_index;
use crushr_format::{
    ftr4::{Ftr4, FTR4_LEN},
    tailframe::parse_tail_frame,
};
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

fn canonical(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn assert_ordered_duplicate_sources(stderr: &str, logical_path: &str, sources: &[String]) {
    let expected = format!(
        "duplicate logical archive path '{logical_path}' from inputs: {}",
        sources.join(", ")
    );
    assert!(
        stderr.contains(&expected),
        "stderr did not contain expected ordered message: {stderr}"
    );
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

    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let one = td.path().join("one.crs");
    let two = td.path().join("two.crs");

    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        one.to_str().unwrap(),
        "--level",
        "3",
    ]));
    run(Command::new(bin).args([
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
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    run(Command::new(bin).args([
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

#[test]
fn crushr_pack_help_lists_experimental_flags() {
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(bin).arg("--help").output().expect("run help");
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--experimental-self-describing-extents"));
    assert!(stdout.contains("--experimental-file-identity-extents"));
    assert!(stdout.contains("--experimental-self-identifying-blocks"));
}

#[test]
fn crushr_pack_accepts_experimental_writer_flags_and_emits_archives() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("payload.txt"), b"payload").unwrap();

    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let self_describing = td.path().join("self-describing.crushr");
    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        self_describing.to_str().unwrap(),
        "--level",
        "3",
        "--experimental-self-describing-extents",
    ]));
    assert!(self_describing.exists());

    let file_identity = td.path().join("file-identity.crushr");
    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        file_identity.to_str().unwrap(),
        "--level",
        "3",
        "--experimental-file-identity-extents",
    ]));
    assert!(file_identity.exists());

    let format05 = td.path().join("format05.crushr");
    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        format05.to_str().unwrap(),
        "--level",
        "3",
        "--experimental-self-identifying-blocks",
    ]));
    assert!(format05.exists());
}

#[test]
fn crushr_pack_distinct_standalone_paths_succeed() {
    let td = TempDir::new().unwrap();
    let left = td.path().join("left.txt");
    let right = td.path().join("right.txt");
    fs::write(&left, b"left").unwrap();
    fs::write(&right, b"right").unwrap();

    let archive = td.path().join("distinct.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    run(Command::new(bin).args([
        left.to_str().unwrap(),
        right.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--level",
        "3",
    ]));

    assert!(archive.exists());
}

#[test]
fn crushr_pack_rejects_duplicate_basename_collisions_before_archive_create() {
    let td = TempDir::new().unwrap();
    let left_dir = td.path().join("left");
    let right_dir = td.path().join("right");
    fs::create_dir_all(&left_dir).unwrap();
    fs::create_dir_all(&right_dir).unwrap();
    let left = left_dir.join("same.txt");
    let right = right_dir.join("same.txt");
    fs::write(&left, b"left").unwrap();
    fs::write(&right, b"right").unwrap();

    let archive = td.path().join("collision.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(bin)
        .args([
            left.to_str().unwrap(),
            right.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run command");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut expected_sources = vec![canonical(&left), canonical(&right)];
    expected_sources.sort();
    assert_ordered_duplicate_sources(&stderr, "same.txt", &expected_sources);
    assert!(
        !archive.exists(),
        "archive should not be created on duplicate"
    );
}

#[test]
fn crushr_pack_rejects_normalized_path_collisions() {
    let td = TempDir::new().unwrap();
    let standalone = td.path().join(r"dir\item.txt");
    fs::write(&standalone, b"standalone").unwrap();

    let tree_root = td.path().join("tree");
    fs::create_dir_all(tree_root.join("dir")).unwrap();
    fs::write(tree_root.join("dir/item.txt"), b"tree").unwrap();

    let archive = td.path().join("normalized-collision.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(bin)
        .args([
            standalone.to_str().unwrap(),
            tree_root.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run command");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut expected_sources = vec![
        canonical(&standalone),
        canonical(&tree_root.join("dir/item.txt")),
    ];
    expected_sources.sort();
    assert_ordered_duplicate_sources(&stderr, "dir/item.txt", &expected_sources);
    assert!(
        !archive.exists(),
        "archive should not be created on duplicate"
    );
}

#[test]
fn crushr_pack_metadata_profile_runs_remain_byte_identical() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(input.join("nested")).unwrap();
    fs::write(input.join("nested/a.txt"), b"aaa").unwrap();
    fs::write(input.join("nested/b.txt"), b"bbb").unwrap();

    let one = td.path().join("one.crs");
    let two = td.path().join("two.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        one.to_str().unwrap(),
        "--level",
        "3",
        "--metadata-profile",
        "extent_identity_path_dict_header_tail",
    ]));
    run(Command::new(bin).args([
        input.to_str().unwrap(),
        "-o",
        two.to_str().unwrap(),
        "--level",
        "3",
        "--metadata-profile",
        "extent_identity_path_dict_header_tail",
    ]));

    assert_eq!(fs::read(one).unwrap(), fs::read(two).unwrap());
}

#[test]
fn crushr_pack_payload_only_profile_records_profile_in_redundant_map() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("payload.txt"), b"payload").unwrap();
    let archive = td.path().join("archive.crs");
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));

    run(Command::new(pack_bin).args([
        input.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--metadata-profile",
        "payload_only",
    ]));
    let bytes = fs::read(archive).unwrap();
    let footer_off = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&bytes[footer_off..]).unwrap();
    let tail = parse_tail_frame(&bytes[footer.blocks_end_offset as usize..]).unwrap();
    let map_bytes = tail
        .ldg1
        .expect("tail should include redundant map ledger")
        .json;
    let map: serde_json::Value = serde_json::from_slice(&map_bytes).expect("parse ledger json");
    assert_eq!(map["schema"], "crushr-redundant-file-map.experimental.v2");
    assert_eq!(map["experimental_metadata_profile"], "payload_only");
}

#[test]
fn crushr_pack_rejects_walked_tree_to_walked_tree_collisions_with_ordered_sources() {
    let td = TempDir::new().unwrap();
    let first = td.path().join("first");
    let second = td.path().join("second");
    fs::create_dir_all(first.join("dir")).unwrap();
    fs::create_dir_all(second.join("dir")).unwrap();
    let first_file = first.join("dir/item.txt");
    let second_file = second.join("dir/item.txt");
    fs::write(&first_file, b"one").unwrap();
    fs::write(&second_file, b"two").unwrap();

    let archive = td.path().join("tree-tree-collision.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(bin)
        .args([
            first.to_str().unwrap(),
            second.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run command");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut expected_sources = vec![canonical(&first_file), canonical(&second_file)];
    expected_sources.sort();
    assert_ordered_duplicate_sources(&stderr, "dir/item.txt", &expected_sources);
    assert!(
        !archive.exists(),
        "archive should not be created on duplicate"
    );
}

#[test]
fn crushr_pack_rejects_three_way_collisions_with_stable_source_ordering() {
    let td = TempDir::new().unwrap();
    let a = td.path().join("a");
    let b = td.path().join("b");
    let c = td.path().join("c");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    fs::create_dir_all(&c).unwrap();

    let a_file = a.join("same.txt");
    let b_file = b.join("same.txt");
    let c_file = c.join("same.txt");
    fs::write(&a_file, b"a").unwrap();
    fs::write(&b_file, b"b").unwrap();
    fs::write(&c_file, b"c").unwrap();

    let archive = td.path().join("three-way-collision.crs");
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let out = Command::new(bin)
        .args([
            a_file.to_str().unwrap(),
            b_file.to_str().unwrap(),
            c_file.to_str().unwrap(),
            "-o",
            archive.to_str().unwrap(),
            "--level",
            "3",
        ])
        .output()
        .expect("run command");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut expected_sources = vec![canonical(&a_file), canonical(&b_file), canonical(&c_file)];
    expected_sources.sort();
    assert_ordered_duplicate_sources(&stderr, "same.txt", &expected_sources);
    assert!(
        !archive.exists(),
        "archive should not be created on duplicate"
    );
}

#[test]
fn placement_strategy_is_deterministic_per_strategy() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    for i in 0..6 {
        fs::write(input.join(format!("f{i}.txt")), format!("payload-{i}")).unwrap();
    }
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    for strategy in ["fixed_spread", "hash_spread", "golden_spread"] {
        let a = td.path().join(format!("{strategy}-a.crushr"));
        let b = td.path().join(format!("{strategy}-b.crushr"));
        run(Command::new(bin).args([
            input.to_str().unwrap(),
            "-o",
            a.to_str().unwrap(),
            "--level",
            "3",
            "--experimental-self-identifying-blocks",
            "--experimental-file-manifest-checkpoints",
            "--placement-strategy",
            strategy,
        ]));
        run(Command::new(bin).args([
            input.to_str().unwrap(),
            "-o",
            b.to_str().unwrap(),
            "--level",
            "3",
            "--experimental-self-identifying-blocks",
            "--experimental-file-manifest-checkpoints",
            "--placement-strategy",
            strategy,
        ]));
        assert_eq!(fs::read(a).unwrap(), fs::read(b).unwrap());
    }
}

#[test]
fn placement_strategies_differ_for_representative_archive_size() {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    for i in 0..9 {
        fs::write(input.join(format!("f{i}.txt")), format!("payload-{i}")).unwrap();
    }
    let bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let mut outputs = Vec::new();
    for strategy in ["fixed_spread", "hash_spread", "golden_spread"] {
        let a = td.path().join(format!("{strategy}.crushr"));
        run(Command::new(bin).args([
            input.to_str().unwrap(),
            "-o",
            a.to_str().unwrap(),
            "--level",
            "3",
            "--experimental-self-identifying-blocks",
            "--experimental-file-manifest-checkpoints",
            "--placement-strategy",
            strategy,
        ]));
        outputs.push(fs::read(a).unwrap());
    }
    assert!(outputs[0] != outputs[1] || outputs[1] != outputs[2] || outputs[0] != outputs[2]);
}
