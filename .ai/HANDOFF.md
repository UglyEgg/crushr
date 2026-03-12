# .ai/HANDOFF.md

## Immediate next packet

Phase 2.2 — cross-format comparison execution harnessing from Phase 2 manifest scenarios.

## First actions for a fresh instance

1. Read startup order from `../AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` both show Phase 2.1-A/2.1-B/2.1-C complete and Phase 2.2 next.
3. Keep strict extraction + integrity-first thesis unchanged.
4. Consume `crushr-lab` Phase 2 manifest/scenario enumeration plus the deterministic foundation builder (`build-phase2-foundation`) as canonical execution inputs; do not mutate locked matrix semantics.
5. Run full workspace fmt/test/clippy gates.

## Gotchas

- Do not reintroduce recovery/salvage language in active docs.
- Keep authority order aligned across AGENTS/bootstrap/guardrails/state/.ai files.


## Recently completed

- CRUSHR-CLEANUP-2.0-C finished: active schemas are now strict contracts and validated with JSON Schema in integration tests.
- CRUSHR-CLEANUP-2.0-D finished: extraction report assembly/refusal classification moved to `crushr-core::extraction`; `crushr-info` structural-failure report assembly now uses shared propagation helper.


- CRUSHR-P2.1-A finished: typed manifest/scenario model, deterministic scenario IDs and enumeration (2160), schema file, and validation tests are in place.

- CRUSHR-P2.1-B finished: deterministic dataset fixtures, inventories/provenance, typed archive build records, and reproducibility tests are in place.

- CRUSHR-P2.1-C finished: locked corruption classes/targets/magnitudes/seeds are implemented with deterministic provenance output and determinism tests.


## 2026-03-12 update (CRUSHR-P2.1-D)
- Added `crushr-lab run-phase2-execution` to execute locked manifest scenarios against Phase 2 foundation archives and emit deterministic raw evidence under `docs/RESEARCH/artifacts/phase2_execution`.
- Raw records are typed (`RawRunRecord`) and include required scenario fields, exit code, stdout/stderr paths, optional JSON result path, tool version, and execution metadata.
- Completeness auditing now detects missing, duplicate, and mismatched scenario IDs and writes `completeness_audit.json`.
- Next expected packet should map raw records into normalized comparative results (Phase 2.2).
