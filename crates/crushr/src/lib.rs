// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! Library support crate for the `crushr-*` binaries.
//!
//! ## Boundary policy
//! - Stable product surface: CLI tools (`crushr-pack`, `crushr-info`,
//!   `crushr-extract`, `crushr-extract --verify`).
//! - Bounded internal surface: `format` and `index_codec` modules, used by this
//!   repository's binaries/tests.
//! - Internal-only implementation: extraction path confinement helpers are kept
//!   private and are not supported as a library API.
//!
//! This crate is not a general external SDK.
//!
//! ```compile_fail
//! // Internal extraction confinement helpers are intentionally not public API.
//! use crushr::extraction_path::resolve_confined_path;
//! ```
pub mod about;
pub mod cli_presentation;
pub mod commands;
mod extraction_path;
pub mod format;
pub mod index_codec;
pub mod strict_extract_impl;

pub mod versioning;

pub use versioning::product_version;
