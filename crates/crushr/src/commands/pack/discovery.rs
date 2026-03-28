// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use super::*;

pub(super) fn collect_files(
    inputs: &[PathBuf],
    requirements: MetadataCaptureRequirements,
) -> Result<Vec<InputFile>> {
    collect_files_impl(inputs, requirements)
}

pub(super) fn metadata_capture_requirements(
    profile: PreservationProfile,
) -> MetadataCaptureRequirements {
    metadata_capture_requirements_for_profile(profile)
}

pub(super) fn plan_pack_profile(
    candidates: Vec<InputFile>,
    profile: PreservationProfile,
) -> PackProfilePlan {
    plan_pack_profile_impl(candidates, profile)
}

pub(super) fn emit_profile_warnings(omissions: &[ProfileOmission]) {
    emit_profile_warnings_impl(omissions);
}

pub(super) fn reject_duplicate_logical_paths(files: &[InputFile]) -> Result<()> {
    reject_duplicate_logical_paths_impl(files)
}

#[derive(Debug, Default, Clone)]
struct CapturedSecurityMetadata {
    acl_access: Option<Vec<u8>>,
    acl_default: Option<Vec<u8>>,
    selinux_label: Option<Vec<u8>>,
    linux_capability: Option<Vec<u8>>,
}

fn capture_xattrs(path: &Path) -> (Vec<Xattr>, CapturedSecurityMetadata) {
    #[cfg(unix)]
    {
        let mut out = Vec::new();
        let mut security = CapturedSecurityMetadata::default();
        if let Ok(names) = xattr::list(path) {
            for name_os in names {
                let name = name_os.to_string_lossy().to_string();
                if let Ok(Some(value)) = xattr::get(path, &name_os) {
                    match name.as_str() {
                        POSIX_ACL_ACCESS_XATTR => security.acl_access = Some(value),
                        POSIX_ACL_DEFAULT_XATTR => security.acl_default = Some(value),
                        SELINUX_LABEL_XATTR => security.selinux_label = Some(value),
                        LINUX_CAPABILITY_XATTR => security.linux_capability = Some(value),
                        _ => out.push(Xattr { name, value }),
                    }
                }
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        (out, security)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        (Vec::new(), CapturedSecurityMetadata::default())
    }
}

pub(super) struct CapturedMeta {
    pub(super) mode: u32,
    pub(super) mtime: i64,
    pub(super) uid: u32,
    pub(super) gid: u32,
    pub(super) hardlink_key: Option<(u64, u64)>,
    pub(super) device_major: Option<u32>,
    pub(super) device_minor: Option<u32>,
}

pub(super) fn capture_mode_mtime_uid_gid(meta: &std::fs::Metadata) -> CapturedMeta {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let hardlink_key = if meta.is_file() && meta.nlink() > 1 {
            Some((meta.dev(), meta.ino()))
        } else {
            None
        };
        let file_kind = meta.mode() & libc_s_ifmt();
        let (device_major, device_minor) =
            if file_kind == libc_s_ifchr() || file_kind == libc_s_ifblk() {
                let rdev = meta.rdev();
                (Some(libc_major(rdev) as u32), Some(libc_minor(rdev) as u32))
            } else {
                (None, None)
            };
        CapturedMeta {
            mode: meta.mode() & 0o7777,
            mtime: meta.mtime(),
            uid: meta.uid(),
            gid: meta.gid(),
            hardlink_key,
            device_major,
            device_minor,
        }
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        CapturedMeta {
            mode: 0,
            mtime: 0,
            uid: 0,
            gid: 0,
            hardlink_key: None,
            device_major: None,
            device_minor: None,
        }
    }
}

#[cfg(unix)]
fn capture_ownership_names(uid: u32, gid: u32) -> (Option<String>, Option<String>) {
    use std::ffi::CStr;
    #[repr(C)]
    struct Passwd {
        pw_name: *const std::os::raw::c_char,
        pw_passwd: *const std::os::raw::c_char,
        pw_uid: u32,
        pw_gid: u32,
        pw_gecos: *const std::os::raw::c_char,
        pw_dir: *const std::os::raw::c_char,
        pw_shell: *const std::os::raw::c_char,
    }
    #[repr(C)]
    struct Group {
        gr_name: *const std::os::raw::c_char,
        gr_passwd: *const std::os::raw::c_char,
        gr_gid: u32,
        gr_mem: *mut *mut std::os::raw::c_char,
    }
    unsafe extern "C" {
        fn getpwuid(uid: u32) -> *mut Passwd;
        fn getgrgid(gid: u32) -> *mut Group;
    }

    let uname = unsafe {
        let ptr = getpwuid(uid);
        if ptr.is_null() || (*ptr).pw_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*ptr).pw_name).to_string_lossy().to_string())
        }
    };
    let gname = unsafe {
        let ptr = getgrgid(gid);
        if ptr.is_null() || (*ptr).gr_name.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*ptr).gr_name).to_string_lossy().to_string())
        }
    };
    (uname, gname)
}

