// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn parse_cli_options() -> Result<CliOptions> {
    let mut archive = None;
    let mut json = false;
    let mut json_out = None;
    let mut export_fragments = None;
    let mut silent = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--json" {
            json = true;
        } else if arg == "--silent" {
            silent = true;
        } else if arg == "--json-out" {
            let path = args.next().context(USAGE)?;
            json_out = Some(PathBuf::from(path));
        } else if arg == "--export-fragments" {
            let path = args.next().context(USAGE)?;
            export_fragments = Some(PathBuf::from(path));
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else if archive.is_none() {
            archive = Some(PathBuf::from(arg));
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    Ok(CliOptions {
        archive: archive.context(USAGE)?,
        json,
        json_out,
        export_fragments,
        silent,
    })
}
