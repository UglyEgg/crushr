use anyhow::{bail, Context, Result};
use crushr::format::EntryKind;
use crushr::index_codec::decode_index;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    propagation::{build_propagation_report_v1, FileDependencyV1},
    snapshot::{info_envelope_from_open_archive, serialize_snapshot_json},
    verify::verify_block_payloads_v1,
};
use std::collections::BTreeSet;
use std::fs::File;

struct FileReader {
    file: File,
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

fn run() -> Result<()> {
    let mut archive = None;
    let mut json = false;
    let mut report = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--json" {
            json = true;
        } else if arg == "--report" {
            report = Some(args.next().context("missing value for --report")?);
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(arg);
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    let archive = archive.context("usage: crushr-info <archive> --json [--report propagation]")?;
    if !json {
        bail!("only --json output is implemented");
    }

    let reader = FileReader {
        file: File::open(&archive).with_context(|| format!("open {archive}"))?,
    };

    let opened = open_archive_v1(&reader)?;

    if let Some(report_kind) = report {
        if report_kind != "propagation" {
            bail!("unsupported report: {report_kind} (expected propagation)");
        }

        let index = decode_index(&opened.tail.idx3_bytes).context("decode IDX3 index")?;
        let mut file_dependencies = Vec::new();
        for entry in index.entries {
            if entry.kind != EntryKind::Regular {
                continue;
            }
            file_dependencies.push(FileDependencyV1 {
                file_path: entry.path,
                required_blocks: entry.extents.into_iter().map(|e| e.block_id).collect(),
            });
        }

        let corrupted_blocks =
            verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;
        let corrupted_structures = BTreeSet::new();
        let report = build_propagation_report_v1(
            &file_dependencies,
            &corrupted_structures,
            &corrupted_blocks,
        );
        println!("{}", serialize_snapshot_json(&report)?);
        return Ok(());
    }

    let snapshot =
        info_envelope_from_open_archive(&opened, env!("CARGO_PKG_VERSION"), "1970-01-01T00:00:00Z");
    println!("{}", serialize_snapshot_json(&snapshot)?);
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err:#}");
            let msg = format!("{err:#}");
            let code = if msg.contains("usage:")
                || msg.contains("missing value for --report")
                || msg.contains("unsupported report")
                || msg.contains("unsupported flag")
                || msg.contains("unexpected argument")
                || msg.contains("only --json")
            {
                1
            } else {
                2
            };
            std::process::exit(code);
        }
    }
}
