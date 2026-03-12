#[allow(dead_code)]
pub const BLK_MAGIC: &[u8; 4] = b"BLK1";
pub const BLK_MAGIC_V2: &[u8; 4] = b"BLK2";
pub const FTR_MAGIC_V1: &[u8; 4] = b"FTR1";
pub const FTR_MAGIC_V2: &[u8; 4] = b"FTR2";
#[allow(dead_code)]
pub const FTR_MAGIC_V3: &[u8; 4] = b"FTR3";

pub const CODEC_ZSTD: u32 = 1;

pub const IDX_MAGIC_V1: &[u8; 4] = b"IDX1";
pub const IDX_MAGIC_V2: &[u8; 4] = b"IDX2";
pub const IDX_MAGIC_V3: &[u8; 4] = b"IDX3";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Regular = 0,
    Symlink = 1,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: String,
    pub kind: EntryKind,
    pub mode: u32,
    pub mtime: i64,
    pub size: u64,
    pub extents: Vec<Extent>,
    pub link_target: Option<String>,
    pub xattrs: Vec<Xattr>,
}

#[derive(Debug, Clone)]
pub struct Extent {
    pub block_id: u32,
    pub offset: u64,
    pub len: u64,
}

#[derive(Debug, Clone)]
pub struct Xattr {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FooterV2 {
    pub blocks_end_offset: u64,
    pub index_offset: u64,
    pub index_len: u64,
    pub index_hash: [u8; 32],
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FooterV1 {
    pub index_offset: u64,
    pub index_len: u64,
    pub index_hash: [u8; 32],
}

pub fn is_probably_incompressible(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let exts = [
        ".png",
        ".jpg",
        ".jpeg",
        ".webp",
        ".gif",
        ".mp4",
        ".mkv",
        ".mov",
        ".mp3",
        ".ogg",
        ".opus",
        ".flac",
        ".pdf",
        ".zip",
        ".7z",
        ".rar",
        ".gz",
        ".xz",
        ".bz2",
        ".zst",
        ".dwarfs",
        ".dwarfsx",
        ".ktc",
        ".squashfs",
    ];
    exts.iter().any(|e| lower.ends_with(e))
}

pub fn classify_group(path: &str) -> u8 {
    let p = path.to_ascii_lowercase();
    if is_probably_incompressible(&p) {
        return 3;
    }
    if p.ends_with(".json")
        || p.ends_with(".yaml")
        || p.ends_with(".yml")
        || p.ends_with(".toml")
        || p.ends_with(".xml")
    {
        return 0;
    }
    if p.ends_with(".txt") || p.ends_with(".md") || p.ends_with(".rst") || p.ends_with(".log") {
        return 0;
    }
    if p.ends_with(".rs")
        || p.ends_with(".c")
        || p.ends_with(".cc")
        || p.ends_with(".cpp")
        || p.ends_with(".h")
        || p.ends_with(".hpp")
        || p.ends_with(".py")
        || p.ends_with(".sh")
        || p.ends_with(".bash")
        || p.ends_with(".zsh")
        || p.ends_with(".js")
        || p.ends_with(".ts")
        || p.ends_with(".css")
        || p.ends_with(".html")
    {
        return 1;
    }
    2
}

// Dictionary table
#[allow(dead_code)]
pub const DCT_MAGIC_V1: &[u8; 4] = b"DCT1";

// Embedded event frames
#[allow(dead_code)]
pub const EVT_MAGIC_V1: &[u8; 4] = b"EVT1";
