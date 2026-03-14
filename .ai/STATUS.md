# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-SALVAGE-02 complete (verified block analysis added to standalone salvage planning executable)

Recent completed packet: CRUSHR-SALVAGE-02 (deterministic candidate verification + verification-backed file salvageability)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-salvage` now includes deterministic block-level verification states (header/bounds/decode/raw-hash/dictionary dependency).

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, or archive mutation in CRUSHR-SALVAGE-02.
- CRUSHR-SALVAGE-02 remains plan-only (JSON planning output; no fragment emission).

## Next actions

1. Keep strict extraction interfaces/semantics untouched.
2. Preserve deterministic salvage schema v2 output stability in follow-up packets.
3. Keep Phase 2 corpus and frozen artifacts unchanged.
