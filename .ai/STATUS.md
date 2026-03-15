# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-FORMAT-02 complete (experimental self-describing extents + distributed checkpoints + bounded three-arm comparison)

Recent completed packet: CRUSHR-FORMAT-02 (experimental metadata writer path + strict salvage fallback provenance + bounded three-arm comparison summaries)

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


## Latest packet: CRUSHR-FORMAT-02

- Added explicit experimental writer flag `--experimental-self-describing-extents` in `crushr-pack`.
- Added strict salvage support for experimental metadata provenance paths: `CHECKPOINT_MAP_PATH` and `SELF_DESCRIBING_EXTENT_PATH`.
- Added bounded comparison workflow `crushr-lab-salvage run-experimental-resilience-comparison --output <dir>`.
- Comparison outputs: `experimental_comparison_summary.json` and `experimental_comparison_summary.md`.
