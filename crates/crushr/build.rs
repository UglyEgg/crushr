// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::process::Command;

const VERSION_FILE: &str = include_str!("../../VERSION");

fn main() {
    for key in [
        "CRUSHR_VERSION",
        "CRUSHR_GIT_COMMIT",
        "CRUSHR_BUILD_TIMESTAMP",
        "CRUSHR_TARGET_TRIPLE",
        "CRUSHR_RUSTC_VERSION",
        "SOURCE_DATE_EPOCH",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=../../VERSION");
    println!("cargo:rerun-if-changed=../../.git/HEAD");

    emit(
        "CRUSHR_VERSION",
        env_or("CRUSHR_VERSION", version_from_file).unwrap_or_else(unknown),
    );
    emit(
        "CRUSHR_GIT_COMMIT",
        env_or("CRUSHR_GIT_COMMIT", git_commit).unwrap_or_else(unknown),
    );
    emit(
        "CRUSHR_BUILD_TIMESTAMP",
        env_or("CRUSHR_BUILD_TIMESTAMP", build_timestamp).unwrap_or_else(unknown),
    );
    emit(
        "CRUSHR_TARGET_TRIPLE",
        env_or("CRUSHR_TARGET_TRIPLE", target_triple).unwrap_or_else(unknown),
    );
    emit(
        "CRUSHR_RUSTC_VERSION",
        env_or("CRUSHR_RUSTC_VERSION", rustc_version).unwrap_or_else(unknown),
    );
}

fn emit(key: &str, value: String) {
    println!("cargo:rustc-env={key}={value}");
}

fn env_or(key: &str, fallback: fn() -> Option<String>) -> Option<String> {
    std::env::var(key)
        .ok()
        .and_then(clean)
        .or_else(fallback)
        .and_then(clean)
}

fn clean(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn unknown() -> String {
    "unknown".to_string()
}

fn version_from_file() -> Option<String> {
    clean(VERSION_FILE.to_string())
}

fn git_commit() -> Option<String> {
    run_cmd(["git", "rev-parse", "--short=12", "HEAD"])
}

fn build_timestamp() -> Option<String> {
    if let Ok(epoch) = std::env::var("SOURCE_DATE_EPOCH")
        && let Some(epoch) = clean(epoch)
    {
        return run_cmd([
            "date",
            "-u",
            "-d",
            &format!("@{epoch}"),
            "+%Y-%m-%dT%H:%M:%SZ",
        ]);
    }
    run_cmd(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"])
}

fn target_triple() -> Option<String> {
    std::env::var("TARGET").ok()
}

fn rustc_version() -> Option<String> {
    run_cmd(["rustc", "--version"])
}

fn run_cmd<const N: usize>(args: [&str; N]) -> Option<String> {
    let mut cmd = Command::new(args[0]);
    cmd.args(&args[1..]);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}
