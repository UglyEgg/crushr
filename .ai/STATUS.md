# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.10
- Fix iteration: 2

## Current Objective

Implement the first real read-only archive open + `crushr-info --json` snapshot emission path via `crushr-core`.

## What Changed (since last Step)

- Added `crushr-core::open::open_archive_v1` over `ReadAt + Len`, using the real `crushr-format` FTR4/tail-frame parser path.
- Replaced `InfoSnapshotV1` skeleton `serde_json::Value` placeholders with typed snapshot metadata structures and added `info_snapshot_from_open_archive`/envelope serialization helpers.
- Added a minimal `crushr-info` binary (`crates/crushr/src/bin/crushr-info.rs`) supporting `crushr-info <archive> --json` and emitting a populated info snapshot envelope.
- Added `crushr-core` unit tests for:
  1. minimal archive snapshot emission,
  2. DCT1+LDG1 presence reporting,
  3. deterministic JSON serialization,
  4. clean invalid-archive failure.
- Hardened `crushr-core` info snapshot test coverage to assert parsed JSON envelope/payload fields (`schema_version`, `tool_version`, fingerprint presence, summary/tail offsets) instead of substring checks.
- Confirmed current `crushr` pack path still emits legacy archives, so a `crushr-info` CLI end-to-end JSON test remains deferred until pack is wired to the v1 tail-frame format.

- Extended `OpenArchiveV1` + `InfoSummaryV1` with explicit footer metadata (`footer_offset`, `footer_len`, `has_footer`) and mapped those fields into real `InfoSnapshotV1` emission.
- Added a `crushr-core` test that writes synthetic valid v1 archive bytes to a temp file and exercises the real `crushr-info --json` binary path via `cargo run -p crushr --bin crushr-info -- <file> --json`.

## What Remains (next actions)

1. Extend open path to locate the last valid tail frame when trailing/corrupt tails exist (recovery scanning behavior).
2. Wire real `crushr-fsck --json` verify/impact snapshot emission from parsed archives.
3. Run and record the first end-to-end corruption experiment.

## How to Build / Test (best known)

- `cargo test -p crushr-core`
- `cargo test -p crushr --no-run`
