use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, shells};

fn normalize_abs(p: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    // Absolute, lexically-normalized path (no filesystem access, does not resolve symlinks).
    let cwd = std::env::current_dir()?;
    let joined = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };

    let mut out = std::path::PathBuf::new();
    for c in joined.components() {
        use std::path::Component;
        match c {
            Component::Prefix(px) => out.push(px.as_os_str()),
            Component::RootDir => out.push(std::path::MAIN_SEPARATOR.to_string()),
            Component::CurDir => {}
            Component::ParentDir => { out.pop(); }
            Component::Normal(s) => out.push(s),
        }
    }
    Ok(out)
}

fn resolve_inputs_abs(inputs: &[std::path::PathBuf]) -> anyhow::Result<Vec<std::path::PathBuf>> {
    inputs.iter().map(|p| normalize_abs(p)).collect()
}

mod cli_ui;
mod dict;
mod extract;
mod format;
mod index_codec;
mod pack;
mod progress;
mod read;
mod recovery;
mod tune;

fn read_paths_from(spec: &Option<String>) -> anyhow::Result<Vec<PathBuf>> {
    use anyhow::Context;
    use std::io::Read as _;
    let Some(s) = spec else { return Ok(Vec::new()); };
    let mut buf = String::new();
    if s == "-" {
        std::io::stdin().read_to_string(&mut buf).context("read stdin")?;
    } else {
        buf = std::fs::read_to_string(s).with_context(|| format!("read {}", s))?;
    }
    let mut out = Vec::new();
    for line in buf.lines() {
        let t = line.trim();
        if t.is_empty() { continue; }
        out.push(PathBuf::from(t));
    }
    Ok(out)
}



fn resolve_base_for_inputs(base: &Option<PathBuf>, inputs: &[PathBuf]) -> Result<PathBuf> {
    if let Some(b) = base.as_ref() {
        return normalize_abs(b.as_path());
    }

    // Infer a base directory. Prefer a directory input; otherwise use the parent of a file input.
    // All inputs are expected to be absolute+normalized already.
    let mut inferred: Option<PathBuf> = None;

    for p in inputs {
        let md = std::fs::symlink_metadata(p).with_context(|| format!("stat {}", p.display()))?;
        let cand = if md.is_dir() {
            p.clone()
        } else {
            p.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf()
        };
        inferred = match inferred.take() {
            None => Some(cand),
            Some(prev) => Some(common_prefix(&prev, &cand)),
        };
    }

    Ok(inferred.unwrap_or_else(|| std::path::PathBuf::from(".")))
}

/// Compute the common path prefix for two *absolute normalized* paths.
fn common_prefix(a: &Path, b: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    let mut ita = a.components();
    let mut itb = b.components();
    loop {
        match (ita.next(), itb.next()) {
            (Some(ca), Some(cb)) if ca == cb => out.push(ca.as_os_str()),
            _ => break,
        }
    }
    if out.as_os_str().is_empty() { PathBuf::from(std::path::MAIN_SEPARATOR.to_string()) } else { out }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Parser, Debug)]
#[command(
    name = "crushr",
    version,
    about = "Solid-block archive compressor (zstd) with random-access extraction",
    long_about = "crushr creates a compact solid-block archive optimized for small size while preserving random-access extraction via an index. It supports append-by-rewriting-tail, symlinks, and optional xattr capture/restore.",
    after_help = r#"Examples:
  crushr pack -o out.crs README.md src/
  crushr pack -o out.crs --files-from <(git ls-files)

  # List-only input (one path per line):
  #   find . -type f | crushr pack -o out.crs --files-from -

