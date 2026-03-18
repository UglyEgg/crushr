# Handoff

Current boundary update (2026-03-18):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.
- Reader-boundary hardening tightened legacy permissive behavior: block-region mismatch and decoded raw-length mismatch in `read.rs` now fail closed.
- `crushr-extract --verify` now runs strict extraction semantics in an isolated temp output path to ensure strict-verify alignment.

Next focus:
- CRUSHR-HARDEN-03E is complete: comparison engine decomposition landed with bounded module files under `lab/comparison/`.
- Continue CRUSHR-HARDEN-03F follow-up for helper visibility tightening and incremental typed-helper migration in format09/10 internals.


## CRUSHR-HARDEN-03E handoff
- Active comparison summaries now have dedicated schema files under `schemas/` for FORMAT-12/13/14A/15 baseline + stress outputs.
- Integration test `comparison_output_schemas.rs` runs active comparison commands and checks emitted artifacts against required schema fields/version constants.
- Comparison engine is now split into `lab/comparison/mod.rs`, `common.rs`, `experimental.rs`, `format06_to12.rs`, and `format13_to15.rs`.
- Command dispatch in `crushr-lab-salvage` is unchanged; import path now points to `comparison/mod.rs`.
- Remaining concern: format09/10 helper internals still use permissive helper visibility and some untyped `Value` helper flow that should be tightened in follow-up 03F.
