// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

//! Shared CLI plumbing for the crushr tool suite.
//!
//! This crate intentionally keeps the surface small:
//! - global flags (modeled as a plain struct)
//! - logging initialization hooks
//! - shared output helpers (human/json)
//!
//! Individual tool binaries may use a full argument parser (e.g., clap) on top
//! of these types; the parser choice is not part of the public contract.

use anyhow::Result;

/// Standard output mode across tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

/// Global flags shared across all tools where they make sense.
///
/// NOTE: This is a semantic model. Parsing is handled by each tool binary.
#[derive(Debug, Clone)]
pub struct GlobalArgs {
    pub output: OutputMode,
    pub color: ColorMode,
    pub verbose: u8,
    pub quiet: bool,
    pub threads: Option<usize>,
    pub cache_mib: Option<u64>,
}

impl Default for GlobalArgs {
    fn default() -> Self {
        Self {
            output: OutputMode::Human,
            color: ColorMode::Auto,
            verbose: 0,
            quiet: false,
            threads: None,
            cache_mib: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Initialize logging/tracing based on global flags.
///
/// The concrete logging backend is intentionally deferred.
pub fn init_logging(_g: &GlobalArgs) -> Result<()> {
    // Skeleton: actual tracing/log setup will be introduced when tool binaries land.
    Ok(())
}

/// Standardized exit codes.
///
/// Tool binaries should map errors and verification outcomes into these codes.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    UserError = 1,
    CorruptionDetected = 2,
    RepairPerformed = 3,
    ToolFailure = 4,
}