#[cfg(not(unix))]
fn capture_ownership_names(_uid: u32, _gid: u32) -> (Option<String>, Option<String>) {
    (None, None)
}

#[cfg(unix)]
fn capture_sparse_chunks(path: &Path, size: u64) -> Vec<SparseChunk> {
    use std::os::unix::fs::FileExt;
    use std::os::unix::io::AsRawFd;
    if size == 0 {
        return Vec::new();
    }
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let mut chunks = Vec::new();
    let mut cursor = 0u64;
    while cursor < size {
        let Some(data_off) = sparse_lseek(file.as_raw_fd(), cursor, libc_seek_data()) else {
            break;
        };
        if data_off >= size {
            break;
        }
        let hole_off = sparse_lseek(file.as_raw_fd(), data_off, libc_seek_hole()).unwrap_or(size);
        let end = hole_off.min(size);
        if end <= data_off {
            break;
        }
        chunks.push(SparseChunk {
            logical_offset: data_off,
            len: end - data_off,
        });
        cursor = end;
    }
    if chunks.is_empty() {
        let mut probe = [0u8; 1];
        let has_data = file.read_at(&mut probe, 0).ok().unwrap_or(0) > 0;
        if has_data {
            vec![SparseChunk {
                logical_offset: 0,
                len: size,
            }]
        } else {
            Vec::new()
        }
    } else {
        chunks
    }
}

#[cfg(not(unix))]
fn capture_sparse_chunks(_path: &Path, _size: u64) -> Vec<SparseChunk> {
    Vec::new()
}

