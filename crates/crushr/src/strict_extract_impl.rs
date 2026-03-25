// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::extraction_path::resolve_confined_path;
use crate::format::{Entry, EntryKind, PreservationProfile};
use crate::index_codec::decode_index;
use anyhow::{Context, Result, bail};
use crushr_core::{
    extraction::{ExtractionOutcomeKind, build_extraction_report, classify_refusal_paths},
    io::{Len, ReadAt},
    open::open_archive_v1,
    verify::{scan_blocks_v1, verify_block_payloads_v1},
};
use crushr_format::blk3::read_blk3_header;
use std::collections::{BTreeMap, BTreeSet};
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
pub struct StrictExtractOptions {
    pub archive: PathBuf,
    pub out_dir: PathBuf,
    pub overwrite: bool,
    pub selected_paths: Option<Vec<String>>,
    pub verify_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MetadataClass {
    Ownership,
    Acl,
    Selinux,
    Capability,
    Xattr,
    SpecialFile,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct StrictExtractRun {
    pub outcome_kind: ExtractionOutcomeKind,
    pub report: crushr_core::extraction::ExtractionReport,
}

pub fn run_strict_extract(opts: &StrictExtractOptions) -> Result<StrictExtractRun> {
    let reader = FileReader {
        file: File::open(&opts.archive)
            .with_context(|| format!("open {}", opts.archive.display()))?,
    };

    let opened = open_archive_v1(&reader)?;
    let blocks = scan_blocks_v1(&reader, opened.tail.footer.blocks_end_offset)?;
    let index = decode_index(&opened.tail.idx3_bytes).context("decode IDX3")?;
    let preservation_profile = index.preservation_profile;
    let corrupted = verify_block_payloads_v1(&reader, opened.tail.footer.blocks_end_offset)?;

    if !opts.verify_only {
        fs::create_dir_all(&opts.out_dir)
            .with_context(|| format!("create {}", opts.out_dir.display()))?;
    }

    let mut entries: Vec<Entry> = index.entries;
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let selected = opts
        .selected_paths
        .as_ref()
        .map(|paths| paths.iter().cloned().collect::<BTreeSet<_>>());

    let mut required_blocks_by_path = BTreeMap::<String, Vec<u32>>::new();
    for entry in &entries {
        if let Some(selected_paths) = &selected
            && !selected_paths.contains(&entry.path)
        {
            continue;
        }

        if entry.kind == EntryKind::Regular {
            required_blocks_by_path.insert(
                entry.path.clone(),
                entry.extents.iter().map(|extent| extent.block_id).collect(),
            );
        }
    }

    let candidate_paths = required_blocks_by_path.keys().cloned().collect::<Vec<_>>();
    let (safe_files, refused_files) = classify_refusal_paths(candidate_paths, &corrupted, |path| {
        required_blocks_by_path
            .get(path)
            .cloned()
            .unwrap_or_default()
    });

    let safe_paths = safe_files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut hardlink_roots = BTreeMap::<u64, PathBuf>::new();
    let mut metadata_failures: Vec<(String, Vec<MetadataClass>)> = Vec::new();

    for entry in entries {
        if entry.kind == EntryKind::Regular && !safe_paths.contains(entry.path.as_str()) {
            continue;
        }

        if opts.verify_only {
            validate_entry_bytes_strict(&reader, &entry, &blocks)?;
        } else {
            let destination = resolve_confined_path(&opts.out_dir, &entry.path)?;
            write_entry(
                &reader,
                &entry,
                destination.as_path(),
                &blocks,
                opts.overwrite,
                &mut hardlink_roots,
            )
            .map(|failed| {
                let failed = failed
                    .into_iter()
                    .filter(|class| {
                        metadata_required_by_profile(preservation_profile, &entry, *class)
                    })
                    .collect::<Vec<_>>();
                if !failed.is_empty() {
                    metadata_failures.push((entry.path.clone(), failed));
                }
            })?;
        }
    }

    if !metadata_failures.is_empty() {
        metadata_failures.sort_by(|a, b| a.0.cmp(&b.0));
        let (path, classes) = &metadata_failures[0];
        let class_list = classes
            .iter()
            .map(|class| format!("{class:?}").to_lowercase())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "strict extraction refused: metadata restoration failed for {} entries (first: '{}' failed [{}])",
            metadata_failures.len(),
            path,
            class_list
        );
    }

    let (outcome_kind, report) = build_extraction_report(safe_files, refused_files);

    Ok(StrictExtractRun {
        outcome_kind,
        report,
    })
}

fn metadata_required_by_profile(
    profile: PreservationProfile,
    entry: &Entry,
    class: MetadataClass,
) -> bool {
    match profile {
        PreservationProfile::Full => true,
        PreservationProfile::Basic => match class {
            MetadataClass::SpecialFile => !matches!(
                entry.kind,
                EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice
            ),
            MetadataClass::Xattr
            | MetadataClass::Ownership
            | MetadataClass::Acl
            | MetadataClass::Selinux
            | MetadataClass::Capability => false,
        },
        PreservationProfile::PayloadOnly => false,
    }
}

fn read_entry_bytes_strict(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[crushr_core::verify::BlockSpanV1],
) -> Result<Vec<u8>> {
    let mut out = vec![0u8; entry.size as usize];
    let mut write_cursor = 0u64;

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;

        let raw = block_raw_payload(reader, block)?;

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

        let target_off = if entry.sparse {
            extent.logical_offset as usize
        } else {
            write_cursor as usize
        };
        let target_end = target_off
            .checked_add(extent.len as usize)
            .context("target extent overflow")?;
        if target_end > out.len() {
            bail!("entry size mismatch while reading {}", entry.path);
        }
        out[target_off..target_end].copy_from_slice(&raw[begin..end]);
        write_cursor = write_cursor
            .checked_add(extent.len)
            .context("entry size overflow while reading")?;
    }

