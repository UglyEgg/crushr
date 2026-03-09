**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.14
- Fix iteration: 3

## Current Objective

Harden strict extraction outcome handling with typed internal classification while preserving existing strict extraction/refusal/JSON semantics.

## What Changed (since last Step)

- Refactored `crushr-extract` to classify outcomes and errors using explicit internal enums (success, partial refusal, usage error, structural/open/parse failure) rather than string matching.
- Exit-code mapping now comes from typed classification helpers: usage=1, structural/open/parse=2, success=0, and partial refusal maps to 0 or 3 per `--refusal-exit` policy.
- Preserved existing `--json` output schema/behavior and refusal semantics; added a focused unit test for typed exit-code mapping.

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
