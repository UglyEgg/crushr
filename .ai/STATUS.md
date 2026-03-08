# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.9
- Fix iteration: 3

## Current Objective

Continue the integrity-first implementation path by wiring real archive parsing in `crushr-core` after completing tail frame assembly helpers in `crushr-format`.

## What Changed (since last Step)

- Hardened `crates/crushr-format/src/tailframe.rs` with checked component slicing to avoid unchecked integer casts when deriving DCT1/LDG1 sub-slices from footer offsets.
- Simplified LDG1 trailing-byte validation to reuse a single parser pass (`Cursor::position`) while keeping strict full-consumption semantics.
- Verified `crushr-format` is test-clean and clippy-clean with `-D warnings`.

## What Remains (next actions)

1. Wire real archive parsing in `crushr-core` for open/info/fsck.
2. Emit real `crushr-fsck --json` impact reports from parsed archives.
3. Run and record the first end-to-end corruption experiment.

## How to Build / Test (best known)

- `cargo test -p crushr-format`
- `cargo clippy -p crushr-format --all-targets -- -D warnings`