    if !entry.sparse && write_cursor != entry.size {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(out)
}

fn validate_entry_bytes_strict(
    reader: &FileReader,
    entry: &Entry,
    blocks: &[crushr_core::verify::BlockSpanV1],
) -> Result<()> {
    let mut total = 0u64;
    let mut max_end = 0u64;

    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;

        let raw = block_raw_payload(reader, block)?;
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
        total = total
            .checked_add(extent.len)
            .context("entry size overflow while validating extents")?;
        let logical_end = extent
            .logical_offset
            .checked_add(extent.len)
            .context("logical extent overflow while validating extents")?;
        max_end = max_end.max(logical_end);
    }

    if (!entry.sparse && total != entry.size) || (entry.sparse && max_end > entry.size) {
        bail!("entry size mismatch while reading {}", entry.path);
    }

    Ok(())
}

fn block_raw_payload(
    reader: &FileReader,
    block: &crushr_core::verify::BlockSpanV1,
) -> Result<Vec<u8>> {
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

    Ok(raw)
}

fn write_entry(
    reader: &FileReader,
    entry: &Entry,
    path: &Path,
    blocks: &[crushr_core::verify::BlockSpanV1],
    overwrite: bool,
    hardlink_roots: &mut BTreeMap<u64, PathBuf>,
) -> Result<Vec<MetadataClass>> {
    match entry.kind {
        EntryKind::Directory => {
            fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
            let mut failed = Vec::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if fs::set_permissions(path, fs::Permissions::from_mode(entry.mode)).is_err() {
                    failed.push(MetadataClass::SpecialFile);
                }
            }
            if restore_mtime(path, entry.mtime).is_err() {
                failed.push(MetadataClass::SpecialFile);
            }
            if restore_xattrs(path, entry)? {
                failed.push(MetadataClass::Xattr);
            }
            if restore_ownership(path, entry)? {
                failed.push(MetadataClass::Ownership);
            }
            failed.extend(restore_security_metadata(path, entry));
            failed.sort();
            failed.dedup();
            Ok(failed)
        }
        EntryKind::Symlink => {
            if path.exists() {
                if overwrite {
                    fs::remove_file(path)
                        .or_else(|_| fs::remove_dir_all(path))
                        .ok();
                } else {
                    bail!("destination exists (use --overwrite): {}", path.display());
                }
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            let target = entry.link_target.clone().unwrap_or_default();
            crate::extraction_path::validate_symlink_target(&target)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, path)
                .with_context(|| format!("symlink {} -> {}", path.display(), target))?;
            #[cfg(not(unix))]
            bail!("symlink extraction is unsupported on this platform");
            let failed = {
                let mut failed = Vec::new();
                if restore_ownership(path, entry)? {
                    failed.push(MetadataClass::Ownership);
                }
                failed.extend(restore_security_metadata(path, entry));
                failed
            };
            Ok(failed)
        }
        EntryKind::Regular => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            if let Some(group_id) = entry.hardlink_group_id {
                if let Some(root_path) = hardlink_roots.get(&group_id) {
                    if path.exists() {
                        if overwrite {
                            fs::remove_file(path)
                                .or_else(|_| fs::remove_dir_all(path))
                                .ok();
                        } else {
                            bail!("destination exists (use --overwrite): {}", path.display());
                        }
                    }
                    fs::hard_link(root_path, path).with_context(|| {
                        format!("hardlink {} -> {}", path.display(), root_path.display())
                    })?;
                } else {
                    if entry.sparse {
                        write_sparse_entry(reader, entry, path, blocks, overwrite)?;
                    } else {
                        let bytes = read_entry_bytes_strict(reader, entry, blocks)?;
                        if path.exists() && !overwrite {
                            bail!("destination exists (use --overwrite): {}", path.display());
                        }
                        fs::write(path, &bytes)
                            .with_context(|| format!("write {}", path.display()))?;
                    }
                    hardlink_roots.insert(group_id, path.to_path_buf());
                }
            } else if entry.sparse {
                write_sparse_entry(reader, entry, path, blocks, overwrite)?;
            } else {
                let bytes = read_entry_bytes_strict(reader, entry, blocks)?;
                if path.exists() && !overwrite {
                    bail!("destination exists (use --overwrite): {}", path.display());
                }
                fs::write(path, &bytes).with_context(|| format!("write {}", path.display()))?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(entry.mode)).ok();
            }
            let mut failed = Vec::new();
            if restore_xattrs(path, entry)? {
                failed.push(MetadataClass::Xattr);
            }
            if restore_ownership(path, entry)? {
                failed.push(MetadataClass::Ownership);
            }
            failed.extend(restore_security_metadata(path, entry));
            if restore_mtime(path, entry.mtime).is_err() {
                failed.push(MetadataClass::SpecialFile);
            }
            failed.sort();
            failed.dedup();
            Ok(failed)
        }
        EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            if path.exists() {
                if overwrite {
                    fs::remove_file(path)
                        .or_else(|_| fs::remove_dir_all(path))
                        .ok();
                } else {
                    bail!("destination exists (use --overwrite): {}", path.display());
                }
            }
            let mut failed = Vec::new();
            failed.extend(restore_special(path, entry)?);
            if restore_ownership(path, entry)? {
                failed.push(MetadataClass::Ownership);
            }
            failed.extend(restore_security_metadata(path, entry));
            failed.sort();
            failed.dedup();
            Ok(failed)
        }
    }
}

