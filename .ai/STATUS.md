# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.12
- Fix iteration: 0

## Current Objective

Implement the first real read-only `crushr-fsck --json` impact-reporting path via the existing open parser + impact model.

## What Changed (since last Step)

- Added a real `crushr-fsck` binary (`crates/crushr/src/bin/crushr-fsck.rs`) supporting `crushr-fsck <archive> --json`, reusing `open_archive_v1` and returning deterministic exit codes (`1` usage/flags, `2` structural corruption/open failures).
- Extended `crushr-core::snapshot` with typed fsck report mapping (`FsckVerifyV1`, `FsckBlastRadiusV1`, `FsckSnapshotV1`) and helpers (`fsck_clean_report`, `fsck_snapshot_from_open_archive`, `fsck_envelope_from_open_archive`).
- Wired fsck clean success output through `ImpactReportV1` (`schema_version=1`, empty corrupted blocks/affected files when metadata path validates).
- Added fsck tests covering valid JSON emission, deterministic fsck JSON for identical bytes, corrupted footer failure, and corrupted IDX3 hash failure using synthetic archives + temp files.

## What Remains (next actions)

1. Extend open path to locate the last valid tail frame when trailing/corrupt tails exist (recovery scanning behavior).
2. Expand fsck verification beyond metadata/tail structure into block-level integrity once scope authorizes it.
3. Run and record the first end-to-end corruption experiment.

## How to Build / Test (best known)

- `cargo fmt --all`
- `cargo test -p crushr-core`
- `cargo test -p crushr --no-run`
