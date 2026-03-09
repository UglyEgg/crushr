**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: F
- Step: F.3
- Fix iteration: 1

## Current Objective

Keep the first recorded corruption-experiment baseline stable while restoring workspace hygiene (`cargo test --workspace`) and normalizing CLI failure exit-code behavior for parse/open/structural failures.

## What Changed (since last Step)

- Repaired the pre-existing workspace test failure in `crates/crushr/tests/mvp.rs` by removing brittle crate-local `target/debug` assumptions and resolving the binary path from the workspace root after an explicit `cargo build -p crushr --bin crushr`.
- Normalized `crushr-info` error exit behavior to match `crushr-fsck` for this baseline:
  - usage/argument errors => exit `1`
  - archive open + parse/structural/validation failures => exit `2`
- Added/extended binary-path tests in `crates/crushr-core/src/snapshot.rs` to enforce normalized exit codes for:
  - structural footer corruption (`crushr-info` => `2`)
  - missing archive open failure (`crushr-info` and `crushr-fsck` => both `2`)
- Clarified this normalized policy in `docs/CONTRACTS/ERROR_MODEL.md`.
- Verified `cargo test --workspace` now passes.

## What Remains (next actions)

1. Continue Phase F.3 with additional controlled fixture classes (many-small + mixed datasets).
2. Extend deterministic corruption model coverage incrementally (truncate/tail overwrite) without changing format contracts.
3. Prepare bounded Phase F.4 baseline-comparison packet after explicit approval.

## How to Build / Test (best known)

- `cargo test -p crushr --test mvp`
- `cargo test -p crushr-core`
- `cargo test --workspace`

## Active constraints / gotchas

- Existing workspace warnings remain in legacy areas (`crushr` and `crushr-tui`) but are non-blocking for current test gate.
- Current experiment evidence remains intentionally structural-only (no decompression/salvage/repair claims).