fn write_sparse_entry(
    reader: &FileReader,
    entry: &Entry,
    path: &Path,
    blocks: &[crushr_core::verify::BlockSpanV1],
    overwrite: bool,
) -> Result<()> {
    use std::os::unix::fs::FileExt;
    if path.exists() && !overwrite {
        bail!("destination exists (use --overwrite): {}", path.display());
    }
    let out = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    out.set_len(entry.size)
        .with_context(|| format!("set_len {}", path.display()))?;
    for extent in &entry.extents {
        let block = blocks
            .get(extent.block_id as usize)
            .with_context(|| format!("extent references missing block {}", extent.block_id))?;
        let raw = block_raw_payload(reader, block)?;
        let begin = extent.offset as usize;
        let end = begin
            .checked_add(extent.len as usize)
            .context("extent length overflow")?;
        if end > raw.len() {
            bail!("extent out of range for sparse write {}", entry.path);
        }
        out.write_at(&raw[begin..end], extent.logical_offset)
            .with_context(|| format!("write sparse extent {}", path.display()))?;
    }
    Ok(())
}

fn restore_special(path: &Path, entry: &Entry) -> Result<Vec<MetadataClass>> {
    let mut failed = Vec::new();
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn mkfifo(pathname: *const std::os::raw::c_char, mode: u32) -> std::os::raw::c_int;
            fn mknod(
                pathname: *const std::os::raw::c_char,
                mode: u32,
                dev: u64,
            ) -> std::os::raw::c_int;
        }
        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for special restore: {}", path.display()))?;
        let rc = match entry.kind {
            EntryKind::Fifo => unsafe { mkfifo(c_path.as_ptr(), entry.mode) },
            EntryKind::CharDevice | EntryKind::BlockDevice => {
                let mode = entry.mode
                    | if entry.kind == EntryKind::CharDevice {
                        0o020000
                    } else {
                        0o060000
                    };
                let major = entry.device_major.unwrap_or(0) as u64;
                let minor = entry.device_minor.unwrap_or(0) as u64;
                let dev = ((major & 0xfffff000) << 32)
                    | ((major & 0xfff) << 8)
                    | ((minor & 0xffffff00) << 12)
                    | (minor & 0xff);
                unsafe { mknod(c_path.as_ptr(), mode, dev) }
            }
            _ => 0,
        };
        if rc != 0 {
            eprintln!(
                "WARNING[special-restore]: could not restore '{}' at '{}': {}",
                entry.path,
                path.display(),
                std::io::Error::last_os_error()
            );
            failed.push(MetadataClass::SpecialFile);
        } else {
            if restore_mtime(path, entry.mtime).is_err() {
                failed.push(MetadataClass::SpecialFile);
            }
            if restore_xattrs(path, entry)? {
                failed.push(MetadataClass::Xattr);
            }
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!(
            "WARNING[special-restore]: skipped '{}' at '{}' (unsupported platform)",
            entry.path,
            path.display()
        );
        failed.push(MetadataClass::SpecialFile);
    }
    Ok(failed)
}

