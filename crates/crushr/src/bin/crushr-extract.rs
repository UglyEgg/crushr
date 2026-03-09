use anyhow::{bail, Context, Result};
use crushr::format::{Entry, EntryKind};
use crushr::index_codec::decode_index;
use crushr_core::{
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{scan_blocks_v1, verify_block_payloads_v1},
};
use crushr_format::blk3::read_blk3_header;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
struct BlockPayload {
    raw: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefusalExitPolicy {
    Success,
    PartialFailure,
}

impl RefusalExitPolicy {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "success" => Some(Self::Success),
            "partial-failure" => Some(Self::PartialFailure),
            _ => None,
        }
    }
}

const USAGE: &str =
    "usage: crushr-extract <archive> -o <out-dir> [--overwrite] [--refusal-exit <success|partial-failure>] [--json]";

#[derive(Debug)]
struct CliOptions {
    archive: PathBuf,
    out_dir: PathBuf,
    overwrite: bool,
    refusal_exit: RefusalExitPolicy,
    json: bool,
}

#[derive(Debug, Serialize)]
struct RefusedFileReport {
    path: String,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
struct ExtractionReport {
    overall_status: &'static str,
    extracted_files: Vec<String>,
    refused_files: Vec<RefusedFileReport>,
}

#[derive(Debug, Serialize)]
struct ExtractionErrorReport {
    overall_status: &'static str,
    error: String,
}

fn parse_cli_options() -> Result<CliOptions> {
    let mut archive = None;
    let mut out_dir = None;
    let mut overwrite = false;
    let mut refusal_exit = RefusalExitPolicy::Success;
    let mut json = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-o" || arg == "--output" {
            let value = args.next().context(USAGE)?;
            out_dir = Some(PathBuf::from(value));
        } else if arg == "--overwrite" {
            overwrite = true;
        } else if arg == "--json" {
            json = true;
        } else if arg == "--refusal-exit" {
            let value = args.next().context(USAGE)?;
            refusal_exit = RefusalExitPolicy::parse(&value).with_context(|| {
                format!(
                    "unsupported value for --refusal-exit: {value} (expected success|partial-failure)"
                )
            })?;
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
        out_dir: out_dir.context(USAGE)?,
        overwrite,
        refusal_exit,
        json,
    })
}

fn run(opts: &CliOptions) -> Result<ExtractionReport> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };

    let opened = open_archive_v1(&reader)?;
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    let index = decode_index(&opened.tail.idx3_bytes).context("decode IDX3")?;
    let corrupted = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;

    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("create {}", opts.out_dir.display()))?;

    let mut payload_cache = BTreeMap::<u32, BlockPayload>::new();
    let mut entries: Vec<Entry> = index.entries;
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let mut extracted_files = Vec::new();
    let mut refused_files = Vec::new();

    for entry in entries {
        if entry.kind != EntryKind::Regular {
            bail!(
                "unsupported entry kind for strict extraction: {}",
                entry.path
            );
        }

        if entry
            .extents
            .iter()
            .any(|extent| corrupted.contains(&extent.block_id))
        {
            refused_files.push(RefusedFileReport {
                path: entry.path,
                reason: "corrupted_required_blocks",
            });
            continue;
        }

        let bytes = read_entry_bytes_strict(&reader, &entry, &blocks, &mut payload_cache)?;
        let destination = opts.out_dir.join(&entry.path);
        write_entry(destination.as_path(), &bytes, opts.overwrite)?;
        extracted_files.push(entry.path);
    }

    if !refused_files.is_empty() {
        for refused in &refused_files {
            eprintln!(
                "strict: refused extraction due to corrupted required blocks: {}",
                refused.path
            );
        }
    }

    Ok(ExtractionReport {
        overall_status: if refused_files.is_empty() {
            "success"
        } else {
            "partial_refusal"
        },
        extracted_files,
        refused_files,
    })
}

fn read_entry_bytes_strict(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[crushr_core::verify::BlockSpanV1],
    payload_cache: &mut BTreeMap<u32, BlockPayload>,
) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(entry.size as usize);

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;

        let raw = block_raw_payload(reader, block, payload_cache)?;

        let begin = extent.offset as usize;
        let end = begin
            .checked_add(extent.len as usize)
            .context("extent length overflow")?;
        if end > raw.len() {
            bail!(
                "extent out of range for block {} while reading {}",
                extent.block_id,
                entry.path
            );
        }

        out.extend_from_slice(&raw[begin..end]);
    }

    if out.len() as u64 != entry.size {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(out)
}

fn block_raw_payload(
    reader: &FileReader,
    block: &crushr_core::verify::BlockSpanV1,
    payload_cache: &mut BTreeMap<u32, BlockPayload>,
) -> Result<Vec<u8>> {
    if let Some(payload) = payload_cache.get(&block.block_id) {
        return Ok(payload.raw.clone());
    }

    let header_len = (block.payload_offset - block.header_offset) as usize;
    let mut header_bytes = vec![0u8; header_len];
    read_exact_at(reader, block.header_offset, &mut header_bytes)?;
    let header = read_blk3_header(Cursor::new(&header_bytes)).context("parse BLK3 header")?;

    if header.codec != 1 {
        bail!(
            "unsupported BLK3 codec {} for block {}",
            header.codec,
            block.block_id
        );
    }

    let mut payload = vec![0u8; block.comp_len as usize];
    read_exact_at(reader, block.payload_offset, &mut payload)?;

    let raw = zstd::decode_all(Cursor::new(payload)).context("decompress BLK3 payload")?;
    if raw.len() as u64 != header.raw_len {
        bail!("raw length mismatch for block {}", block.block_id);
    }

    payload_cache.insert(block.block_id, BlockPayload { raw: raw.clone() });
    Ok(raw)
}

fn write_entry(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    if path.exists() && !overwrite {
        bail!("destination exists (use --overwrite): {}", path.display());
    }

    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_exact_at<R: ReadAt>(reader: &R, mut offset: u64, mut dst: &mut [u8]) -> Result<()> {
    while !dst.is_empty() {
        let n = reader.read_at(offset, dst)?;
        if n == 0 {
            bail!("unexpected EOF while reading archive");
        }
        let (_, rest) = dst.split_at_mut(n);
        dst = rest;
        offset += n as u64;
    }
    Ok(())
}

fn main() {
    let opts = match parse_cli_options() {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(1);
        }
    };

    match run(&opts) {
        Ok(report) => {
            if opts.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("serialize extraction report")
                );
            }
            let code = if report.overall_status == "partial_refusal"
                && opts.refusal_exit == RefusalExitPolicy::PartialFailure
            {
                3
            } else {
                0
            };
            std::process::exit(code);
        }
        Err(err) => {
            eprintln!("{err:#}");
            let msg = format!("{err:#}");
            let code = if msg.contains("usage:")
                || msg.contains("unsupported flag")
                || msg.contains("unexpected argument")
                || msg.contains("unsupported value for --refusal-exit")
            {
                1
            } else {
                2
            };

            if opts.json {
                let json_err = ExtractionErrorReport {
                    overall_status: "error",
                    error: msg,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json_err)
                        .expect("serialize extraction error report")
                );
            }
            std::process::exit(code);
        }
    }
}
