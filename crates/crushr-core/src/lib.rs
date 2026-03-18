//! Engine and algorithms for `crushr`.
//!
//! `crushr-core` operates over minimal IO traits (read-at / write-at) so the logic
//! can be tested deterministically and reused by CLI/TUI without duplicating parsers.
//!
//! Filesystem integration, directory walking, xattrs, caching, and concurrency live in the
//! `crushr` crate.
//!
//! Boundary note: this crate is a bounded implementation crate for workspace tools,
//! not a stability-promised external SDK.

pub mod io;
pub mod open;
pub mod propagation;
pub mod snapshot;

pub mod extraction;
pub mod impact;
pub mod verification_model;
pub mod verify;
