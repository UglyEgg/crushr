use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::extraction_path::{resolve_confined_path, validate_symlink_target};
use crate::format::EntryKind;
use crate::read::ArchiveReader;

#[allow(dead_code)]
pub fn extract_all(archive: &Path, out_dir: &Path, overwrite: bool) -> Result<()> {
    let sink: crate::progress::SharedSink = std::sync::Arc::new(crate::progress::NullProgressSink);
    extract_all_progress(archive, out_dir, overwrite, sink)
}

pub fn extract_all_progress(
    archive: &Path,
    out_dir: &Path,
    overwrite: bool,
    sink: crate::progress::SharedSink,
) -> Result<()> {
    sink.on_event(crate::progress::ProgressEvent::Start {
        op: crate::progress::ProgressOp::Extract,
        phase: crate::progress::ProgressPhase::Decompress,
        total_bytes: 0,
    });
    let mut ar = ArchiveReader::open(archive)?;
    fs::create_dir_all(out_dir)?;
    let idx = ar.index().clone();

    let total: u64 = idx.entries.iter().map(|e| e.size).sum();
    sink.on_event(crate::progress::ProgressEvent::Phase {
        phase: crate::progress::ProgressPhase::WriteFiles,
        total_bytes: Some(total),
    });

    for e in idx.entries {
        let dest = resolve_confined_path(out_dir, &e.path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        match e.kind {
            EntryKind::Symlink => {
                let target = e.link_target.clone().unwrap_or_default();
                validate_symlink_target(&target)?;
                anyhow::bail!(
                    "symlink extraction is disabled in hardened mode: {} -> {}",
                    e.path,
                    target
                );
            }
            EntryKind::Regular => {
                if dest.exists() && !overwrite {
                    continue;
                }
                let bytes = ar.read_entry_bytes(&e)?;
                fs::write(&dest, &bytes).with_context(|| format!("write {}", dest.display()))?;
                sink.on_event(crate::progress::ProgressEvent::AdvanceBytes {
                    bytes: bytes.len() as u64,
                });
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perm = fs::Permissions::from_mode(e.mode);
                    fs::set_permissions(&dest, perm).ok();
                }
            }
        }
    }
    sink.on_event(crate::progress::ProgressEvent::Finish { ok: true });
    Ok(())
}

pub fn extract_paths_progress(
    archive: &Path,
    out_dir: &Path,
    overwrite: bool,
    paths: &[std::path::PathBuf],
    sink: crate::progress::SharedSink,
) -> Result<()> {
    use std::collections::HashSet;

    sink.on_event(crate::progress::ProgressEvent::Start {
        op: crate::progress::ProgressOp::Extract,
        phase: crate::progress::ProgressPhase::Decompress,
        total_bytes: 0,
    });

    let mut ar = ArchiveReader::open(archive)?;
    fs::create_dir_all(out_dir)?;

    let idx = ar.index().clone();
    let wanted: HashSet<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let total: u64 = idx
        .entries
        .iter()
        .filter(|e| wanted.contains(&e.path))
        .map(|e| e.size)
        .sum();

    sink.on_event(crate::progress::ProgressEvent::Phase {
        phase: crate::progress::ProgressPhase::WriteFiles,
        total_bytes: Some(total),
    });

    for rel in wanted.iter() {
        let dest = resolve_confined_path(out_dir, rel)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if dest.exists() && !overwrite {
            anyhow::bail!("destination exists (use --overwrite): {}", dest.display());
        }
        ar.extract_to(rel, &dest, "basic")?;
        sink.on_event(crate::progress::ProgressEvent::AdvanceBytes { bytes: 0 });
    }

    sink.on_event(crate::progress::ProgressEvent::Finish { ok: true });
    Ok(())
}
