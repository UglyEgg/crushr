# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-FORMAT-01 complete (redundant verified file-map metadata + strict salvage fallback)

Recent completed packet: CRUSHR-FORMAT-01 (LDG1 redundant file-map metadata emission + salvage-plan v3 fallback provenance)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` includes deterministic block-level verification states and optional verified fragment export (`--export-fragments`).
- `crushr-pack` now emits a compact redundant verified file-map ledger (`crushr-redundant-file-map.v1`) in LDG1 for new archives.
- `crushr-salvage` uses primary IDX3 mappings first and only falls back to redundant map metadata when IDX3 is unusable and redundant metadata verifies fully.
- salvage output schema is now `crushr-salvage-plan.v3` with `redundant_map_analysis` and per-file `mapping_provenance`.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation in CRUSHR-SALVAGE-03.
- Export outputs are research-only and explicitly labeled `UNVERIFIED_RESEARCH_OUTPUT`.

## Next actions

1. Keep strict extraction interfaces/semantics untouched.
2. Preserve deterministic salvage schema v2 output stability, including optional exported_artifacts references.
3. Keep Phase 2 corpus and frozen artifacts unchanged.
4. Keep salvage experiment outputs labeled as unverified research output and deterministic in archive ordering/IDs.
5. Preserve `--resummarize <experiment_dir>` behavior as summary+analysis regeneration without rerunning salvage.
6. Preserve strict all-or-nothing validation for redundant map fallback metadata.
7. Keep `crushr-lab-salvage` salvage binary resolution deterministic (sibling binary / test env / explicit `CRUSHR_SALVAGE_BIN`) and independent of global PATH.
