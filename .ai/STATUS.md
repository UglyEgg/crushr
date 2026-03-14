# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-SALVAGE-01 in progress (standalone deterministic salvage planning executable)

Recent completed packet: CRUSHR-P2-ANALYSIS-01 (deterministic cross-format comparison tables + rankings)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` is the active workstream as a separate experimental executable.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, or archive mutation in CRUSHR-SALVAGE-01.
- CRUSHR-SALVAGE-01 is plan-only (JSON planning output; no fragment emission).

## Next actions

1. Finish `crushr-salvage` deterministic planning + schema/tests.
2. Keep strict extraction interfaces/semantics untouched.
3. Keep Phase 2 corpus and frozen artifacts unchanged.
