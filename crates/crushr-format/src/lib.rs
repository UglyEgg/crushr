//! On-disk format primitives for `crushr`.
//!
//! This crate is the single source of truth for:
//! - byte layouts (BLK*/IDX*/DCT*/FTR*)
//! - encoding/decoding
//! - structural validation helpers
//!
//! It intentionally does **not** perform filesystem IO, directory walking, or CLI/TUI concerns.

pub mod version;
pub mod ledger;
pub mod blk3;
pub mod dct1;

pub mod ftr4;
