# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-SALVAGE-05 complete (deterministic experiment-level salvage summaries + `--resummarize` added to `crushr-lab-salvage`)

Recent completed packet: CRUSHR-SALVAGE-05 (deterministic compact experiment summaries and non-rerunning resummarize mode)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` includes deterministic block-level verification states and optional verified fragment export (`--export-fragments`).
- `crushr-lab-salvage` orchestrates deterministic salvage experiments over `.crushr` sets, writes experiment manifests/run metadata, and emits compact `summary.json` + `summary.md` outputs.

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
5. Preserve `--resummarize <experiment_dir>` behavior as summary-only regeneration without rerunning salvage.