fn collect_files_impl(
    inputs: &[PathBuf],
    requirements: MetadataCaptureRequirements,
) -> Result<Vec<InputFile>> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().context("read current working directory")?;
    let mut uname_by_uid = BTreeMap::<u32, Option<String>>::new();
    let mut gname_by_gid = BTreeMap::<u32, Option<String>>::new();

    for input in inputs {
        let abs = if input.is_absolute() {
            input.clone()
        } else {
            cwd.join(input)
        };
        let meta =
            std::fs::symlink_metadata(&abs).with_context(|| format!("stat {}", input.display()))?;

        if meta.is_file() {
            let name = abs
                .file_name()
                .context("input file has no file name")?
                .to_string_lossy()
                .to_string();
            let size = meta.len();
            let captured = capture_mode_mtime_uid_gid(&meta);
            let mode = captured.mode;
            let mtime = captured.mtime;
            let (uid, gid, uname, gname) = if requirements.capture_ownership_names {
                let uname = uname_by_uid.entry(captured.uid).or_insert_with(|| {
                    let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                    uname
                });
                let gname = gname_by_gid.entry(captured.gid).or_insert_with(|| {
                    let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                    gname
                });
                (captured.uid, captured.gid, uname.clone(), gname.clone())
            } else {
                (captured.uid, captured.gid, None, None)
            };
            let (xattrs, security) = if requirements.capture_xattrs {
                capture_xattrs(input)
            } else {
                (Vec::new(), CapturedSecurityMetadata::default())
            };
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: abs,
                raw_len: size,
                kind: EntryKind::Regular,
                mode,
                mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: captured.hardlink_key,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: if requirements.capture_sparse_layout {
                    capture_sparse_chunks(input, size)
                } else {
                    Vec::new()
                },
                device_major: None,
                device_minor: None,
            });
            continue;
        }

        if meta.file_type().is_symlink() {
            let name = abs
                .file_name()
                .context("input symlink has no file name")?
                .to_string_lossy()
                .to_string();
            let captured = capture_mode_mtime_uid_gid(&meta);
            let (uid, gid, uname, gname) = if requirements.capture_ownership_names {
                let (uname, gname) = capture_ownership_names(captured.uid, captured.gid);
                (captured.uid, captured.gid, uname, gname)
            } else {
                (captured.uid, captured.gid, None, None)
            };
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: input.clone(),
                raw_len: 0,
                kind: EntryKind::Symlink,
                mode: captured.mode,
                mtime: captured.mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: None,
                xattrs: Vec::new(),
                acl_access: None,
                acl_default: None,
                selinux_label: None,
                linux_capability: None,
                sparse_chunks: Vec::new(),
                device_major: None,
                device_minor: None,
            });
            continue;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let captured = capture_mode_mtime_uid_gid(&meta);
            let ft = meta.file_type();
            let kind = if ft.is_fifo() {
                Some(EntryKind::Fifo)
            } else if ft.is_char_device() {
                Some(EntryKind::CharDevice)
            } else if ft.is_block_device() {
                Some(EntryKind::BlockDevice)
            } else {
                None
            };
            if let Some(kind) = kind {
                let name = abs
                    .file_name()
                    .context("input special file has no file name")?
                    .to_string_lossy()
                    .to_string();
                let (uname, gname) = if requirements.capture_ownership_names {
                    capture_ownership_names(captured.uid, captured.gid)
                } else {
                    (None, None)
                };
                files.push(InputFile {
                    rel_path: normalize_logical_path(&name),
                    abs_path: abs,
                    raw_len: 0,
                    kind,
                    mode: captured.mode,
                    mtime: captured.mtime,
                    uid: captured.uid,
                    gid: captured.gid,
                    uname,
                    gname,
                    hardlink_key: captured.hardlink_key,
                    xattrs: Vec::new(),
                    acl_access: None,
                    acl_default: None,
                    selinux_label: None,
                    linux_capability: None,
                    sparse_chunks: Vec::new(),
                    device_major: captured.device_major,
                    device_minor: captured.device_minor,
                });
                continue;
            }
        }

        if !meta.is_dir() {
            bail!("unsupported input type: {}", input.display());
        }

        let mut saw_child = false;
        for entry in walkdir::WalkDir::new(&abs).follow_links(false) {
            let entry = entry?;
            let rel = entry
                .path()
                .strip_prefix(&abs)
                .context("strip input prefix")?
                .to_string_lossy()
                .to_string();
            if rel.is_empty() {
                continue;
            }
            saw_child = true;
            let rel_path = normalize_logical_path(&rel);
            let entry_meta = std::fs::symlink_metadata(entry.path())
                .with_context(|| format!("stat {}", entry.path().display()))?;
            let captured = capture_mode_mtime_uid_gid(&entry_meta);
            #[cfg(unix)]
            use std::os::unix::fs::FileTypeExt;
            let kind = if entry.file_type().is_file() {
                EntryKind::Regular
            } else if entry.file_type().is_symlink() {
                EntryKind::Symlink
            } else if entry.file_type().is_dir() {
                EntryKind::Directory
            } else if entry_meta.file_type().is_fifo() {
                EntryKind::Fifo
            } else if entry_meta.file_type().is_char_device() {
                EntryKind::CharDevice
            } else if entry_meta.file_type().is_block_device() {
                EntryKind::BlockDevice
            } else {
                continue;
            };
            let (xattrs, security) = if requirements.capture_xattrs
                && (kind == EntryKind::Regular || kind == EntryKind::Directory)
            {
                capture_xattrs(entry.path())
            } else {
                (Vec::new(), CapturedSecurityMetadata::default())
            };
            let (mode, mtime) = (captured.mode, captured.mtime);
            let (uid, gid, uname, gname) = if requirements.capture_ownership_names {
                let uname = uname_by_uid.entry(captured.uid).or_insert_with(|| {
                    let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                    uname
                });
                let gname = gname_by_gid.entry(captured.gid).or_insert_with(|| {
                    let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                    gname
                });
                (captured.uid, captured.gid, uname.clone(), gname.clone())
            } else {
                (captured.uid, captured.gid, None, None)
            };

            files.push(InputFile {
                rel_path,
                abs_path: entry.path().to_path_buf(),
                raw_len: if kind == EntryKind::Regular {
                    entry_meta.len()
                } else {
                    0
                },
                kind,
                mode,
                mtime,
                uid,
                gid,
                uname,
                gname,
                hardlink_key: captured.hardlink_key,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: if requirements.capture_sparse_layout && kind == EntryKind::Regular {
                    capture_sparse_chunks(entry.path(), entry_meta.len())
                } else {
                    Vec::new()
                },
                device_major: captured.device_major,
                device_minor: captured.device_minor,
            });
        }
        if !saw_child {
            let captured = capture_mode_mtime_uid_gid(&meta);
            let (xattrs, security) = if requirements.capture_xattrs {
                capture_xattrs(input)
            } else {
                (Vec::new(), CapturedSecurityMetadata::default())
            };
            let name = abs
                .file_name()
                .context("input directory has no file name")?
                .to_string_lossy()
                .to_string();
            files.push(InputFile {
                rel_path: normalize_logical_path(&name),
                abs_path: abs,
                raw_len: 0,
                kind: EntryKind::Directory,
                mode: captured.mode,
                mtime: captured.mtime,
                uid: captured.uid,
                gid: captured.gid,
                uname: if requirements.capture_ownership_names {
                    let (uname, _) = capture_ownership_names(captured.uid, captured.gid);
                    uname
                } else {
                    None
                },
                gname: if requirements.capture_ownership_names {
                    let (_, gname) = capture_ownership_names(captured.uid, captured.gid);
                    gname
                } else {
                    None
                },
                hardlink_key: None,
                xattrs,
                acl_access: security.acl_access,
                acl_default: security.acl_default,
                selinux_label: security.selinux_label,
                linux_capability: security.linux_capability,
                sparse_chunks: Vec::new(),
                device_major: None,
                device_minor: None,
            });
        }
    }

    files.sort_by(|a, b| {
        a.rel_path
            .cmp(&b.rel_path)
            .then_with(|| a.abs_path.cmp(&b.abs_path))
    });
    Ok(files)
}

