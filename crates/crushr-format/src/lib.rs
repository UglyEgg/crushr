//! On-disk format primitives for `crushr`.
//!
//! This crate is the single source of truth for:
//! - byte layouts (BLK*/IDX*/DCT*/FTR*)
//! - encoding/decoding
//! - structural validation helpers
//!
//! It intentionally does **not** perform filesystem IO, directory walking, or CLI/TUI concerns.
//!
//! Boundary note: `crushr-format` is used as a core implementation boundary inside
//! this workspace; external stability promises are defined by product/tool contracts,
//! not by exposing all module internals as a public platform API.

pub mod blk3;
pub mod dct1;
pub mod ledger;
pub mod version;

pub mod ftr4;
pub mod tailframe;
