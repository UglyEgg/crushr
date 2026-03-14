# .ai/HANDOFF.md

## Immediate next packet

CRUSHR-SALVAGE-01: implement standalone `crushr-salvage` deterministic salvage planning with machine-readable JSON output.

## First actions for a fresh instance

1. Read startup order from `AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` show Phase 2 execution/normalization/comparison complete and CRUSHR-SALVAGE-01 active.
3. Keep strict extraction (`crushr-extract`) semantics unchanged.
4. Treat salvage as a separate experimental executable only.
5. Run workspace gates (`fmt`, `test`, `clippy`).

## Gotchas

- Do not add salvage modes/flags to `crushr-extract`.
- Do not claim salvage output is canonical extraction.
- Salvage output must be labeled unverified research output.
- CRUSHR-SALVAGE-01 is plan-only: no fragment emission, no payload carving directories, no reconstruction.

## Recently completed

- Phase 2 execution corpus is frozen.
- Phase 2 normalization artifacts are frozen.
- Phase 2 comparison/ranking artifacts are complete and frozen.

## Expected CRUSHR-SALVAGE-01 outputs

- `crushr-salvage` executable
- plan JSON emitted by `crushr-salvage` (`--json-out` optional path)
- salvage-plan schema + focused deterministic tests