fn restore_ownership(path: &Path, entry: &Entry) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn lchown(
                path: *const std::os::raw::c_char,
                owner: u32,
                group: u32,
            ) -> std::os::raw::c_int;
        }
        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for ownership restore: {}", path.display()))?;
        let rc = unsafe { lchown(c_path.as_ptr(), entry.uid, entry.gid) };
        if rc != 0 {
            let label = entry
                .uname
                .as_ref()
                .zip(entry.gname.as_ref())
                .map(|(u, g)| format!("{u}:{g}"))
                .unwrap_or_else(|| format!("{}:{}", entry.uid, entry.gid));
            eprintln!(
                "WARNING[ownership-restore]: could not restore '{}' on '{}': {}",
                label,
                path.display(),
                std::io::Error::last_os_error()
            );
            return Ok(true);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, entry);
    }
    Ok(false)
}

fn restore_mtime(path: &Path, mtime_secs: i64) -> Result<()> {
    #[cfg(unix)]
    {
        if mtime_secs < 0 {
            return Ok(());
        }
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::io::RawFd;

        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }

        unsafe extern "C" {
            fn utimensat(
                dirfd: RawFd,
                pathname: *const std::os::raw::c_char,
                times: *const Timespec,
                flags: std::os::raw::c_int,
            ) -> std::os::raw::c_int;
        }

        const AT_FDCWD: RawFd = -100;
        const UTIME_OMIT: i64 = 1_073_741_822;

        let c_path = CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("invalid path for mtime restore: {}", path.display()))?;
        let times = [
            Timespec {
                tv_sec: 0,
                tv_nsec: UTIME_OMIT,
            },
            Timespec {
                tv_sec: mtime_secs,
                tv_nsec: 0,
            },
        ];
        let rc = unsafe { utimensat(AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("set mtime {}", path.display()));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mtime_secs);
    }
    Ok(())
}

fn restore_xattrs(path: &Path, entry: &Entry) -> Result<bool> {
    let mut failed = false;
    #[cfg(unix)]
    {
        for xa in &entry.xattrs {
            if let Err(err) = xattr::set(path, &xa.name, &xa.value) {
                failed = true;
                eprintln!(
                    "WARNING[xattr-restore]: could not restore '{}' on '{}': {err}",
                    xa.name,
                    path.display()
                );
            }
        }
    }
    #[cfg(not(unix))]
    {
        if !entry.xattrs.is_empty() {
            failed = true;
            eprintln!(
                "WARNING[xattr-restore]: skipped {} xattrs on '{}' (unsupported platform)",
                entry.xattrs.len(),
                path.display()
            );
        }
    }
    Ok(failed)
}

fn restore_security_metadata(path: &Path, entry: &Entry) -> Vec<MetadataClass> {
    let mut failed = Vec::new();
    #[cfg(unix)]
    {
        if restore_single_xattr(
            path,
            "acl-restore",
            "system.posix_acl_access",
            entry.acl_access.as_deref(),
        ) {
            failed.push(MetadataClass::Acl);
        }
        if restore_single_xattr(
            path,
            "acl-restore",
            "system.posix_acl_default",
            entry.acl_default.as_deref(),
        ) {
            failed.push(MetadataClass::Acl);
        }
        if restore_single_xattr(
            path,
            "selinux-restore",
            "security.selinux",
            entry.selinux_label.as_deref(),
        ) {
            failed.push(MetadataClass::Selinux);
        }
        if restore_single_xattr(
            path,
            "capability-restore",
            "security.capability",
            entry.linux_capability.as_deref(),
        ) {
            failed.push(MetadataClass::Capability);
        }
    }
    #[cfg(not(unix))]
    {
        if entry.acl_access.is_some() || entry.acl_default.is_some() {
            failed.push(MetadataClass::Acl);
            eprintln!(
                "WARNING[acl-restore]: skipped ACL metadata on '{}' (unsupported platform)",
                path.display()
            );
        }
        if entry.selinux_label.is_some() {
            failed.push(MetadataClass::Selinux);
            eprintln!(
                "WARNING[selinux-restore]: skipped SELinux label on '{}' (unsupported platform)",
                path.display()
            );
        }
        if entry.linux_capability.is_some() {
            failed.push(MetadataClass::Capability);
            eprintln!(
                "WARNING[capability-restore]: skipped Linux capabilities on '{}' (unsupported platform)",
                path.display()
            );
        }
    }
    failed
}

#[cfg(unix)]
fn restore_single_xattr(path: &Path, warning_code: &str, name: &str, value: Option<&[u8]>) -> bool {
    if let Some(value) = value
        && let Err(err) = xattr::set(path, name, value)
    {
        eprintln!(
            "WARNING[{warning_code}]: could not restore '{name}' on '{}': {err}",
            path.display()
        );
        return true;
    }
    false
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
