use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: crushr-lab-salvage <input_dir> --output <experiment_dir> [--export-fragments] [--limit <N>] [--verbose]";
const VERIFICATION_LABEL: &str = "UNVERIFIED_RESEARCH_OUTPUT";

#[derive(Debug)]
struct CliOptions {
    input_dir: PathBuf,
    experiment_dir: PathBuf,
    export_fragments: bool,
    limit: Option<usize>,
    verbose: bool,
}

#[derive(Debug, Clone)]
struct ArchiveRun {
    source_path: PathBuf,
    archive_path: String,
    archive_fingerprint: String,
    archive_id: String,
}

#[derive(Debug, Serialize)]
struct ExperimentManifest {
    experiment_id: String,
    tool_version: &'static str,
    schema_version: &'static str,
    run_count: usize,
    run_timestamp: String,
    verification_label: &'static str,
    archive_list: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RunMetadata {
    archive_path: String,
    archive_fingerprint: String,
    salvage_plan_summary: PlanSummary,
    verified_block_count: u64,
    exported_artifact_count: usize,
    salvageable_file_count: u64,
    unsalvageable_file_count: u64,
    unmappable_file_count: u64,
}

#[derive(Debug, Serialize)]
struct PlanSummary {
    salvageable_files: u64,
    unsalvageable_files: u64,
    unmappable_files: u64,
}

fn parse_cli_options() -> Result<CliOptions> {
    let mut input_dir = None;
    let mut experiment_dir = None;
    let mut export_fragments = false;
    let mut limit = None;
    let mut verbose = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                experiment_dir = Some(PathBuf::from(args.next().context(USAGE)?));
            }
            "--export-fragments" => {
                export_fragments = true;
            }
            "--limit" => {
                let value = args.next().context(USAGE)?;
                limit = Some(
                    value
                        .parse::<usize>()
                        .with_context(|| format!("invalid --limit value: {value}"))?,
                );
            }
            "--verbose" => {
                verbose = true;
            }
            _ if arg.starts_with('-') => bail!("unsupported flag: {arg}"),
            _ if input_dir.is_none() => input_dir = Some(PathBuf::from(arg)),
            _ => bail!("unexpected argument: {arg}"),
        }
    }

    Ok(CliOptions {
        input_dir: input_dir.context(USAGE)?,
        experiment_dir: experiment_dir.context(USAGE)?,
        export_fragments,
        limit,
        verbose,
    })
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn archive_id(archive_path: &str, archive_fingerprint: &str) -> String {
    let digest = blake3::hash(format!("{archive_path}\n{archive_fingerprint}").as_bytes());
    format!(
        "{}-{}",
        sanitize_component(archive_path),
        to_hex(&digest.as_bytes()[..8])
    )
}

