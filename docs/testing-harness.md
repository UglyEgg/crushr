# Testing Harness

This repository uses two layers of validation:

## Runtime validation

Use standard Rust checks against runtime binaries and tests.

Recommended commands:

- `cargo test -p crushr --tests`
- `cargo clippy --all-targets --all-features`
- `cargo fmt --check`

## Lab validation

Research comparisons and corruption scenarios run through lab tooling.

- Binary: `crushr-lab-salvage`
- Scope: format experiments, corruption harness runs, comparison summaries

Lab commands are research-only and must not be treated as canonical extraction behavior.
