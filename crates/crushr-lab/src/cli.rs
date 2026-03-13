use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub const USAGE: &str = "usage:\n  crushr-lab corrupt <input> <output> [--model <bit_flip|byte_overwrite|zero_fill|truncation|tail_damage> --target <header|index|payload|tail> --magnitude <1B|256B|4KB> --seed <1337|2600|65535> --scenario-id <id> [--offset <u64>]]\n  crushr-lab write-phase2-manifest [--output <path>]\n  crushr-lab build-phase2-foundation [--artifact-dir <path>]\n  crushr-lab run-phase2-execution [--manifest <path> --foundation-report <path> --artifact-dir <path>]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Corrupt,
    WritePhase2Manifest,
    BuildPhase2Foundation,
    RunPhase2Execution,
}

impl Command {
    pub fn from_str(raw: &str) -> Option<Self> {
        match raw {
            "corrupt" => Some(Self::Corrupt),
            "write-phase2-manifest" => Some(Self::WritePhase2Manifest),
            "build-phase2-foundation" => Some(Self::BuildPhase2Foundation),
            "run-phase2-execution" => Some(Self::RunPhase2Execution),
            _ => None,
        }
    }
}

pub fn print_usage() {
    eprintln!("{USAGE}");
}

pub fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to derive workspace root")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parsing_handles_known_and_unknown_commands() {
        assert_eq!(Command::from_str("corrupt"), Some(Command::Corrupt));
        assert_eq!(Command::from_str(""), None);
        assert_eq!(Command::from_str("unknown"), None);
    }
}
