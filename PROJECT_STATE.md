# crushr — Project State

crushr is an integrity-first archival container for deterministic corruption analysis.

## Canonical product thesis

- Integrity-first archive design.
- Strict extraction semantics: extract verified-safe content, refuse unsafe content.
- Deterministic corruption impact reporting and experimentability.
- No speculative recovery, reconstruction, or automatic repair.

## Active toolchain

Active tools:

- `crushr-pack`
- `crushr-info`
- `crushr-fsck`
- `crushr-extract`
- `crushr-lab`

Active libraries:

- `crushr-format`
- `crushr-core`

`crates/crushr/` remains in-repo but is not authority for project direction.

## Archive/verification model

Minimal v1 layout:

`BLK3 blocks -> (optional DCT1) -> IDX3 -> FTR4`

Verification progression:

`FTR4 -> tail frame -> IDX3 -> BLK3 payload/hash checks -> per-file impact`

## Current implementation scope

- regular files only
- one block per file
- deterministic strict extraction reporting (`safe_files` / `refused_files`)

## Phase status

- Phase 1: complete.
- Cleanup packet: complete.
- Phase 2: active engineering focus.
- Next packet: Phase 2.1 controlled corruption matrix manifest/schema.
