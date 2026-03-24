// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fmt::Write;

use crate::cli_presentation::{KV_LABEL_WIDTH, VisualToken, canonical_divider, paint_for_stdout};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildMetadata {
    pub version: String,
    pub commit: String,
    pub built: String,
    pub target: String,
    pub rust: String,
}

impl BuildMetadata {
    fn with_fallback(value: Option<&str>) -> String {
        value
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or("unknown")
            .to_string()
    }

    fn from_parts(
        version: &str,
        commit: Option<&str>,
        built: Option<&str>,
        target: Option<&str>,
        rust: Option<&str>,
    ) -> Self {
        Self {
            version: version.to_string(),
            commit: Self::with_fallback(commit),
            built: Self::with_fallback(built),
            target: Self::with_fallback(target),
            rust: Self::with_fallback(rust),
        }
    }

    pub fn from_env() -> Self {
        Self::from_parts(
            option_env!("CRUSHR_VERSION").unwrap_or(crate::product_version()),
            option_env!("CRUSHR_GIT_COMMIT"),
            option_env!("CRUSHR_BUILD_TIMESTAMP"),
            option_env!("CRUSHR_TARGET_TRIPLE"),
            option_env!("CRUSHR_RUSTC_VERSION"),
        )
    }
}

pub fn render_about(metadata: &BuildMetadata) -> String {
    let mut out = String::new();

    out.push('\n');
    line(&mut out, VisualToken::TitleProductLine, "crushr  /  about");
    out.push_str(&canonical_divider());
    out.push_str("\n\n");
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  Deterministic archives with verifiable structure and explicit outcomes.",
    );
    out.push('\n');

    line(&mut out, VisualToken::SectionHeader, "Build");
    kv_line(&mut out, "version", &metadata.version);
    kv_line(&mut out, "commit", &metadata.commit);
    kv_line(&mut out, "built", &metadata.built);
    kv_line(&mut out, "target", &metadata.target);
    kv_line(&mut out, "rust", &metadata.rust);

    out.push('\n');
    line(&mut out, VisualToken::SectionHeader, "Behavior");
    kv_line(&mut out, "pack", "deterministic archive creation");
    kv_line(
        &mut out,
        "extract",
        "strict extraction (verification-gated)",
    );
    kv_line(&mut out, "verify", "structural and integrity validation");
    kv_line(
        &mut out,
        "salvage",
        "research-mode recovery planning (non-canonical)",
    );

    out.push('\n');
    line(&mut out, VisualToken::SectionHeader, "Data Model");
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  tail-framed archive layout",
    );
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  index + ledger backed structure",
    );
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  optional dictionary support",
    );
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  deterministic output contracts",
    );

    out.push('\n');
    line(&mut out, VisualToken::SectionHeader, "Built with");
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  Rust • clap • serde • zstd • blake3",
    );
    kv_line(&mut out, "Notices", "THIRD_PARTY_NOTICES.md");
    kv_line(&mut out, "Source", "https://github.com/UglyEgg/crushr");

    out.push('\n');
    line(&mut out, VisualToken::SectionHeader, "Support");
    line(
        &mut out,
        VisualToken::SecondaryText,
        "  If something looks wrong, attach:",
    );
    line(
        &mut out,
        VisualToken::SecondaryText,
        "    crushr info <archive> --json",
    );
    line(
        &mut out,
        VisualToken::SecondaryText,
        "    crushr extract --verify <archive>",
    );

    out
}

fn kv_line(out: &mut String, label: &str, value: &str) {
    let padded = format!("{label:<width$}", width = KV_LABEL_WIDTH);
    let painted = paint_for_stdout(VisualToken::PrimaryLabel, &padded);
    let _ = writeln!(out, "  {} {}", painted, value);
}

fn line(out: &mut String, token: VisualToken, text: &str) {
    let _ = writeln!(out, "{}", paint_for_stdout(token, text));
}

#[cfg(test)]
mod tests {
    use super::{BuildMetadata, render_about};

    #[test]
    fn golden_about_output_is_locked_for_fixed_metadata() {
        let metadata = BuildMetadata {
            version: "0.2.2".to_string(),
            commit: "2d06ea041fd3".to_string(),
            built: "2026-03-20T18:12:04Z".to_string(),
            target: "x86_64-unknown-linux-musl".to_string(),
            rust: "rustc 1.93.1 (01f6ddf75 2026-02-11)".to_string(),
        };

        let expected = include_str!("../tests/golden/about.txt");
        assert_eq!(render_about(&metadata), expected);
    }

    #[test]
    fn metadata_fallback_is_bounded_and_non_empty() {
        let metadata = BuildMetadata::from_parts("0.2.2", None, Some(""), None, Some("   "));
        let rendered = render_about(&metadata);

        for label in ["commit", "built", "target", "rust"] {
            assert!(rendered.contains(&format!(
                "  {:<width$} unknown",
                label,
                width = super::KV_LABEL_WIDTH
            )));
        }
    }
}
