use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn run(cmd: &mut std::process::Command) {
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
fn pack_list_extract_roundtrip() {
    let td = TempDir::new().unwrap();
    let in_dir = td.path().join("in");
    let out_dir = td.path().join("out");
    fs::create_dir_all(&in_dir).unwrap();
    fs::create_dir_all(in_dir.join("sub")).unwrap();

    fs::write(in_dir.join("a.txt"), b"hello\n").unwrap();
    fs::write(in_dir.join("sub/b.json"), br#"{"k":"v","n":1}"#).unwrap();

    let archive = td.path().join("test.crs");

    run(std::process::Command::new("cargo")
        .args(["build"])
        .current_dir(env!("CARGO_MANIFEST_DIR")));

    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/crushr");

    run(std::process::Command::new(&bin).args([
        "pack",
        in_dir.to_str().unwrap(),
        "-o",
        archive.to_str().unwrap(),
        "--block-mib",
        "1",
        "--level",
        "3",
    ]));

    run(std::process::Command::new(&bin).args(["list", archive.to_str().unwrap()]));

    fs::create_dir_all(&out_dir).unwrap();
    run(std::process::Command::new(&bin).args([
        "extract",
        archive.to_str().unwrap(),
        "--all",
        "-o",
        out_dir.to_str().unwrap(),
    ]));

    assert_eq!(fs::read(out_dir.join("a.txt")).unwrap(), b"hello\n");
    assert_eq!(
        fs::read(out_dir.join("sub/b.json")).unwrap(),
        br#"{"k":"v","n":1}"#
    );
}
