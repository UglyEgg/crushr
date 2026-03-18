# Handoff

Current boundary update (2026-03-17):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.

Next focus:
- CRUSHR-HARDEN-03B contract repair is complete: salvage output now aligns with `crushr-salvage-plan.v3` enums and typed reason-code vocabulary.
- Continue CRUSHR-HARDEN-03 follow-ups to normalize any remaining lab/report legacy classification labels (`*_VERIFIED`, `ORPHAN_EVIDENCE_ONLY`, `NO_VERIFIED_EVIDENCE`) where they still appear outside the salvage-plan contract path.


## CRUSHR-HARDEN-03C handoff
- Active comparison summaries now have dedicated schema files under `schemas/` for FORMAT-12/13/14A/15 baseline + stress outputs.
- Integration test `comparison_output_schemas.rs` runs active comparison commands and checks emitted artifacts against required schema fields/version constants.
- Follow-up 03E should convert remaining untyped `serde_json::Value` summary assembly in `lab/comparison.rs` into typed row/summary structs.