fn metadata_capture_requirements_for_profile(
    profile: PreservationProfile,
) -> MetadataCaptureRequirements {
    match profile {
        PreservationProfile::Full => MetadataCaptureRequirements {
            capture_ownership_names: true,
            capture_xattrs: true,
            capture_sparse_layout: true,
        },
        PreservationProfile::Basic => MetadataCaptureRequirements {
            capture_ownership_names: false,
            capture_xattrs: false,
            capture_sparse_layout: true,
        },
        PreservationProfile::PayloadOnly => MetadataCaptureRequirements {
            capture_ownership_names: false,
            capture_xattrs: false,
            capture_sparse_layout: false,
        },
    }
}

fn plan_pack_profile_impl(
    candidates: Vec<InputFile>,
    profile: PreservationProfile,
) -> PackProfilePlan {
    let mut included = Vec::with_capacity(candidates.len());
    let mut omitted = Vec::new();

    for mut entry in candidates {
        let omission_reason = match (profile, entry.kind) {
            (PreservationProfile::Basic, EntryKind::Fifo)
            | (PreservationProfile::Basic, EntryKind::CharDevice)
            | (PreservationProfile::Basic, EntryKind::BlockDevice) => {
                Some(ProfileOmissionReason::BasicOmitsSpecialEntries)
            }
            (PreservationProfile::PayloadOnly, EntryKind::Symlink) => {
                Some(ProfileOmissionReason::PayloadOnlyOmitsSymlinks)
            }
            (PreservationProfile::PayloadOnly, EntryKind::Fifo)
            | (PreservationProfile::PayloadOnly, EntryKind::CharDevice)
            | (PreservationProfile::PayloadOnly, EntryKind::BlockDevice) => {
                Some(ProfileOmissionReason::PayloadOnlyOmitsSpecialEntries)
            }
            _ => None,
        };

        if let Some(reason) = omission_reason {
            omitted.push(ProfileOmission {
                rel_path: entry.rel_path,
                kind: entry.kind,
                reason,
            });
            continue;
        }

        match profile {
            PreservationProfile::Full => {}
            PreservationProfile::Basic => {
                entry.xattrs.clear();
                entry.uid = 0;
                entry.gid = 0;
                entry.uname = None;
                entry.gname = None;
                entry.acl_access = None;
                entry.acl_default = None;
                entry.selinux_label = None;
                entry.linux_capability = None;
            }
            PreservationProfile::PayloadOnly => {
                match entry.kind {
                    EntryKind::Regular => {
                        entry.mode = 0;
                        entry.mtime = -1;
                        entry.hardlink_key = None;
                        entry.sparse_chunks.clear();
                    }
                    EntryKind::Directory => {
                        entry.mode = 0;
                        entry.mtime = -1;
                    }
                    _ => {}
                }
                entry.xattrs.clear();
                entry.uid = 0;
                entry.gid = 0;
                entry.uname = None;
                entry.gname = None;
                entry.acl_access = None;
                entry.acl_default = None;
                entry.selinux_label = None;
                entry.linux_capability = None;
                entry.device_major = None;
                entry.device_minor = None;
            }
        }
        included.push(entry);
    }

    PackProfilePlan { included, omitted }
}

