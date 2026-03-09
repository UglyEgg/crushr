**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.14
- Fix iteration: 1

## Current Objective

Deliver the first strict extraction path for minimal v1 archives: clean extraction for regular files, deterministic refusal of files requiring corrupted blocks, and explicit failure on invalid archive structure.

## What Changed (since last Step)

- Added a policy-controlled refusal exit flag to `crushr-extract`: `--refusal-exit <success|partial-failure>` with default `success`.
- Strict extraction behavior is preserved: unaffected files still extract, refused files are still reported deterministically on stderr, and refusal reporting remains stable.
- Added integration coverage for both refusal policies on clean archives, selective-refusal archives, and structurally invalid archives.
- Updated contract/state docs to record exit code `3` semantics for policy-requested partial extraction refusal signaling in strict extraction.

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
