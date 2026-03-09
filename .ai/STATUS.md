**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.14
- Fix iteration: 0

## Current Objective

Deliver the first strict extraction path for minimal v1 archives: clean extraction for regular files, deterministic refusal of files requiring corrupted blocks, and explicit failure on invalid archive structure.

## What Changed (since last Step)

- Added a new `crushr-extract` binary implementing strict minimal-v1 extraction over the current `open_archive_v1` + BLK3 scan + IDX3 decode path.
- Extraction now verifies payload hashes via existing `crushr-core::verify` and refuses only files whose required block IDs are in the corrupted set.
- Extraction keeps deterministic behavior by sorting paths and emitting stable refusal lines for skipped files.
- Added integration coverage in `crates/crushr-core/tests/minimal_pack_v1.rs` for clean single-file and tiny-directory extraction round trips, corrupted-payload selective refusal, invalid-footer failure, and deterministic stderr behavior.
- Updated `PROJECT_STATE.md` to reflect that strict minimal-v1 extraction now exists while salvage/metadata fidelity remains out of scope.

## What Remains (next actions)

1. Implement salvage-mode extraction and any recovery semantics as a separate packet.
2. Extend extraction support beyond minimal regular-file scope (symlinks/xattrs/dicts/append behavior) only when explicitly packeted.
3. Keep strict behavior integrity-first: never read/decompress bytes from corrupted required blocks.

## How to Build / Test (best known)

- `cargo test -p crushr-core --test minimal_pack_v1`
- `cargo test -p crushr --tests`

## Active constraints / gotchas

- Current strict extraction path supports only regular files from the minimal v1 pack layout.
- `crushr-extract` is intentionally strict-only in this packet (no salvage/hole filling/repair behavior).
- Existing legacy `crushr` monolith extract path remains separate from this bounded strict minimal-v1 tool path.
