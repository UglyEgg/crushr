# .ai/HANDOFF.md

## Immediate next packet

Next salvage packet (TBD): build on CRUSHR-SALVAGE-03 export model without adding reconstruction.

## First actions for a fresh instance

1. Read startup order from `AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` show CRUSHR-SALVAGE-03 complete and salvage remains separate from strict extraction.
3. Keep strict extraction (`crushr-extract`) semantics unchanged.
4. Treat salvage as a separate experimental executable only.
5. Run workspace gates (`fmt`, `test`, `clippy`).

## Gotchas

- Do not add salvage modes/flags to `crushr-extract`.
- Do not claim salvage output is canonical extraction.
- Salvage output must be labeled unverified research output.
- CRUSHR-SALVAGE export is research-only: no reconstruction, no guessed bytes, no canonical extraction claims.

## Recently completed

- Phase 2 execution corpus is frozen.
- Phase 2 normalization artifacts are frozen.
- Phase 2 comparison/ranking artifacts are complete and frozen.

## Completed CRUSHR-SALVAGE-03 outputs

- verified block-analysis states in `crushr-salvage` JSON
- schema version bump to salvage-plan v2
- deterministic tests for decode/hash/dictionary/file downgrade behavior

- optional `--export-fragments` artifact emission with deterministic ordering
- block/extent/full-file export gating only from content-verified data
- salvage-plan v2 `exported_artifacts` references when export mode is enabled
