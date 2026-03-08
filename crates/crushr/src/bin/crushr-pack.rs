use anyhow::{bail, Context, Result};
use crushr::format::{Entry, EntryKind, Extent, Index};
use crushr::index_codec::encode_index;
use crushr_format::blk3::{write_blk3_header, Blk3Flags, Blk3Header};
use crushr_format::tailframe::assemble_tail_frame;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

const ZSTD_CODEC: u32 = 1;

#[derive(Debug)]
struct InputFile {
    rel_path: String,
    abs_path: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        let message = format!("{err:#}");
        let code = if message.contains("usage:")
            || message.contains("unsupported flag")
            || message.contains("unexpected argument")
        {
            1
        } else {
            2
        };
        std::process::exit(code);
    }
}

fn run() -> Result<()> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut level: i32 = 3;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-o" || arg == "--output" {
            let value = args
                .next()
                .context("usage: crushr-pack <input>... -o <archive> [--level <n>]")?;
            output = Some(PathBuf::from(value));
        } else if arg == "--level" {
            let value = args
                .next()
                .context("usage: crushr-pack <input>... -o <archive> [--level <n>]")?;
            level = value
                .parse::<i32>()
                .with_context(|| format!("invalid --level value: {value}"))?;
        } else if arg.starts_with('-') {
            bail!("unsupported flag: {arg}");
        } else {
            inputs.push(PathBuf::from(arg));
        }
    }

    let output = output.context("usage: crushr-pack <input>... -o <archive> [--level <n>]")?;
    if inputs.is_empty() {
        bail!("usage: crushr-pack <input>... -o <archive> [--level <n>]");
    }

    pack_minimal_v1(&inputs, &output, level)
}

fn pack_minimal_v1(inputs: &[PathBuf], output: &Path, level: i32) -> Result<()> {
    let files = collect_files(inputs)?;
    if files.is_empty() {
        bail!("no input files to pack");
    }

    let mut out = File::create(output).with_context(|| format!("create {}", output.display()))?;
    let mut entries = Vec::with_capacity(files.len());
    let mut block_id: u32 = 0;

    for file in files {
        let raw = std::fs::read(&file.abs_path)
            .with_context(|| format!("read {}", file.abs_path.display()))?;
        let compressed = zstd::bulk::compress(&raw, level)
            .with_context(|| format!("compress {}", file.abs_path.display()))?;

        let payload_hash = *blake3::hash(&compressed).as_bytes();
        let raw_hash = *blake3::hash(&raw).as_bytes();
        let flags = Blk3Flags(Blk3Flags::HAS_PAYLOAD_HASH | Blk3Flags::HAS_RAW_HASH);
        let header = Blk3Header {
            header_len: (4 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 32 + 32) as u16,
            flags,
            codec: ZSTD_CODEC,
            level,
            dict_id: 0,
            raw_len: raw.len() as u64,
            comp_len: compressed.len() as u64,
            payload_hash: Some(payload_hash),
            raw_hash: Some(raw_hash),
        };

        write_blk3_header(&mut out, &header)?;
        out.write_all(&compressed)?;

        entries.push(Entry {
            path: file.rel_path,
            kind: EntryKind::Regular,
            mode: 0,
            mtime: 0,
            size: raw.len() as u64,
            extents: vec![Extent {
                block_id,
                offset: 0,
                len: raw.len() as u64,
            }],
            link_target: None,
            xattrs: Vec::new(),
        });

        block_id = block_id
            .checked_add(1)
            .context("too many files for minimal packer")?;
    }

    let blocks_end_offset = out.stream_position()?;
    let idx3 = encode_index(&Index { entries });
    let tail = assemble_tail_frame(blocks_end_offset, None, &idx3, None)?;
    out.write_all(&tail)?;

    Ok(())
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<InputFile>> {
    let mut files = Vec::new();

    for input in inputs {
        let abs = std::fs::canonicalize(input)
            .with_context(|| format!("canonicalize {}", input.display()))?;
        let meta =
            std::fs::symlink_metadata(&abs).with_context(|| format!("stat {}", input.display()))?;

        if meta.is_file() {
            let name = abs
                .file_name()
                .context("input file has no file name")?
                .to_string_lossy()
                .to_string();
            files.push(InputFile {
                rel_path: name,
                abs_path: abs,
            });
            continue;
        }

        if !meta.is_dir() {
            bail!("unsupported input type: {}", input.display());
        }

        for entry in walkdir::WalkDir::new(&abs).follow_links(false) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let rel = entry
                .path()
                .strip_prefix(&abs)
                .context("strip input prefix")?
                .to_string_lossy()
                .replace('\\', "/");

            files.push(InputFile {
                rel_path: rel,
                abs_path: entry.path().to_path_buf(),
            });
        }
    }

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(files)
}
