// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildMetadata {
    pub(crate) version: String,
    pub(crate) commit: String,
    pub(crate) built: String,
    pub(crate) target: String,
    pub(crate) rust: String,
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

    pub(crate) fn from_env() -> Self {
        Self::from_parts(
            option_env!("CRUSHR_VERSION").unwrap_or(crushr::product_version()),
            option_env!("CRUSHR_GIT_COMMIT"),
            option_env!("CRUSHR_BUILD_TIMESTAMP"),
            option_env!("CRUSHR_TARGET_TRIPLE"),
            option_env!("CRUSHR_RUSTC_VERSION"),
        )
    }
}

pub(crate) fn render_about(metadata: &BuildMetadata) -> String {
    let mut out = String::new();

    out.push_str("crushr / about\n");
    out.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n\n",
    );
    out.push_str("  Deterministic archives with verifiable structure and explicit outcomes.\n\n");

    out.push_str("Build\n");
    kv_line(&mut out, "version", &metadata.version);
    kv_line(&mut out, "commit", &metadata.commit);
    kv_line(&mut out, "built", &metadata.built);
    kv_line(&mut out, "target", &metadata.target);
    kv_line(&mut out, "rust", &metadata.rust);

    out.push_str("\nBehavior\n");
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

    out.push_str("\nData Model\n");
    out.push_str("  tail-framed archive layout\n");
    out.push_str("  index + ledger backed structure\n");
    out.push_str("  optional dictionary support\n");
    out.push_str("  deterministic output contracts\n");

    out.push_str("\nBuilt with\n");
    out.push_str("  Rust • clap • serde • zstd • blake3\n");
    kv_line(&mut out, "Notices", "THIRD_PARTY_NOTICES.md");
    kv_line(&mut out, "Source", "https://github.com/UglyEgg/crushr");

    out.push_str("\nSupport\n");
    out.push_str("  If something looks wrong, attach:\n");
    out.push_str("    crushr info <archive> --json\n");
    out.push_str("    crushr extract --verify <archive>\n");

    out
}

fn kv_line(out: &mut String, label: &str, value: &str) {
    let _ = writeln!(out, "  {:<16} {}", label, value);
}

#[cfg(test)]
mod tests {
    use super::{render_about, BuildMetadata};

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
            assert!(rendered.contains(&format!("  {:<16} unknown", label)));
        }
    }
}