fn sanitize_component(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

fn collect_archives(opts: &CliOptions) -> Result<Vec<ArchiveRun>> {
    let mut archives = Vec::new();
    for entry in fs::read_dir(&opts.input_dir)
        .with_context(|| format!("read {}", opts.input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|v| v.to_str()) != Some("crushr") {
            continue;
        }

        let rel = path
            .strip_prefix(&opts.input_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let fingerprint = to_hex(blake3::hash(&bytes).as_bytes());
        archives.push(ArchiveRun {
            source_path: path,
            archive_path: rel.clone(),
            archive_fingerprint: fingerprint.clone(),
            archive_id: archive_id(&rel, &fingerprint),
        });
    }

    archives.sort_by(|a, b| a.archive_path.cmp(&b.archive_path));
    if let Some(limit) = opts.limit {
        archives.truncate(limit);
    }
    Ok(archives)
}

fn count_exported_artifacts(plan: &Value) -> usize {
    let Some(exported) = plan.get("exported_artifacts") else {
        return 0;
    };

    [
        "exported_block_artifacts",
        "exported_fragment_artifacts",
        "exported_complete_file_artifacts",
    ]
    .into_iter()
    .map(|field| {
        exported
            .get(field)
            .and_then(Value::as_array)
            .map_or(0, |entries| entries.len())
    })
    .sum()
}

fn run_salvage(archive: &ArchiveRun, run_dir: &Path, opts: &CliOptions) -> Result<RunMetadata> {
    let plan_path = run_dir.join("salvage_plan.json");
    let export_dir = run_dir.join("exported_artifacts");

    let salvage_bin =
        std::env::var("CRUSHR_SALVAGE_BIN").unwrap_or_else(|_| "crushr-salvage".to_string());
    let mut cmd = Command::new(&salvage_bin);
    cmd.arg(&archive.source_path)
        .arg("--json-out")
        .arg(&plan_path);
    if opts.export_fragments {
        cmd.arg("--export-fragments").arg(&export_dir);
    }

    let output = cmd.output().with_context(|| format!("run {:?}", cmd))?;
    if !output.status.success() {
        bail!(
            "crushr-salvage failed for {}\nstdout:\n{}\nstderr:\n{}",
            archive.source_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let plan: Value = serde_json::from_slice(
        &fs::read(&plan_path).with_context(|| format!("read {}", plan_path.display()))?,
    )
    .with_context(|| format!("parse {}", plan_path.display()))?;

    let summary = plan.get("summary").context("missing summary")?;
    let salvageable = summary
        .get("salvageable_files")
        .and_then(Value::as_u64)
        .context("missing summary.salvageable_files")?;
    let unsalvageable = summary
        .get("unsalvageable_files")
        .and_then(Value::as_u64)
        .context("missing summary.unsalvageable_files")?;
    let unmappable = summary
        .get("unmappable_files")
        .and_then(Value::as_u64)
        .context("missing summary.unmappable_files")?;

    let verified_block_count = plan
        .get("block_candidates")
        .and_then(Value::as_array)
        .map_or(0, |candidates| {
            candidates
                .iter()
                .filter(|candidate| {
                    candidate
                        .get("content_verification_status")
                        .and_then(Value::as_str)
                        == Some("content_verified")
                })
                .count() as u64
        });

    Ok(RunMetadata {
        archive_path: archive.archive_path.clone(),
        archive_fingerprint: archive.archive_fingerprint.clone(),
        salvage_plan_summary: PlanSummary {
            salvageable_files: salvageable,
            unsalvageable_files: unsalvageable,
            unmappable_files: unmappable,
        },
        verified_block_count,
        exported_artifact_count: count_exported_artifacts(&plan),
        salvageable_file_count: salvageable,
        unsalvageable_file_count: unsalvageable,
        unmappable_file_count: unmappable,
    })
}

fn run_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{now}")
}
fn run() -> Result<()> {
    let opts = parse_cli_options()?;
    fs::create_dir_all(&opts.experiment_dir)
        .with_context(|| format!("create {}", opts.experiment_dir.display()))?;

    let archives = collect_archives(&opts)?;
    let runs_root = opts.experiment_dir.join("runs");
    fs::create_dir_all(&runs_root).with_context(|| format!("create {}", runs_root.display()))?;

    let mut archive_ids = Vec::new();

    for archive in &archives {
        if opts.verbose {
            eprintln!("salvage: {}", archive.archive_path);
        }

        let run_dir = runs_root.join(&archive.archive_id);
        fs::create_dir_all(&run_dir).with_context(|| format!("create {}", run_dir.display()))?;

        let metadata = run_salvage(archive, &run_dir, &opts)?;
        fs::write(
            run_dir.join("run_metadata.json"),
            serde_json::to_string_pretty(&metadata)?,
        )
        .with_context(|| format!("write {}", run_dir.join("run_metadata.json").display()))?;
        archive_ids.push(archive.archive_id.clone());
    }

    let experiment_id = {
        let mut hasher = blake3::Hasher::new();
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
        hasher.update(if opts.export_fragments {
            b"export"
        } else {
            b"plan"
        });
        for archive_id in &archive_ids {
            hasher.update(archive_id.as_bytes());
            hasher.update(b"\n");
        }
        to_hex(&hasher.finalize().as_bytes()[..10])
    };

    let manifest = ExperimentManifest {
        experiment_id,
        tool_version: env!("CARGO_PKG_VERSION"),
        schema_version: "crushr-lab-salvage-experiment.v1",
        run_count: archive_ids.len(),
        run_timestamp: run_timestamp(),
        verification_label: VERIFICATION_LABEL,
        archive_list: archive_ids,
    };

    fs::write(
        opts.experiment_dir.join("experiment_manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )
    .with_context(|| {
        format!(
            "write {}",
            opts.experiment_dir
                .join("experiment_manifest.json")
                .display()
        )
    })?;

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}
