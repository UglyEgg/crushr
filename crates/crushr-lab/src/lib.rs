// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use anyhow::Result;

mod cli;
mod phase2_audit;
mod phase2_comparison;
mod phase2_corruption;
mod phase2_domain;
mod phase2_foundation;
mod phase2_manifest;
mod phase2_normalization;
mod phase2_runner;

use cli::{Command, print_usage};
use phase2_audit::run_phase2_pretrial_audit_cmd;
use phase2_comparison::run_phase2_comparison_cmd;
use phase2_corruption::run_corrupt;
use phase2_foundation::run_phase2_foundation;
use phase2_manifest::write_phase2_manifest;
use phase2_normalization::run_phase2_normalization_cmd;
use phase2_runner::run_phase2_execution_cmd;

pub fn dispatch(raw_args: Vec<String>) -> Result<i32> {
    let mut args = raw_args.into_iter();
    let cmd = args.next().unwrap_or_default();
    let rest = args.collect();

    match Command::from_str(&cmd) {
        Some(Command::Corrupt) => run_corrupt(rest)?,
        Some(Command::WritePhase2Manifest) => write_phase2_manifest(rest)?,
        Some(Command::BuildPhase2Foundation) => run_phase2_foundation(rest)?,
        Some(Command::RunPhase2Execution) => run_phase2_execution_cmd(rest)?,
        Some(Command::RunPhase2PretrialAudit) => run_phase2_pretrial_audit_cmd(rest)?,
        Some(Command::RunPhase2Normalization) => run_phase2_normalization_cmd(rest)?,
        Some(Command::RunPhase2Comparison) => run_phase2_comparison_cmd(rest)?,
        None => {
            print_usage();
            return Ok(1);
        }
    }

    Ok(0)
}
