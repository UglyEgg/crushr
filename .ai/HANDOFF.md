# .ai/HANDOFF.md

## Immediate next packet

Next salvage packet (TBD): build on CRUSHR-SALVAGE-02 verification model without adding reconstruction/export.

## First actions for a fresh instance

1. Read startup order from `AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` show CRUSHR-SALVAGE-02 complete and salvage remains separate from strict extraction.
3. Keep strict extraction (`crushr-extract`) semantics unchanged.
4. Treat salvage as a separate experimental executable only.
5. Run workspace gates (`fmt`, `test`, `clippy`).

## Gotchas

- Do not add salvage modes/flags to `crushr-extract`.
- Do not claim salvage output is canonical extraction.
- Salvage output must be labeled unverified research output.
- CRUSHR-SALVAGE remains plan-only in current scope: no fragment emission, no payload carving directories, no reconstruction.

## Recently completed

- Phase 2 execution corpus is frozen.
- Phase 2 normalization artifacts are frozen.
- Phase 2 comparison/ranking artifacts are complete and frozen.

## Completed CRUSHR-SALVAGE-02 outputs

- verified block-analysis states in `crushr-salvage` JSON
- schema version bump to salvage-plan v2
- deterministic tests for decode/hash/dictionary/file downgrade behavior
