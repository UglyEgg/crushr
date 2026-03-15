# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-FORMAT-03-f1 complete (lab-salvage command dispatch/help fix for comparison workflow invocations)

Recent completed packet: CRUSHR-FORMAT-03-f1 (dispatch/help/compatibility repair so documented comparison commands are invokable)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-pack` emits redundant verified file-map ledger metadata (`crushr-redundant-file-map.v1`) in LDG1 for new archives.
- `crushr-salvage` uses primary IDX3 mapping first, then redundant metadata only when primary mapping is unusable and redundant metadata fully verifies.
- `crushr-lab-salvage` now supports `run-redundant-map-comparison` for deterministic old-style vs new-style targeted comparison runs and emits compact `comparison_summary.json` and `comparison_summary.md`.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation.
- Comparison workflow is bounded (24 deterministic scenarios), storage-conscious, and does not rerun the full Phase 2 matrix.

## Next actions

1. Preserve strict extraction interfaces/semantics untouched.
2. Keep redundant map fallback strict all-or-nothing and deterministic.
3. Preserve deterministic comparison ordering/classification and compact grouped metrics outputs.
4. Keep Phase 2 corpus and frozen artifacts unchanged.
5. Keep file-identity extent path experimental and opt-in only.

## Latest packet: CRUSHR-FORMAT-03-f1

- Added explicit top-level help mode for `crushr-lab-salvage --help` / `-h` / `help` with bounded usage text listing supported modes.
- Hardened parser dispatch so known comparison subcommand names are not treated as positional input paths when used in the wrong position.
- Added focused harness tests for help discoverability, subcommand misparse regression guard, and direct `run-file-identity-comparison` invocation.

