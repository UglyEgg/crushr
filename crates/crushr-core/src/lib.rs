//! Engine and algorithms for `crushr`.
//!
//! `crushr-core` operates over minimal IO traits (read-at / write-at) so the logic
//! can be tested deterministically and reused by CLI/TUI without duplicating parsers.
//!
//! Filesystem integration, directory walking, xattrs, caching, and concurrency live in the
//! `crushr` crate.

pub mod io;
pub mod snapshot;

pub mod impact;
