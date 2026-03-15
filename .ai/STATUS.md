# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-SALVAGE-07 complete (hardened salvage binary resolution + format-identity archive discovery)

Recent completed packet: CRUSHR-SALVAGE-07 (deterministic salvage binary resolution without PATH dependency + format-identity archive discovery for `.crushr`/`.crs`/extensionless archives)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` includes deterministic block-level verification states and optional verified fragment export (`--export-fragments`).
- `crushr-lab-salvage` orchestrates deterministic salvage experiments over archives discovered by on-disk identity (not filename extension), writes experiment manifests/run metadata, and emits compact `summary.json` + `summary.md` outputs and grouped `analysis.json` + `analysis.md` views.

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
6. Keep `crushr-lab-salvage` salvage binary resolution deterministic (sibling binary / test env / explicit `CRUSHR_SALVAGE_BIN`) and independent of global PATH.
