// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::{Result, bail};

fn usage() -> &'static str {
    "Usage:\n  crushr-tui <archive>\n  crushr-tui --snapshot <crushr-info.json> [--verify <crushr-extract-verify.json>]\n\nNotes:\n  Live mode opens the archive directly.\n  Snapshot mode loads JSON outputs from crushr-info/crushr-extract --verify.\n  This is currently a skeleton (UI not implemented yet).\n"
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        bail!("{}", usage());
    }

    let mut snapshot: Option<String> = None;
    let mut verify: Option<String> = None;
    let mut archive: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--snapshot" => {
                i += 1;
                if i >= args.len() {
                    bail!("--snapshot requires a path\n\n{}", usage());
                }
                snapshot = Some(args[i].clone());
            }
            "--verify" => {
                i += 1;
                if i >= args.len() {
                    bail!("--verify requires a path\n\n{}", usage());
                }
                verify = Some(args[i].clone());
            }
            "-h" | "--help" => {
                print!("{}", usage());
                return Ok(());
            }
            other => {
                if other.starts_with('-') {
                    bail!("unknown flag: {other}\n\n{}", usage());
                }
                if archive.is_some() {
                    bail!("unexpected extra argument: {other}\n\n{}", usage());
                }
                archive = Some(other.to_string());
            }
        }
        i += 1;
    }

    match (snapshot, verify, archive) {
        (Some(info_json), verify_json, None) => {
            eprintln!("crushr-tui: snapshot mode requested");
            eprintln!("  info: {info_json}");
            if let Some(p) = verify_json {
                eprintln!("  verify: {p}");
            }
            eprintln!(
                "  (UI not implemented yet; snapshot schemas are documented in docs/SNAPSHOT_FORMAT.md)"
            );
            Ok(())
        }
        (None, None, Some(archive_path)) => {
            eprintln!("crushr-tui: live mode requested");
            eprintln!("  archive: {archive_path}");
            eprintln!(
                "  (UI not implemented yet; planned views are documented in docs/ARCHITECTURE.md)"
            );
            Ok(())
        }
        _ => bail!("invalid arguments\n\n{}", usage()),
    }
}
