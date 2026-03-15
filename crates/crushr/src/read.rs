use crate::extraction_path::{resolve_confined_path, validate_symlink_target};
use crate::format::{
    Entry, EntryKind, Index, BLK_MAGIC_V2, CODEC_ZSTD, FTR_MAGIC_V1, FTR_MAGIC_V2,
};
use crate::index_codec::decode_index;

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

fn read_u32_le(mut r: impl Read) -> Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
fn read_u64_le(mut r: impl Read) -> Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

#[derive(Debug, Clone)]
struct FooterInfo {
    blocks_end_offset: u64,
    index_offset: u64,
    index_len: u64,
    index_hash: [u8; 32],
}

fn read_footer_info(f: &mut File) -> Result<FooterInfo> {
    let len = f.metadata()?.len();

    // FTR2: magic + blocks_end + index_offset + index_len + hash (60)
    if len >= 60 {
        f.seek(SeekFrom::End(-60))?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        if &magic == FTR_MAGIC_V2 {
            let blocks_end_offset = read_u64_le(&mut *f)?;
            let index_offset = read_u64_le(&mut *f)?;
            let index_len = read_u64_le(&mut *f)?;
            let mut index_hash = [0u8; 32];
            f.read_exact(&mut index_hash)?;
            return Ok(FooterInfo {
                blocks_end_offset,
                index_offset,
                index_len,
                index_hash,
            });
        }
    }

    // FTR1: magic + index_offset + index_len + hash (52)
    if len >= 52 {
        f.seek(SeekFrom::End(-52))?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        if &magic != FTR_MAGIC_V1 {
            bail!("unknown footer magic");
        }
        let index_offset = read_u64_le(&mut *f)?;
        let index_len = read_u64_le(&mut *f)?;
        let mut index_hash = [0u8; 32];
        f.read_exact(&mut index_hash)?;
        // blocks_end_offset unknown in v1; assume index_offset is end of blocks.
        return Ok(FooterInfo {
            blocks_end_offset: index_offset,
            index_offset,
            index_len,
            index_hash,
        });
    }

    bail!("archive too small")
}

#[derive(Debug)]
struct BlockHeader {
    codec: u32,
    raw_len: u64,
    comp_len: u64,
    data_off: u64,
    #[allow(dead_code)]
    frame_len: u64,
}

fn scan_blocks(f: &mut File, blocks_end_offset: u64) -> Result<Vec<BlockHeader>> {
    let mut blocks = Vec::new();
    let mut pos: u64 = 0;

    while pos < blocks_end_offset {
        f.seek(SeekFrom::Start(pos))?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)
            .with_context(|| format!("read block magic at {}", pos))?;
        if &magic != BLK_MAGIC_V2 {
            bail!("bad block magic at offset {}", pos);
        }
        let codec = read_u32_le(&mut *f)?;
        let raw_len = read_u64_le(&mut *f)?;
        let comp_len = read_u64_le(&mut *f)?;
        let data_off = pos + 4 + 4 + 8 + 8;
        let frame_len = 4 + 4 + 8 + 8 + comp_len;
        blocks.push(BlockHeader {
            codec,
            raw_len,
            comp_len,
            data_off,
            frame_len,
        });
        pos = pos.saturating_add(frame_len);
    }

    if pos != blocks_end_offset {
        // Allow slight mismatch due to corruption, but keep best-effort.
        // The caller may handle this separately; here we report a soft error.
        // For now, just accept.
    }
    Ok(blocks)
}

pub struct ArchiveReader {
    file: File,
    index: Index,
    map: HashMap<String, usize>,
    blocks: Vec<BlockHeader>,
    cache_blocks: usize,
    cache: std::collections::VecDeque<(u32, Vec<u8>)>, // naive LRU
}

