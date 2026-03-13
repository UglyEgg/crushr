# crushr Development Status

Current Phase: Phase 2 — Comparative Corruption Research

Current Step: 2.1 pre-trial audit milestone complete (CRUSHR-P2-TRIAL-READY-01 pre-trial readiness gate + deterministic audit checks)

Recent completed packet: CRUSHR-P2-TRIAL-READY-01 (added deterministic pre-trial readiness audit command, machine-readable audit report, and operator-flow updates)

## Current truth

- Phase 1 is complete.
- Phase 2.1 packets CRUSHR-P2.1-A/B/C/D are complete: `crushr-lab` now has typed Phase 2 manifest/scenario structures, deterministic locked-core scenario enumeration (2700 runs), deterministic dataset fixture builders (`smallfiles`, `mixed`, `largefiles`), deterministic inventory/provenance emission, typed archive build execution records for `crushr`, `zip`, `tar+zstd`, `tar+gz`, and `tar+xz`, a locked corruption injection engine (`bit_flip`, `byte_overwrite`, `zero_fill`, `truncation`, `tail_damage`) with locked targets/magnitudes/seeds and deterministic mutation provenance, and a manifest-driven execution runner that emits typed `RawRunRecord` evidence plus completeness audits over missing/duplicate/mismatched scenario IDs.
- Cleanup packets CRUSHR-CLEANUP-2.0-C and CRUSHR-CLEANUP-2.0-D are complete.
- Cleanup packet CRUSHR-P2-CLEAN-01 is complete: deleted packet-era scaffold experiment commands/helpers/tests and reduced `crushr-lab` main dispatch/help surface to `corrupt`, `write-phase2-manifest`, `build-phase2-foundation`, and `run-phase2-execution`.
- Cleanup packet CRUSHR-P2-CLEAN-02 is complete: replaced hand-authored command prose (`observed_command`) in `RawRunRecord.execution_metadata` with structured invocation metadata (`tool_kind`, executable, argv, cwd, exit status, stdout/stderr artifact paths) captured directly from the real `Command` invocation before/after execution.
- Cleanup packet CRUSHR-P2-CLEAN-03 is complete: established `PHASE2_RESEARCH/` as canonical Phase 2 workspace (`methodology/`, `manifests/`, `generated/`, `normalized/`, `summaries/`, `whitepaper_support/`), moved lock guidance to `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md`, and migrated Phase 2 default output roots out of `docs/`.
- Cleanup packet CRUSHR-P2-CLEAN-07 is complete: slimmed `crushr-lab` `main.rs` into command dispatch only; moved usage/workspace helpers into `cli.rs`, moved `corrupt` argument parsing/log emission into `phase2_corruption`, and moved `write-phase2-manifest` / `build-phase2-foundation` / `run-phase2-execution` command orchestration into their owning modules.
- Cleanup packet CRUSHR-P2-CLEAN-08 is complete: removed stale control-doc path/process residue, added a concise Phase 2 operator path in `PHASE2_RESEARCH/README.md`, and aligned active docs on pre-trial audit as the next required milestone before Phase 2.2.

- Added `crushr-lab run-phase2-pretrial-audit` command that validates manifest schema/shape, locked 2700-scenario matrix semantics (ordering/comparator set/ID completeness), tool availability, required support files, and writable `PHASE2_RESEARCH/*` roots before execution.
- Pre-trial audit now emits deterministic machine-readable JSON (`phase2_pretrial_readiness_v1`) with pass/fail, failing checks, failure categories, tool/output-root status, locked matrix summary, and compact readiness summary; default snapshot path is `PHASE2_RESEARCH/outputs/pretrial_audit.json`.
- Added focused audit tests covering serialization/readiness summary, duplicate scenario-ID detection, missing required support file detection, and unwritable output-root checks.
- Updated canonical operator flow in `PHASE2_RESEARCH/README.md` to include pre-trial audit command and output locations as a first-class gate between manifest generation and foundation/execution runs.
- Active machine-readable schemas are now tightened contracts for: `crushr-info` snapshot, `crushr-fsck` snapshot, `crushr-impact` report, extraction result, and propagation graph.
- Integration tests now perform real JSON Schema instance-vs-schema validation for active outputs.

## Active constraints

- Minimal v1 scope: regular files, one block per file.
- No speculative recovery/reconstruction/repair.
- Strict extraction semantics remain canonical.
- Workspace cargo config now sets rustc flag `-A unknown-lints` so required command `cargo clippy --workspace --all-targets -- -D warning` does not emit command-line unknown-lint noise.
- `schemas/crushr-impact.v1.schema.json` remains active as a nested contract dependency (used by fsck blast-radius payload and direct impact report typing); no obsolete schema was deleted in this packet.

## Next action

Begin Phase 2.2 comparative mapping/reporting using `PHASE2_RESEARCH/generated/execution/` raw evidence once audit-gated execution inputs are prepared.
