use anyhow::{anyhow, bail, Result};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Resolve an archive entry path under `out_dir`.
///
/// Accepts only confined relative paths composed of normal components.
/// Rejects absolute, prefix/drive, parent traversal, empty, and degenerate forms.
pub fn resolve_confined_path(out_dir: &Path, archive_path: &str) -> Result<PathBuf> {
    let candidate = Path::new(archive_path);
    if archive_path.is_empty() {
        bail!("reject archive path: empty path is not extractable");
    }

    let mut rel = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Prefix(prefix) => bail!(
                "reject archive path '{}': unsupported path prefix {}",
                archive_path,
                prefix.as_os_str().to_string_lossy()
            ),
            Component::RootDir => bail!(
                "reject archive path '{}': absolute paths are not allowed",
                archive_path
            ),
            Component::CurDir => {
                // Skip '.' so `a/./b` still resolves deterministically to `a/b`.
            }
            Component::ParentDir => bail!(
                "reject archive path '{}': parent traversal is not allowed",
                archive_path
            ),
            Component::Normal(part) => {
                if part.is_empty() {
                    bail!("reject archive path '{}': degenerate segment", archive_path);
                }
                rel.push(part);
            }
        }
    }

    if rel.as_os_str().is_empty() {
        bail!(
            "reject archive path '{}': empty normalized path",
            archive_path
        );
    }

    let dest = out_dir.join(&rel);
    ensure_path_within_root(out_dir, &dest, archive_path)?;
    Ok(dest)
}

fn ensure_path_within_root(root: &Path, dest: &Path, archive_path: &str) -> Result<()> {
    let root_abs = lexical_abs(root)?;
    let dest_abs = lexical_abs(dest)?;
    if !dest_abs.starts_with(&root_abs) {
        bail!(
            "reject archive path '{}': extraction destination escapes output root",
            archive_path
        );
    }
    Ok(())
}

fn lexical_abs(path: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Prefix(px) => out.push(px.as_os_str()),
            Component::RootDir => out.push(std::path::MAIN_SEPARATOR.to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(anyhow!("failed lexical normalization"));
                }
            }
            Component::Normal(part) => out.push(part),
        }
    }
    Ok(out)
}

#[cfg(unix)]
pub fn validate_symlink_target(link_target: &str) -> Result<()> {
    let target = Path::new(link_target);
    if link_target.is_empty() {
        bail!("reject symlink target: empty target");
    }
    for component in target.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                bail!(
                    "reject symlink target '{}': only confined relative targets are allowed",
                    link_target
                )
            }
            Component::CurDir => {}
            Component::Normal(part) => {
                if part == OsStr::new("") {
                    bail!(
                        "reject symlink target '{}': degenerate segment",
                        link_target
                    );
                }
            }
        }
    }
    if target.components().next().is_none() {
        bail!(
            "reject symlink target '{}': empty normalized target",
            link_target
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_confined_path, validate_symlink_target};
    use std::path::Path;

    #[test]
    fn relative_safe_path_is_allowed() {
        let out = resolve_confined_path(Path::new("/tmp/out"), "nested/file.txt").unwrap();
        assert_eq!(out, Path::new("/tmp/out/nested/file.txt"));
    }

    #[test]
    fn parent_traversal_is_rejected() {
        let err = resolve_confined_path(Path::new("/tmp/out"), "../../outside.txt").unwrap_err();
        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn absolute_path_is_rejected() {
        let err = resolve_confined_path(Path::new("/tmp/out"), "/tmp/pwned").unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }

    #[test]
    fn normalization_escape_is_rejected() {
        let err =
            resolve_confined_path(Path::new("/tmp/out"), "safe/../../escape.txt").unwrap_err();
        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn accepted_path_remains_under_root() {
        let out = resolve_confined_path(Path::new("/tmp/out"), "a/b/./c.txt").unwrap();
        assert!(out.starts_with(Path::new("/tmp/out")));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_target_rejects_parent_traversal() {
        let err = validate_symlink_target("../outside").unwrap_err();
        assert!(err.to_string().contains("confined relative targets"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_target_rejects_absolute_target() {
        let err = validate_symlink_target("/tmp/pwned").unwrap_err();
        assert!(err.to_string().contains("confined relative targets"));
    }
}
