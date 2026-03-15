use crushr_format::{
    ftr4::{Ftr4, FTR4_LEN},
    tailframe::{assemble_tail_frame, parse_tail_frame},
};
use serde_json::Value;
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

fn build_archive(pack_bin: &Path, path: &Path, experimental: bool) {
    let td = TempDir::new().unwrap();
    let input = td.path().join("input");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("data.txt"), b"experimental-resilience-payload").unwrap();

    let mut cmd = Command::new(pack_bin);
    cmd.args([
        input.to_str().unwrap(),
        "-o",
        path.to_str().unwrap(),
        "--level",
        "3",
    ]);
    if experimental {
        cmd.arg("--experimental-self-describing-extents");
    }
    run(&mut cmd);
}

fn rewrite_tail_without_ledger_and_with_corrupt_index(archive_path: &Path) {
    let bytes = fs::read(archive_path).unwrap();
    let footer_off = bytes.len() - FTR4_LEN;
    let footer = Ftr4::read_from(&bytes[footer_off..]).unwrap();
    let blocks_end = footer.blocks_end_offset;
    let tail = parse_tail_frame(&bytes[blocks_end as usize..]).unwrap();

    let mut bad_idx = tail.idx3_bytes;
    bad_idx[4] ^= 0x7f;

    let mut rewritten = bytes[..blocks_end as usize].to_vec();
    let rebuilt = assemble_tail_frame(blocks_end, None, &bad_idx, None).unwrap();
    rewritten.extend_from_slice(&rebuilt);
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
fn experimental_archive_uses_checkpoint_path_when_primary_and_ledger_are_unusable() {
    let pack_bin = Path::new(env!("CARGO_BIN_EXE_crushr-pack"));
    let salvage_bin = Path::new(env!("CARGO_BIN_EXE_crushr-salvage"));
    let td = TempDir::new().unwrap();
    let archive = td.path().join("archive.crushr");

    build_archive(pack_bin, &archive, true);
    rewrite_tail_without_ledger_and_with_corrupt_index(&archive);

    let plan = run_salvage(salvage_bin, &archive, &td.path().join("plan.json"));
    assert_eq!(plan["index_analysis"]["status"], "invalid");
    assert_eq!(plan["summary"]["salvageable_files"], 1);
    assert_eq!(
        plan["file_plans"][0]["mapping_provenance"],
        "CHECKPOINT_MAP_PATH"
    );
}

#[test]
fn experimental_comparison_outputs_three_arm_summary() {
    let lab_bin = Path::new(env!("CARGO_BIN_EXE_crushr-lab-salvage"));
    let td = TempDir::new().unwrap();
    let out_dir = td.path().join("comparison");

    run(Command::new(lab_bin)
        .arg("run-experimental-resilience-comparison")
        .arg("--output")
        .arg(&out_dir));

    let summary_path = out_dir.join("experimental_comparison_summary.json");
    assert!(summary_path.exists());
    assert!(out_dir.join("experimental_comparison_summary.md").exists());

    let summary: Value = serde_json::from_slice(&fs::read(summary_path).unwrap()).unwrap();
    assert_eq!(summary["scenario_count"], 24);
    assert!(summary["old_outcome_counts"].is_object());
    assert!(summary["redundant_outcome_counts"].is_object());
    assert!(summary["experimental_outcome_counts"].is_object());
}
