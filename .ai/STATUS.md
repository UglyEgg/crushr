# crushr Development Status

Current Phase: Phase 2 — Comparative Corruption Research

Active Objective:

Phase transition complete after CRUSHR-1.1-B hardening closeout.

Current truth:

- Phase 1 Step 1.1 is complete and hardened.
- `crushr-info --json --report propagation` now includes bounded structural-current-state reporting even when the normal open path fails.
- Structural corruption of required structures (`FTR4`, tail frame, `IDX3`) is now reported as current-state impacts when detectable from fallback inspection.
- Propagation schema/test coverage now validates nested object shape, enum stability, unknown-field rejection behavior, and deterministic ordering expectations.
- `crushr-extract --mode salvage` has been removed; extract is strict-only.

Active constraints:

- Minimal v1 scope remains regular files + one block per file.
- No speculative recovery, reconstruction, or repair.

Next actions:

- Start Phase 2 Step 2.1 packet (controlled corruption matrix manifest/schema).


Update 2026-03-12 (CRUSHR-CLEANUP-2.0-A):

- Removed legacy recovery/salvage surfaces from the `crushr` crate CLI and API (`recover`/`salvage` command and API options/functions).
- Deleted legacy recovery implementation module (`crates/crushr/src/recovery.rs`).
- Removed `salvage_plan` from `crushr-core` fsck snapshot model and aligned tests/docs/spec text to strict integrity-first semantics.

Current constraints unchanged:

- Minimal v1 scope remains regular files + one block per file.
- No speculative recovery, reconstruction, or repair workflows.

Next actions:

- Continue Phase 2 Step 2.1 packet (controlled corruption matrix manifest/schema).
