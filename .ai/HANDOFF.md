# .ai/HANDOFF.md

## Immediate next packet

Phase 2.1 — controlled corruption matrix manifest/schema.

## First actions for a fresh instance

1. Read startup order from `../AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` both point to Phase 2.1 as next packet.
3. Keep strict extraction + integrity-first thesis unchanged.
4. Implement manifest/schema only; do not expand into harness execution logic unless packet requests it.
5. Run full workspace fmt/test/clippy gates.

## Gotchas

- Do not reintroduce recovery/salvage language in active docs.
- Keep authority order aligned across AGENTS/bootstrap/guardrails/state/.ai files.


## Recently completed

- CRUSHR-CLEANUP-2.0-C finished: active schemas are now strict contracts and validated with JSON Schema in integration tests.
- CRUSHR-CLEANUP-2.0-D finished: extraction report assembly/refusal classification moved to `crushr-core::extraction`; `crushr-info` structural-failure report assembly now uses shared propagation helper.