"#,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    /// Xattr policy: none|basic|all
    #[arg(long, default_value = "basic")]
    xattr_policy: String,

    /// Disable ANSI colors and UI decorations
    #[arg(long)]
    no_color: bool,

    /// UI verbosity: 0=silent, 1=overall progress, 2=mock multi-progress, 3=mock chart
    #[arg(long, default_value_t = 1)]
    ui: u8,

    /// Decompression cache: number of blocks
    #[arg(long, default_value_t = 64)]
    cache_blocks: usize,

    /// Decompression cache: max MiB
    #[arg(long, default_value_t = 256)]
    cache_mib: usize,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Pack files and/or directories into a new archive
    Pack {
        /// Input paths (files and/or directories)
        #[arg(value_name = "PATH", num_args = 1..)]
        inputs: Vec<PathBuf>,

        /// Optional base directory to make stored paths relative to
        #[arg(long)]
        base: Option<PathBuf>,

        /// Read additional input paths from a file (one path per line). Use '-' for stdin.
        #[arg(long, value_name = "FILE")]
        files_from: Option<String>,

        /// Output archive path
        #[arg(short, long)]
        output: PathBuf,

        /// Use a pre-trained dictionary file (.zdict)
        #[arg(long, value_name = "DICT")]
        dict: Option<PathBuf>,

        /// Autotune (sample-based) block size/level before packing
        #[arg(long)]
        auto: bool,

        /// Autotune time budget in milliseconds
        #[arg(long, default_value_t = 20000)]
        auto_time_ms: u64,

        /// Train and embed default per-content dictionaries (Text and Code)
        #[arg(long)]
        auto_dict: bool,

        /// Dictionary size in KiB for --auto-dict
        #[arg(long, default_value_t = 128)]
        auto_dict_kib: u32,

        /// Max sample files per dictionary family for --auto-dict
        #[arg(long, default_value_t = 500)]
        auto_dict_max_samples: u32,

        /// Max bytes per sample file (KiB) for --auto-dict
        #[arg(long, default_value_t = 256)]
        auto_dict_sample_kib: u32,

        /// Solid block size in MiB (uncompressed)
        #[arg(long, default_value_t = 64)]
        block_mib: u64,

        /// Zstd compression level
        #[arg(long, default_value_t = 15)]
        level: i32,
    },

    /// Append files and/or directories into an existing archive (replaces existing paths)
    Append {
        /// Existing archive path (will be modified)
        archive: PathBuf,

        /// Input paths (files and/or directories) to add
        #[arg(value_name = "PATH", num_args = 1..)]
        inputs: Vec<PathBuf>,

        /// Optional base directory to make stored paths relative to
        #[arg(long)]
        base: Option<PathBuf>,

        /// Read additional input paths from a file (one path per line). Use '-' for stdin.
        #[arg(long, value_name = "FILE")]
        files_from: Option<String>,

        /// Use a pre-trained dictionary file (.zdict). Append preserves existing dict if present.
        #[arg(long, value_name = "DICT")]
        dict: Option<PathBuf>,

        /// Solid block size in MiB (uncompressed) for new data
        #[arg(long, default_value_t = 64)]
        block_mib: u64,

        /// Zstd compression level for new data
        #[arg(long, default_value_t = 15)]
        level: i32,
    },

    /// Extract files from an archive into a directory
    ///
    /// By default, extracts all files (equivalent to --all). Provide one or more PATHs
    /// to extract only specific entries.
    Extract {
        archive: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Extract all entries (default if no PATHs are provided)
        #[arg(long)]
        all: bool,
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Specific paths to extract (paths as stored in the archive)
        #[arg(value_name = "PATH", num_args = 0..)]
        paths: Vec<PathBuf>,
    },


    /// List paths contained in an archive
    List {
        archive: PathBuf,
    },

