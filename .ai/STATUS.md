**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: F
- Step: F.3
- Fix iteration: 2

## Current Objective

Implement first real BLK3 block-payload verification in `crushr-fsck` and surface corrupted block IDs via existing impact reporting.

## What Changed (since last Step)

- Added `crushr-core::verify` with read-only BLK3 block scanning (`scan_blocks_v1`) and payload-hash verification (`verify_block_payloads_v1`) over the blocks region up to `footer.blocks_end_offset`.
- Wired `crushr-fsck` snapshot generation to run block payload verification and populate `blast_radius.impact.corrupted_blocks` deterministically (while keeping `affected_files` empty until real IDX3 extent mapping is wired).
- Updated `crushr-fsck` binary integration to pass the archive reader into fsck snapshot generation so verification runs against real archive bytes.
- Added/updated tests for:
  - clean payload verification (`corrupted_blocks = []`)
  - payload-byte corruption (`corrupted_blocks` includes the damaged block id)
  - deterministic fsck JSON for identical bytes
  - preserved footer/tail structural corruption failure behavior.
- Verified with `cargo test -p crushr-core` and `cargo test --workspace`.

## What Remains (next actions)

1. Continue Phase F.3 with additional controlled fixture classes (many-small + mixed datasets).
2. Keep impact mapping read-only and add real IDX3 file/extent integration in a later bounded packet.
3. Prepare bounded Phase F.4 baseline-comparison packet after explicit approval.

## How to Build / Test (best known)

- `cargo test -p crushr-core`
- `cargo test --workspace`

## Active constraints / gotchas

- Existing workspace warnings remain in legacy areas (`crushr` and `crushr-tui`) but are non-blocking for current test gate.
- Impact mapping is still intentionally decompression-free and currently does not derive real file-level extents from IDX3 in this path.
