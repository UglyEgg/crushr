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
