# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.9
- Fix iteration: 2

## Current Objective

Continue the integrity-first implementation path by wiring real archive parsing in `crushr-core` after completing tail frame assembly helpers in `crushr-format`.

## What Changed (since last Step)

- Fixed `crushr/src/dict.rs` compile errors blocking `cargo check -p crushr` (imported `WalkDir`, repaired function boundary, and updated zstd `from_continuous` call signature).
- Added tracked sample-size accounting for dictionary training progress path and explicit empty-sample rejection before dict training.

## What Remains (next actions)

1. Wire real archive parsing in `crushr-core` for open/info/fsck.
2. Emit real `crushr-fsck --json` impact reports from parsed archives.
3. Run and record the first end-to-end corruption experiment.

## How to Build / Test (best known)

- `cargo test -p crushr-format`
- `cargo clippy -p crushr-format --all-targets -- -D warnings`
