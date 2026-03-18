# Handoff

Current boundary update (2026-03-18):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.
- Reader-boundary hardening tightened legacy permissive behavior: block-region mismatch and decoded raw-length mismatch in `read.rs` now fail closed.
- `crushr-extract --verify` now runs strict extraction semantics in an isolated temp output path to ensure strict-verify alignment.

Next focus:
- CRUSHR-HARDEN-03D is complete: strict verification alignment and reader-boundary tightening landed.
- Continue CRUSHR-HARDEN-03E follow-up to convert remaining untyped comparison summary assembly into typed row/summary structs.


## CRUSHR-HARDEN-03C handoff
- Active comparison summaries now have dedicated schema files under `schemas/` for FORMAT-12/13/14A/15 baseline + stress outputs.
- Integration test `comparison_output_schemas.rs` runs active comparison commands and checks emitted artifacts against required schema fields/version constants.
- Follow-up 03E should convert remaining untyped `serde_json::Value` summary assembly in `lab/comparison.rs` into typed row/summary structs.