fn emit_profile_warnings_impl(omissions: &[ProfileOmission]) {
    for omission in omissions {
        eprintln!(
            "WARNING[preservation-omit]: omitted '{}' ({:?}) due to preservation profile {}",
            omission.rel_path,
            omission.kind,
            omission.reason.profile_name()
        );
    }
}

impl ProfileOmissionReason {
    fn profile_name(self) -> &'static str {
        match self {
            ProfileOmissionReason::BasicOmitsSpecialEntries => "basic",
            ProfileOmissionReason::PayloadOnlyOmitsSymlinks
            | ProfileOmissionReason::PayloadOnlyOmitsSpecialEntries => "payload-only",
        }
    }
}

fn normalize_logical_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(unix)]
fn libc_s_ifmt() -> u32 {
    0o170000
}
#[cfg(not(unix))]
fn libc_s_ifmt() -> u32 {
    0
}
#[cfg(unix)]
fn libc_s_ifchr() -> u32 {
    0o020000
}
#[cfg(not(unix))]
fn libc_s_ifchr() -> u32 {
    0
}
#[cfg(unix)]
fn libc_s_ifblk() -> u32 {
    0o060000
}
#[cfg(not(unix))]
fn libc_s_ifblk() -> u32 {
    0
}
#[cfg(unix)]
fn libc_seek_data() -> i32 {
    3
}
#[cfg(unix)]
fn libc_seek_hole() -> i32 {
    4
}
#[cfg(unix)]
fn libc_major(dev: u64) -> u64 {
    ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff)
}
#[cfg(unix)]
fn libc_minor(dev: u64) -> u64 {
    (dev & 0xff) | ((dev >> 12) & !0xff)
}
#[cfg(unix)]
fn sparse_lseek(fd: std::os::unix::io::RawFd, off: u64, whence: i32) -> Option<u64> {
    unsafe extern "C" {
        fn lseek(fd: std::os::unix::io::RawFd, offset: i64, whence: i32) -> i64;
    }
    let value = unsafe { lseek(fd, off as i64, whence) };
    if value < 0 { None } else { Some(value as u64) }
}

fn reject_duplicate_logical_paths_impl(files: &[InputFile]) -> Result<()> {
    let mut path_sources: BTreeMap<&str, Vec<(EntryKind, String)>> = BTreeMap::new();

    for file in files {
        path_sources
            .entry(file.rel_path.as_str())
            .or_default()
            .push((file.kind, file.abs_path.display().to_string()));
    }

    for (logical_path, mut sources) in path_sources {
        let all_dirs = sources
            .iter()
            .all(|(kind, _)| *kind == EntryKind::Directory);
        if sources.len() > 1 && !all_dirs {
            let mut source_paths = sources
                .drain(..)
                .map(|(_, source)| source)
                .collect::<Vec<_>>();
            source_paths.sort();
            bail!(
                "duplicate logical archive path '{logical_path}' from inputs: {}",
                source_paths.join(", ")
            );
        }
    }

    Ok(())
}
