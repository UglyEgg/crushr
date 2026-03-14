## Update (2026-03-13): CRUSHR-P2-TRIAL-READY-01 complete

- Added `crushr-lab run-phase2-pretrial-audit` command and `phase2_audit` module.
- Audit validates locked matrix (2700 scenarios + axes), manifest/schema shape, duplicate IDs, truthful tool availability, required support files, and writable `PHASE2_RESEARCH/` output roots.
- Machine-readable artifact path: `PHASE2_RESEARCH/generated/audit/phase2_pretrial_audit.json`.
- Operator path in `PHASE2_RESEARCH/README.md` now includes audit immediately after manifest generation.
- Next action: begin Phase 2.2 normalized mapping/reporting work.

# .ai/HANDOFF.md

## Immediate next packet

Phase 2.3 comparative reporting/aggregation on top of frozen manifest + deterministic execution outputs and normalized evidence artifacts under `PHASE2_RESEARCH/`.

## First actions for a fresh instance

1. Read startup order from `../AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` show CRUSHR-P2-EXEC-04 complete and Phase 2.3 comparative reporting as the next active step.
3. Keep strict extraction + integrity-first thesis unchanged.
4. Treat `PHASE2_RESEARCH/` as the canonical Phase 2 workspace and `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md` as the active lock document.
5. Consume `crushr-lab` Phase 2 manifest/scenario enumeration plus the deterministic foundation builder (`build-phase2-foundation`) and execution runner (`run-phase2-execution`) as canonical Phase 2 inputs; do not mutate locked matrix semantics.
6. Run full workspace fmt/test/clippy gates.

## Gotchas

- Do not reintroduce recovery/salvage language in active docs.
- Keep authority order aligned across AGENTS/bootstrap/guardrails/state/.ai files.
- Do not point active control docs back at removed `docs/RESEARCH/*` or repo-local task-packet paths.

## Recently completed

- CRUSHR-P2-EXEC-01 finished: froze canonical trial contract artifacts at `PHASE2_RESEARCH/manifest/phase2_manifest.json` (2700 deterministic scenarios) and `PHASE2_RESEARCH/manifest/manifest_summary.json` (count + locked axis lists).
- CRUSHR-P2-EXEC-02 finished: generated canonical Phase 2 fixture datasets under `PHASE2_RESEARCH/datasets/`, baseline archive corpus under `PHASE2_RESEARCH/baselines/`, and `PHASE2_RESEARCH/foundation/foundation_report.json` including archive hashes/sizes/file counts and deterministic generation confirmation.
- CRUSHR-P2-EXEC-03A finished: `run-phase2-execution` now resolves source archive paths from `foundation_report.json` against workspace root (absolute paths preserved), limits `artifact_dir` to generated run outputs, and defaults to canonical execution paths under `PHASE2_RESEARCH/trials`.
- CRUSHR-P2-EXEC-03B finished: enriched Phase 2 execution evidence model (`raw_run_records.json`) with first-class scenario/invocation/path/result fields, replaced broken generic version strings with truthful tool-version observations, upgraded `execution_report.json` with matrix/exit/json/tool/completeness summaries, and added execution-report/raw-record schema contracts plus focused tests.
- CRUSHR-P2-EXEC-04 finished: added `run-phase2-normalization`, emitted `PHASE2_RESEARCH/results/normalized_results.json` + `normalization_summary.json`, documented deterministic normalization rules, added normalization schemas, and added focused normalization tests (classification/stage/diagnostic/order/schema).
- CRUSHR-P2-EXEC-06A finished: execution now runs extraction commands for all Phase 2 formats and records deterministic recovery evidence (`extraction_output_dir`, `recovery_report_path`, `recovery_accounting`), normalization now emits file/byte recovery fields + blast radius + recovery evidence strength + richer summary aggregates, and raw/normalized summary schemas/tests were updated accordingly.
- CRUSHR-P2-PRETRIAL-DET-01 finished: `crushr-pack` baseline archive generation is deterministic (stable file ordering, normalized mode/mtime metadata, deterministic zstd flags), with focused tests for byte-identical repeated runs and stable index ordering.
- CRUSHR-P2-CLEAN-07: reduced `crushr-lab/src/main.rs` to thin top-level dispatch; moved command-specific parsing/execution into `cli.rs`, `phase2_corruption.rs`, `phase2_manifest.rs`, `phase2_foundation.rs`, and `phase2_runner.rs` with behavior-preserving defaults/help.
- CRUSHR-CLEANUP-2.0-C finished: active schemas are now strict contracts and validated with JSON Schema in integration tests.
- CRUSHR-CLEANUP-2.0-D finished: extraction report assembly/refusal classification moved to `crushr-core::extraction`; `crushr-info` structural-failure report assembly now uses shared propagation helper.
- CRUSHR-P2.1-A finished: typed manifest/scenario model, deterministic scenario IDs and enumeration (2700), schema file, and validation tests are in place.
- CRUSHR-P2.1-B finished: deterministic dataset fixtures, inventories/provenance, typed archive build records, and reproducibility tests are in place.
- CRUSHR-P2.1-C finished: locked corruption classes/targets/magnitudes/seeds are implemented with deterministic provenance output and determinism tests.
- CRUSHR-P2-CLEAN-01 finished: removed obsolete scaffold experiment command paths and associated helper sediment from `crushr-lab` main so only the Phase 2 core pipeline remains.
- CRUSHR-P2-CLEAN-02 finished: Phase 2 execution raw records now store structured invocation provenance from actual command execution and no longer store narrative command strings.
- CRUSHR-P2-CLEAN-03 finished: canonical Phase 2 workspace now lives at `PHASE2_RESEARCH/`; defaults for manifest/foundation/execution outputs no longer target `docs/RESEARCH/artifacts/`, and lock guidance now lives in `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md`.
- CRUSHR-P2-CLEAN-04 finished: Phase 2 locked comparator set is now `crushr`, `zip`, `tar+zstd`, `tar+gz`, `tar+xz`; 7z/lzma is removed from manifest/schema/foundation/runner/docs/tests and core scenario count is now 2700.
- CRUSHR-P2-CLEAN-04 follow-up finished: required clippy invocation diagnostic is resolved via workspace cargo config (`-A unknown-lints`).
- CRUSHR-P2-CLEAN-06 finished: added shared `phase2_domain` types/helpers and removed duplicate manifest/foundation/runner domain enums plus dataset/format map shims so Phase 2 uses one canonical model.
- CRUSHR-P2.1-D finished: `crushr-lab run-phase2-execution` now executes locked manifest scenarios against Phase 2 foundation archives and emits deterministic raw evidence under `PHASE2_RESEARCH/trials`; completeness auditing detects missing, duplicate, and mismatched scenario IDs and writes `completeness_audit.json`.

## Next expected packet

Use `PHASE2_RESEARCH/results/{normalized_results.json,normalization_summary.json}` to build Phase 2.3 comparative reporting outputs (tables/aggregates) without modifying the frozen execution corpus.
