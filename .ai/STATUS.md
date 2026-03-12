# crushr Development Status

Current Phase: Phase 1 — Integrity Intelligence

Active Objective:

Step 1.3 — Extraction Result Formalization (completed)

Goal status:

Formalized the current `crushr-extract --json` minimal v1 result contract as explicit docs + schema + deterministic contract tests, without expanding extraction behavior.

What changed:

- Added dedicated extraction result contract documentation: `docs/CONTRACTS/EXTRACTION_RESULT_V1.md`.
- Added versioned schema for extraction JSON results: `schemas/crushr-extract-result.v1.schema.json`.
- Extended extraction integration assertions to lock strict-vs-salvage field presence/absence, deterministic list ordering, stable refusal reason values, and error-envelope structure.
- Added dedicated automated schema-validation harness test covering strict-success, salvage-partial, and structural-error envelopes against the v1 extract-result schema.
- Linked contract docs from `docs/CONTRACTS/README.md` and `docs/CONTRACTS/ERROR_MODEL.md`.
- Added a targeted clippy allow on `crushr-core::io::Len` to keep required `-D warnings` checks green without API/behavior changes.

Active constraints:

- Minimal v1 extraction scope remains limited to regular files with one-block-per-file mapping.
- No speculative recovery, reconstruction, repair, or hole-filling behavior exists.
- Strict mode remains default; salvage mode remains explicit.

Next actions:

- Await next bounded task packet for Phase 1.
