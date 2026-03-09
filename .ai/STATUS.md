**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: F
- Step: F.3
- Fix iteration: 3

## Current Objective

Provide a deterministic command entrypoint for reproducing the current small corruption-validation experiment matrix and keeping research artifacts aligned.

## What Changed (since last Step)

- Added `crushr-lab run-first-experiment` to execute the current structural-validation loop end-to-end: fixture generation, `crushr-pack`, deterministic `byteflip` corruption, `crushr-info --json`, and `crushr-fsck --json` checks.
- Runner now writes artifacts deterministically into `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip` by default (or a caller-provided directory), and hard-fails on expectation mismatch (including expected nonzero exit code checks for corrupt archive info/fsck).
- Added integration coverage to assert the runner produces the expected artifact set and identifiers.
- Updated research docs to include the explicit runner invocation and limitations language, then refreshed the recorded artifact manifest with runner-managed command metadata.
- Verified via `cargo test -p crushr-lab` and `cargo test --workspace`.

## What Remains (next actions)

1. Continue Phase F.3 with additional controlled fixture classes (many-small + mixed datasets) when packeted.
2. Keep impact mapping read-only and add real IDX3 file/extent integration in a later bounded packet.
3. Prepare bounded Phase F.4 baseline-comparison packet after explicit approval.

## How to Build / Test (best known)

- `cargo run -q -p crushr-lab --bin crushr-lab -- run-first-experiment`
- `cargo test -p crushr-lab`
- `cargo test --workspace`

## Active constraints / gotchas

- Existing workspace warnings remain in legacy areas (`crushr` and `crushr-tui`) but are non-blocking for the current test gate.
- This runner intentionally targets only the existing single-experiment structural validation path; it is not a benchmark/comparison harness.
