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

use cli::{print_usage, Command};
use phase2_audit::run_phase2_pretrial_audit_cmd;
use phase2_comparison::run_phase2_comparison_cmd;
use phase2_corruption::run_corrupt;
use phase2_foundation::run_phase2_foundation;
use phase2_manifest::write_phase2_manifest;
use phase2_normalization::run_phase2_normalization_cmd;
use phase2_runner::run_phase2_execution_cmd;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    let raw_args = args.collect();

    match Command::from_str(&cmd) {
        Some(Command::Corrupt) => run_corrupt(raw_args),
        Some(Command::WritePhase2Manifest) => write_phase2_manifest(raw_args),
        Some(Command::BuildPhase2Foundation) => run_phase2_foundation(raw_args),
        Some(Command::RunPhase2Execution) => run_phase2_execution_cmd(raw_args),
        Some(Command::RunPhase2PretrialAudit) => run_phase2_pretrial_audit_cmd(raw_args),
        Some(Command::RunPhase2Normalization) => run_phase2_normalization_cmd(raw_args),
        Some(Command::RunPhase2Comparison) => run_phase2_comparison_cmd(raw_args),
        None => {
            print_usage();
            std::process::exit(1);
        }
    }
}
