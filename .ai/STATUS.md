# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.11
- Fix iteration: 0

## Current Objective

Implement the first minimal v1 `crushr-pack` output path that writes BLK3 blocks plus a valid tail frame/FTR4 readable by `open_archive_v1`, `crushr-info --json`, and `crushr-fsck --json`.

## What Changed (since last Step)

- Added a new `crushr-pack` binary (`crates/crushr/src/bin/crushr-pack.rs`) implementing a bounded minimal v1 pack path:
  - accepts one or more file/dir inputs and `-o/--output`
  - writes one BLK3 block per file using zstd payloads and BLAKE3 payload/raw hashes
  - writes IDX3 bytes via existing `crushr::index_codec::encode_index`
  - assembles/writes tail frame with `crushr_format::tailframe::assemble_tail_frame` (no DCT1/LDG1 yet)
- Added `crushr-format` dependency to `crates/crushr/Cargo.toml` so `crushr-pack` reuses canonical BLK3/tailframe helpers instead of duplicating format logic.
- Added integration tests (`crates/crushr-core/tests/minimal_pack_v1.rs`) that validate:
  - single-file pack -> `open_archive_v1` + `crushr-info --json` + `crushr-fsck --json` success
  - tiny-directory pack -> deterministic archive bytes for identical inputs
  - produced archive footer region contains `FTR4` and parsed IDX3/tail metadata is valid

## What Remains (next actions)

1. Advance to Step 0.13 blast-zone dump implementation (detect+isolate path expansion).
2. Extend open path recovery behavior to locate the last valid tail frame when trailing/corrupt tails exist.
3. Expand fsck verification from metadata path into block-level integrity checks when authorized.

## How to Build / Test (best known)

- `cargo fmt --all`
- `cargo test -p crushr-core`
- `cargo test -p crushr --no-run`