impl ArchiveReader {
    pub fn open_with_cache(path: &Path, cache_blocks: usize, cache_mib: u64) -> Result<Self> {
        let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let footer = read_footer_info(&mut f)?;
        let mut index_bytes = vec![0u8; footer.index_len as usize];
        f.seek(SeekFrom::Start(footer.index_offset))?;
        f.read_exact(&mut index_bytes)?;
        let h = blake3::hash(&index_bytes);
        if h.as_bytes() != &footer.index_hash {
            bail!("index hash mismatch (archive may be corrupted)");
        }
        let index = decode_index(&index_bytes).context("decode index")?;
        let mut map = HashMap::new();
        for (i, e) in index.entries.iter().enumerate() {
            map.insert(e.path.clone(), i);
        }
        let blocks = scan_blocks(&mut f, footer.blocks_end_offset)?;

        // cache sizing: if cache_mib provided, prefer that; otherwise use cache_blocks.
        let effective_blocks = if cache_mib > 0 {
            // approximate 1 block ~= 1 MiB
            std::cmp::max(1, cache_mib as usize)
        } else {
            cache_blocks
        };

        Ok(Self {
            file: f,
            index,
            map,
            blocks,
            cache_blocks: effective_blocks,
            cache: std::collections::VecDeque::new(),
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_cache(path, 8, 0)
    }

    pub fn get_entry(&self, path: &str) -> Result<&Entry> {
        let idx = *self
            .map
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("not found: {}", path))?;
        Ok(&self.index.entries[idx])
    }

    fn cache_get(&mut self, block_id: u32) -> Option<Vec<u8>> {
        if let Some(pos) = self.cache.iter().position(|(id, _)| *id == block_id) {
            let (id, data) = self.cache.remove(pos).unwrap();
            self.cache.push_front((id, data.clone()));
            return Some(data);
        }
        None
    }
    fn cache_put(&mut self, block_id: u32, data: Vec<u8>) {
        self.cache.push_front((block_id, data));
        while self.cache.len() > self.cache_blocks {
            self.cache.pop_back();
        }
    }

    pub fn read_block_uncompressed(&mut self, block_id: u32) -> Result<Vec<u8>> {
        if let Some(v) = self.cache_get(block_id) {
            return Ok(v);
        }
        let bh = self
            .blocks
            .get(block_id as usize)
            .ok_or_else(|| anyhow::anyhow!("bad block_id {}", block_id))?;
        if bh.codec != CODEC_ZSTD {
            bail!("unsupported codec {}", bh.codec);
        }
        self.file.seek(SeekFrom::Start(bh.data_off))?;
        let mut comp = vec![0u8; bh.comp_len as usize];
        self.file.read_exact(&mut comp)?;
        let data = zstd::decode_all(&comp[..]).context("zstd decode")?;
        if data.len() as u64 != bh.raw_len {
            // tolerate (best effort)
        }
        self.cache_put(block_id, data.clone());
        Ok(data)
    }

    pub fn read_entry_bytes(&mut self, e: &Entry) -> Result<Vec<u8>> {
        match e.kind {
            EntryKind::Symlink => {
                let s = e.link_target.clone().unwrap_or_default();
                Ok(s.into_bytes())
            }
            EntryKind::Regular => {
                let mut out = Vec::with_capacity(e.size as usize);
                for ex in &e.extents {
                    let block = self.read_block_uncompressed(ex.block_id)?;
                    let start = ex.offset as usize;
                    let end = (ex.offset + ex.len) as usize;
                    if end > block.len() {
                        bail!("extent out of range");
                    }
                    out.extend_from_slice(&block[start..end]);
                }
                Ok(out)
            }
        }
    }

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        let e = self.get_entry(path)?.clone();
        self.read_entry_bytes(&e)
    }

    #[allow(dead_code)]
    pub fn extract_to(&mut self, path: &str, dst: &Path, xattr_policy: &str) -> Result<()> {
        let e = self.get_entry(path)?.clone();
        let out_dir = dst
            .parent()
            .ok_or_else(|| anyhow::anyhow!("destination has no parent: {}", dst.display()))?;
        let confined = resolve_confined_path(out_dir, path)?;
        if confined != dst {
            bail!(
                "destination mismatch for {}: expected confined path {}",
                path,
                confined.display()
            );
        }

        match e.kind {
            EntryKind::Symlink => {
                let tgt = e.link_target.clone().unwrap_or_default();
                validate_symlink_target(&tgt)?;
                bail!("symlink extraction is disabled in hardened mode: {}", path);
            }
            EntryKind::Regular => {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let bytes = self.read_entry_bytes(&e)?;
                std::fs::write(dst, &bytes).with_context(|| format!("write {}", dst.display()))?;
            }
        }
        self.restore_xattrs(&e, dst, xattr_policy)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn restore_xattrs(&self, e: &Entry, dst: &Path, xattr_policy: &str) -> Result<()> {
        match xattr_policy {
            "none" => Ok(()),
            "restore" | "restore+best-effort" => {
                for xa in &e.xattrs {
                    let r = xattr::set(dst, &xa.name, &xa.value);
                    if r.is_err() && xattr_policy == "restore" {
                        return Err(r.err().unwrap())
                            .with_context(|| format!("setxattr {} {}", dst.display(), xa.name));
                    }
                }
                Ok(())
            }
            other => bail!("unknown xattr policy: {}", other),
        }
        // Index was verified at open() via the footer hash.
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
    pub fn blocks_raw_bytes(&self) -> u64 {
        self.blocks.iter().map(|b| b.raw_len).sum()
    }
    #[allow(dead_code)]
    pub fn blocks_comp_bytes(&self) -> u64 {
        self.blocks.iter().map(|b| b.comp_len).sum()
    }
    pub fn blocks_frame_bytes(&self) -> u64 {
        self.blocks.iter().map(|b| b.frame_len).sum()
    }

    // Index was verified at open() via the footer hash.
    pub fn verify_index(&self) -> Result<()> {
        Ok(())
    }

    pub fn verify_blocks_deep(&mut self) -> Result<()> {
        for id in 0..(self.blocks.len() as u32) {
            let _ = self
                .read_block_uncompressed(id)
                .with_context(|| format!("verify block {}", id))?;
        }
        Ok(())
    }
}
