// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn version_file_is_strict_semver_and_matches_runtime_surface() {
    let root = repo_root();
    let version_raw = fs::read_to_string(root.join("VERSION")).expect("read VERSION");
    let version = version_raw.trim_end_matches(['\r', '\n']);

    assert_eq!(
        version,
        version.trim(),
        "VERSION must contain only a strict SemVer value"
    );
    assert!(
        crushr::versioning::validate_semver_strict(version),
        "VERSION is not strict SemVer: {version}"
    );
    assert_eq!(version, crushr::product_version());
}

#[test]
fn workspace_cargo_version_matches_version_file() {
    let root = repo_root();
    let version_raw = fs::read_to_string(root.join("VERSION")).expect("read VERSION");
    let version = version_raw.trim_end_matches(['\r', '\n']);
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("read workspace Cargo.toml");

    let mut in_workspace_package = false;
    let mut workspace_version = None;
    for line in cargo.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if in_workspace_package && trimmed.starts_with("version") {
            workspace_version = trimmed
                .split_once('=')
                .map(|(_, rhs)| rhs.trim().trim_matches('"').to_string());
            break;
        }
    }

    let workspace_version = workspace_version.expect("workspace.package.version not found");
    assert_eq!(
        workspace_version, version,
        "workspace.package.version must match VERSION"
    );
}
