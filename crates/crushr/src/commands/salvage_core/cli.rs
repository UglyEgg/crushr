// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn parse_cli_options(args: Vec<String>) -> Result<CliOptions> {
    let early_args = args;

    let mut archive = None;
    let mut json = false;
    let mut json_out = None;
    let mut export_fragments = None;
    let mut silent = false;

    let mut iter = early_args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--json" {
            json = true;
        } else if arg == "--silent" {
            silent = true;
        } else if arg == "--json-out" {
            let path = iter.next().context(USAGE)?;
            json_out = Some(PathBuf::from(path));
        } else if arg == "--export-fragments" {
            let path = iter.next().context(USAGE)?;
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
