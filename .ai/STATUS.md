# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-FORMAT-03 complete (experimental file-identity anchored extents + strict salvage fallback + bounded four-arm comparison)

Recent completed packet: CRUSHR-FORMAT-03 (file-identity anchored extents + strict path linkage verification + FILE_IDENTITY_EXTENT_PATH provenance + bounded four-arm comparison summaries)

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


## Latest packet: CRUSHR-FORMAT-03

- Added explicit experimental writer flag `--experimental-file-identity-extents` in `crushr-pack` for file-identity anchored extent records plus verified path-map records.
- Added strict salvage fallback provenance path `FILE_IDENTITY_EXTENT_PATH` after primary/redundant/checkpoint paths.
- Added strict path linkage verification: named recovery only when file-id + path digest + path-map record all verify.
- Added bounded comparison outputs: `file_identity_comparison_summary.json` and `file_identity_comparison_summary.md` (and compatibility emission of `experimental_comparison_summary.*`).
