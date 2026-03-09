**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: F
- Step: F.4
- Fix iteration: 0

## Current Objective

Deliver the first bounded competitor-comparison experiment scaffold with deterministic artifacts and honest supported/deferred target handling.

## What Changed (since last Step)

- Added `crushr-lab run-competitor-scaffold` to generate a tiny shared fixture, build per-target archives, apply deterministic byteflip corruption, and record build/observe command metadata in a stable manifest.
- Implemented scaffold target handling for `crushr`, `zip`, and `tar+zstd` with environment detection and graceful deferral; added explicit deferred handling for `7z` so unavailable tools do not report false success.
- Added integration tests for scaffold artifact/manifest structure, deferred-target honesty, and docs-to-artifact reference alignment.
- Added and recorded the first scaffold artifact set at `docs/RESEARCH/artifacts/crushr_p0s13f0_competitor_scaffold_byteflip/`.
- Updated research docs to describe scaffold purpose, environment assumptions, and supported vs deferred targets without broad comparative claims.

## What Remains (next actions)

1. Extend bounded comparison cases to additional corruption models only when packeted.
2. Add tar+zstd and 7z execution paths when tool availability and stability constraints are explicitly approved.
3. Keep comparative claims in `docs/RESEARCH/RESULTS.md` limited to recorded scaffold status until a full matrix packet is completed.

## How to Build / Test (best known)

- `cargo run -q -p crushr-lab --bin crushr-lab -- run-competitor-scaffold`
- `cargo test -p crushr-lab`
- `cargo test -p crushr-core --test first_corruption_experiment`
- `cargo test --workspace`

## Active constraints / gotchas

- `tar+zstd` currently defers in this environment because `zstd` is unavailable in `PATH`.
- `7z` currently defers in this environment because `7z/7za` is unavailable in `PATH`.
- This work is scaffold-only and intentionally does not claim benchmark-quality comparative outcomes.