/// Show basic information and size savings for an archive.
Info {
    /// Archive to inspect.
    archive: PathBuf,
},


    /// Print a single file to stdout
    Cat {
        archive: PathBuf,
        path: PathBuf,
    },

    /// Verify archive integrity (index and optional deep block scan)
    Verify {
        archive: PathBuf,
        #[arg(long)]
        deep: bool,
    },

    /// Attempt tail-based repair using redundant tail frames
    Recover {
        input: PathBuf,
        output: PathBuf,
        #[arg(long, default_value_t = 64 * 1024 * 1024)]
        tail_scan_bytes: u64,
    },

    /// Salvage rebuild: scan embedded EVT frames and rebuild an index even if the tail is unrecoverable
    Salvage {
        input: PathBuf,
        output: PathBuf,
    },

    /// Train a zstd dictionary from the input corpus
    DictTrain {
        #[arg(value_name = "PATH", num_args = 1..)]
        inputs: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value_t = 128)]
        dict_kib: u32,
        #[arg(long, default_value_t = 500)]
        max_samples: u32,
        #[arg(long, default_value_t = 256)]
        sample_kib: u32,
    },

    /// Autotune compression params (block size and level) on a representative sample
    Tune {
        #[arg(value_name = "PATH", num_args = 1..)]
        inputs: Vec<PathBuf>,
        #[arg(long, default_value_t = 20000)]
        time_ms: u64,
    },

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sink = cli_ui::make_sink(cli.ui, cli.no_color);

    match cli.cmd {
        Cmd::Completions { shell } => {
            let mut cmd = Cli::command();
            match shell {
                Shell::Bash => generate(shells::Bash, &mut cmd, "crushr", &mut std::io::stdout()),
                Shell::Zsh => generate(shells::Zsh, &mut cmd, "crushr", &mut std::io::stdout()),
                Shell::Fish => generate(shells::Fish, &mut cmd, "crushr", &mut std::io::stdout()),
            }
            return Ok(());
        }
        Cmd::Pack { mut inputs, base, files_from, output, dict, auto, auto_time_ms, auto_dict, auto_dict_kib, auto_dict_max_samples, auto_dict_sample_kib, block_mib, level } => {
            inputs.extend(read_paths_from(&files_from)?);
            if inputs.is_empty() { bail!("no input paths provided"); }
            let inputs = resolve_inputs_abs(&inputs)?;
            let base = resolve_base_for_inputs(&base, &inputs)?;
            let block_size = block_mib * 1024 * 1024;

            if auto {
                let tuned = tune::autotune(&inputs, &base, auto_time_ms)?;
                let bs = tuned.chosen.block_mib * 1024 * 1024;
                let lvl = tuned.chosen.level;
                if auto_dict {
                    return pack::pack_paths_with_auto_dicts_with_xattrs_progress(
                        &inputs, &base, &output, bs, lvl, sink.clone(),
                        auto_dict_kib, auto_dict_max_samples as usize, (auto_dict_sample_kib as usize) * 1024,
                        &cli.xattr_policy,
                    );
                }
                let dict_bytes = if let Some(p) = dict.as_ref() { Some(crate::dict::read_dict(p)?.dict_bytes) } else { None };
                return pack::pack_paths_with_dict_with_xattrs_progress(
                    &inputs, &base, &output, bs, lvl, sink.clone(), dict_bytes.as_deref(), &cli.xattr_policy
                );
            }

            if auto_dict {
                return pack::pack_paths_with_auto_dicts_with_xattrs_progress(
                    &inputs, &base, &output, block_size, level, sink.clone(),
                    auto_dict_kib, auto_dict_max_samples as usize, (auto_dict_sample_kib as usize) * 1024,
                    &cli.xattr_policy,
                );
            }

            let dict_bytes = if let Some(p) = dict.as_ref() { Some(crate::dict::read_dict(p)?.dict_bytes) } else { None };
            pack::pack_paths_with_dict_with_xattrs_progress(
                &inputs, &base, &output, block_size, level, sink.clone(), dict_bytes.as_deref(), &cli.xattr_policy
            )
        }
        Cmd::Append { archive, mut inputs, base, files_from, dict, block_mib, level } => {
            inputs.extend(read_paths_from(&files_from)?);
            if inputs.is_empty() { bail!("no input paths provided"); }
            let inputs = resolve_inputs_abs(&inputs)?;
            let base = resolve_base_for_inputs(&base, &inputs)?;
            let block_size = block_mib * 1024 * 1024;
            let dict_bytes = if let Some(p) = dict.as_ref() { Some(crate::dict::read_dict(p)?.dict_bytes) } else { None };
            pack::append_paths_with_dict_with_xattrs_progress(
                &archive, &inputs, &base, block_size, level, sink.clone(), dict_bytes.as_deref(), &cli.xattr_policy
            )
        }
        Cmd::Extract { archive, output, all, overwrite, paths } => {
            if all || paths.is_empty() {
                extract::extract_all_progress(&archive, &output, overwrite, sink.clone())
            } else {
                extract::extract_paths_progress(&archive, &output, overwrite, &paths, sink.clone())
            }
        }
        Cmd::List { archive } => {
            let ar = read::ArchiveReader::open_with_cache(&archive, cli.cache_blocks, cli.cache_mib as u64)?;
            for e in ar.index().entries.iter() {
                println!("{}", e.path);
            }
            return Ok(());
        }

        Cmd::Cat { archive, path } => {
            let mut ar = read::ArchiveReader::open_with_cache(&archive, cli.cache_blocks, cli.cache_mib as u64)?;
            let p = path.to_string_lossy().to_string();
            let bytes = ar.read_file(&p)?;
            use std::io::Write;
            std::io::stdout().write_all(&bytes)?;
            Ok(())
        }
        Cmd::Verify { archive, deep } => {
            let mut ar = read::ArchiveReader::open_with_cache(&archive, cli.cache_blocks, cli.cache_mib as u64)?;
            ar.verify_index()?;
            if deep { ar.verify_blocks_deep()?; }
            println!("OK");
            Ok(())
        }
        Cmd::Recover { input, output, tail_scan_bytes } => {
            sink.on_event(progress::ProgressEvent::Start { op: progress::ProgressOp::Recover, phase: progress::ProgressPhase::Other, total_bytes: 0 });
            let r = recovery::repair_archive(&input, &output, tail_scan_bytes);
            sink.on_event(progress::ProgressEvent::Finish { ok: r.is_ok() });
            r
        }
        Cmd::Salvage { input, output } => {
            sink.on_event(progress::ProgressEvent::Start { op: progress::ProgressOp::Salvage, phase: progress::ProgressPhase::Other, total_bytes: 0 });
            let r = recovery::salvage_archive(&input, &output);
            sink.on_event(progress::ProgressEvent::Finish { ok: r.is_ok() });
            r
        }
        Cmd::DictTrain { inputs, output, dict_kib, max_samples, sample_kib } => {
            let inputs = resolve_inputs_abs(&inputs)?;
            let base = resolve_base_for_inputs(&None, &inputs)?;
            let bytes = dict::train_dict_for_paths(&inputs, &base, dict_kib, max_samples as usize, (sample_kib as usize) * 1024)?;
            std::fs::write(&output, &bytes)?;
            Ok(())
        }
        Cmd::Tune { inputs, time_ms } => {
            let inputs = resolve_inputs_abs(&inputs)?;
            let base = resolve_base_for_inputs(&None, &inputs)?;
            let tuned = tune::autotune(&inputs, &base, time_ms)?;
            println!("block_mib={} level={}", tuned.chosen.block_mib, tuned.chosen.level);
            Ok(())
        }
    }
}

