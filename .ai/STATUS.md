**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.14
- Fix iteration: 2

## Current Objective

Add deterministic machine-readable JSON extraction result reporting to strict minimal-v1 extraction without changing strict extraction/refusal semantics.

## What Changed (since last Step)

- Added `--json` mode to `crushr-extract` for strict extraction with deterministic machine-readable reporting.
- JSON success/partial reports now include deterministic `extracted_files` and `refused_files` (`reason = "corrupted_required_blocks"`), while strict refusal behavior remains unchanged.
- Preserved refusal-exit policy semantics independently from JSON output: `success` keeps exit `0` on refusal, `partial-failure` returns exit `3` for refusal, and structural/open/parse failures remain exit `2`.
- Added integration coverage for clean success JSON, partial-refusal JSON under both refusal policies, structural failure JSON error envelope behavior, and deterministic serialization for identical inputs.
- Updated `docs/CONTRACTS/ERROR_MODEL.md`, `docs/ARCHITECTURE.md`, and `PROJECT_STATE.md` for the new strict extraction JSON mode.

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
