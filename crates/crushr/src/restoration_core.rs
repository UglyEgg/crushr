// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::format::{Entry, EntryKind, PreservationProfile};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MetadataClass {
    Ownership,
    Acl,
    Selinux,
    Capability,
    Xattr,
    SpecialFile,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RestorationPolicy {
    Strict,
    Recover,
}

pub(crate) fn metadata_required_by_profile(
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

pub(crate) fn restore_entry_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Result<Vec<MetadataClass>> {
    match entry.kind {
        EntryKind::Regular => restore_regular_metadata(path, entry, profile),
        EntryKind::Directory => restore_directory_metadata(path, entry, profile),
        EntryKind::Symlink => restore_symlink_metadata(path, entry, profile),
        EntryKind::Fifo | EntryKind::CharDevice | EntryKind::BlockDevice => {
            restore_special_metadata(path, entry, profile)
        }
    }
}

pub(crate) fn restore_special_filesystem_object(
    path: &Path,
    entry: &Entry,
    policy: RestorationPolicy,
) -> Result<Vec<MetadataClass>> {
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
            if matches!(policy, RestorationPolicy::Strict) {
                failed.push(MetadataClass::SpecialFile);
            }
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
        if matches!(policy, RestorationPolicy::Strict) {
            failed.push(MetadataClass::SpecialFile);
        }
    }
    Ok(failed)
}

fn restore_regular_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Result<Vec<MetadataClass>> {
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
    if metadata_required_by_profile(profile, entry, MetadataClass::Xattr)
        && restore_xattrs(path, entry)?
    {
        failed.push(MetadataClass::Xattr);
    }
    if metadata_required_by_profile(profile, entry, MetadataClass::Ownership)
        && restore_ownership(path, entry)?
    {
        failed.push(MetadataClass::Ownership);
    }
    let security_failures = restore_security_metadata(path, entry, profile);
    if security_failures.contains(&MetadataClass::Acl) {
        failed.push(MetadataClass::Acl);
    }
    if security_failures.contains(&MetadataClass::Selinux) {
        failed.push(MetadataClass::Selinux);
    }
    if security_failures.contains(&MetadataClass::Capability) {
        failed.push(MetadataClass::Capability);
    }
    failed.sort();
    failed.dedup();
    Ok(failed)
}

fn restore_directory_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Result<Vec<MetadataClass>> {
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
    if metadata_required_by_profile(profile, entry, MetadataClass::Xattr)
        && restore_xattrs(path, entry)?
    {
        failed.push(MetadataClass::Xattr);
    }
    if metadata_required_by_profile(profile, entry, MetadataClass::Ownership)
        && restore_ownership(path, entry)?
    {
        failed.push(MetadataClass::Ownership);
    }
    failed.extend(restore_security_metadata(path, entry, profile));
    failed.sort();
    failed.dedup();
    Ok(failed)
}

fn restore_symlink_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Result<Vec<MetadataClass>> {
    let mut failed = Vec::new();
    if metadata_required_by_profile(profile, entry, MetadataClass::Ownership)
        && restore_ownership(path, entry)?
    {
        failed.push(MetadataClass::Ownership);
    }
    failed.extend(restore_security_metadata(path, entry, profile));
    failed.sort();
    failed.dedup();
    Ok(failed)
}

fn restore_special_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Result<Vec<MetadataClass>> {
    let mut failed = Vec::new();
    if metadata_required_by_profile(profile, entry, MetadataClass::Ownership)
        && restore_ownership(path, entry)?
    {
        failed.push(MetadataClass::Ownership);
    }
    failed.extend(restore_security_metadata(path, entry, profile));
    failed.sort();
    failed.dedup();
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

fn restore_security_metadata(
    path: &Path,
    entry: &Entry,
    profile: PreservationProfile,
) -> Vec<MetadataClass> {
    let mut failed = Vec::new();
    #[cfg(unix)]
    {
        if metadata_required_by_profile(profile, entry, MetadataClass::Acl)
            && restore_single_xattr(
                path,
                "acl-restore",
                "system.posix_acl_access",
                entry.acl_access.as_deref(),
            )
        {
            failed.push(MetadataClass::Acl);
        }
        if metadata_required_by_profile(profile, entry, MetadataClass::Acl)
            && restore_single_xattr(
                path,
                "acl-restore",
                "system.posix_acl_default",
                entry.acl_default.as_deref(),
            )
        {
            failed.push(MetadataClass::Acl);
        }
        if metadata_required_by_profile(profile, entry, MetadataClass::Selinux)
            && restore_single_xattr(
                path,
                "selinux-restore",
                "security.selinux",
                entry.selinux_label.as_deref(),
            )
        {
            failed.push(MetadataClass::Selinux);
        }
        if metadata_required_by_profile(profile, entry, MetadataClass::Capability)
            && restore_single_xattr(
                path,
                "capability-restore",
                "security.capability",
                entry.linux_capability.as_deref(),
            )
        {
            failed.push(MetadataClass::Capability);
        }
    }
    #[cfg(not(unix))]
    {
        if metadata_required_by_profile(profile, entry, MetadataClass::Acl)
            && (entry.acl_access.is_some() || entry.acl_default.is_some())
        {
            failed.push(MetadataClass::Acl);
            eprintln!(
                "WARNING[acl-restore]: skipped ACL metadata on '{}' (unsupported platform)",
                path.display()
            );
        }
        if metadata_required_by_profile(profile, entry, MetadataClass::Selinux)
            && entry.selinux_label.is_some()
        {
            failed.push(MetadataClass::Selinux);
            eprintln!(
                "WARNING[selinux-restore]: skipped SELinux label on '{}' (unsupported platform)",
                path.display()
            );
        }
        if metadata_required_by_profile(profile, entry, MetadataClass::Capability)
            && entry.linux_capability.is_some()
        {
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