Cmd::Info { archive } => {
    let archive_size = std::fs::metadata(&archive)
        .with_context(|| format!("stat {}", archive.display()))?
        .len();

    let ar = read::ArchiveReader::open_with_cache(&archive, cli.cache_blocks, cli.cache_mib)?;
    let idx = ar.index();

    let entries = idx.entries.len() as u64;
    let files = idx
        .entries
        .iter()
        .filter(|e| e.kind == crate::format::EntryKind::Regular)
        .count() as u64;
    let uncompressed: u64 = idx
        .entries
        .iter()
        .filter(|e| e.kind == crate::format::EntryKind::Regular)
        .map(|e| e.size)
        .sum();

    let block_count = ar.block_count() as u64;
    let blocks_raw = ar.blocks_raw_bytes();
    let blocks_frame = ar.blocks_frame_bytes();

    let savings = if uncompressed > 0 {
        1.0 - (archive_size as f64 / uncompressed as f64)
    } else {
        0.0
    };

    println!("Archive: {}", archive.display());
    println!("Size: {} bytes", archive_size);
    println!("Entries: {} (files: {})", entries, files);
    println!("Uncompressed files total: {} bytes", uncompressed);
    println!("Blocks: {}  (payload raw: {} bytes, payload+hdr: {} bytes)", block_count, blocks_raw, blocks_frame);
    println!("Savings vs file bytes: {:.2}%", savings * 100.0);

    return Ok(());
}

