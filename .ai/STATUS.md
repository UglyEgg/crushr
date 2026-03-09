**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.15
- Fix iteration: 0

## Current Objective

Add explicit salvage-mode extraction for minimal v1 archives while preserving strict mode behavior and deterministic reporting/exit semantics.

## What Changed (since last Step)

- Added explicit extraction mode selection in `crushr-extract` via `--mode <strict|salvage>` (default `strict`), preserving strict behavior as-is when salvage is not selected.
- Implemented salvage-mode reporting in deterministic JSON form using `mode: "salvage"` and ordered `salvage_decisions` entries (`extracted_verified_extents` or `refused_corrupted_required_blocks`) while keeping existing `extracted_files` / `refused_files` contracts.
- Kept integrity-first refusal rules unchanged: files with corrupted required blocks are never decompressed and remain refused in both strict and salvage modes.
- Added focused integration coverage for salvage-mode clean archives, partial corruption extraction/refusal behavior, refusal-exit interaction, and deterministic JSON output.

## What Remains (next actions)

1. Extend salvage support beyond minimal regular-file scope only via explicit packets (symlinks/xattrs/dicts/append scenarios).
2. Implement pending Step 0.13 blast-zone dump implementation (currently still unchecked in phase plan).
3. Continue Phase F claim validation and artifact-backed result updates.

## How to Build / Test (best known)

- `cargo test -p crushr-core --test minimal_pack_v1`
- `cargo test -p crushr --bin crushr-extract`

## Active constraints / gotchas

- Minimal-v1 extraction path currently supports regular files only.
- Salvage mode is extraction-decision/reporting mode, not reconstruction/repair; corrupted required blocks are deterministically refused.
- Legacy monolith `crushr` extract path remains separate from `crushr-extract` strict/salvage packet.
