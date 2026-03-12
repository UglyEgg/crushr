//! Minimal IO traits used by `crushr-core`.

use anyhow::Result;

/// Random-access reader.
///
/// Implementations must be thread-safe if used concurrently by higher layers.
pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;
}

/// Random-access writer.
pub trait WriteAt {
    fn write_at(&mut self, offset: u64, buf: &[u8]) -> Result<()>;
}

/// Provides the total length of the underlying object.
#[allow(clippy::len_without_is_empty)]
pub trait Len {
    fn len(&self) -> Result<u64>;
}

/// Optional truncate support (used for append/repair flows).
pub trait Truncate {
    fn truncate(&mut self, len: u64) -> Result<()>;
}

/// Optional durability boundary.
pub trait Sync {
    fn sync_all(&mut self) -> Result<()>;
}
