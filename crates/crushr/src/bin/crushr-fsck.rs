use anyhow::{bail, Context, Result};
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    snapshot::{fsck_envelope_from_open_archive, serialize_snapshot_json},
};
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

    for arg in std::env::args().skip(1) {
        if arg == "--json" {
            json = true;
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(arg);
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    let archive = archive.context("usage: crushr-fsck <archive> --json")?;
    if !json {
        bail!("only --json output is implemented");
    }

    let reader = FileReader {
        file: File::open(&archive).with_context(|| format!("open {archive}"))?,
    };

    let opened = open_archive_v1(&reader)?;
    let snapshot =
        fsck_envelope_from_open_archive(&opened, env!("CARGO_PKG_VERSION"), "1970-01-01T00:00:00Z");
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
