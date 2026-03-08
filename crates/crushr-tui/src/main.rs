use anyhow::{bail, Result};

fn usage() -> &'static str {
    "Usage:\n  crushr-tui <archive>\n  crushr-tui --snapshot <crushr-info.json> [--fsck <crushr-fsck.json>]\n\nNotes:\n  Live mode opens the archive directly.\n  Snapshot mode loads JSON outputs from crushr-info/crushr-fsck.\n  This is currently a skeleton (UI not implemented yet).\n"
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        bail!("{}", usage());
    }

    let mut snapshot: Option<String> = None;
    let mut fsck: Option<String> = None;
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
            "--fsck" => {
                i += 1;
                if i >= args.len() {
                    bail!("--fsck requires a path\n\n{}", usage());
                }
                fsck = Some(args[i].clone());
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

    match (snapshot, fsck, archive) {
        (Some(info_json), fsck_json, None) => {
            eprintln!("crushr-tui: snapshot mode requested");
            eprintln!("  info: {info_json}");
            if let Some(p) = fsck_json {
                eprintln!("  fsck: {p}");
            }
            eprintln!("  (UI not implemented yet; snapshot schemas are documented in docs/SNAPSHOT_FORMAT.md)");
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
