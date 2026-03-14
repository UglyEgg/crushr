# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-SALVAGE-03 complete (verified fragment export added to standalone salvage executable)

Recent completed packet: CRUSHR-SALVAGE-03 (deterministic verified block/extent research artifact export)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` now includes deterministic block-level verification states and optional verified fragment export (`--export-fragments`).

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation in CRUSHR-SALVAGE-03.
- Export outputs are research-only and explicitly labeled `UNVERIFIED_RESEARCH_OUTPUT`.

## Next actions

1. Keep strict extraction interfaces/semantics untouched.
2. Preserve deterministic salvage schema v2 output stability, including optional exported_artifacts references.
3. Keep Phase 2 corpus and frozen artifacts unchanged.
