# .ai/STATUS.md

**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: 0
- Step: 0.10
- Fix iteration: 1

## Current Objective

Consolidate the repo into a Codex-ready full workspace artifact and continue the integrity-first implementation path.

## What Changed (since last Step)

- Added contracts and research documentation scaffolding.
- Added project-control documents (`PROJECT_STATE.md`, `REPO_SNAPSHOT.md`, task/review templates).
- Added `crushr-lab` corruption harness skeleton.
- Added `crushr-core::impact` model implementing decompression-free impact enumeration.
- Added impact report schema and CI skeleton.

## What Remains (next actions)

1. Implement tail frame assembly helpers in `crushr-format`.
2. Wire real archive parsing in `crushr-core` for open/info/fsck.
3. Emit real `crushr-fsck --json` impact reports from parsed archives.
4. Run and record the first end-to-end corruption experiment.

## How to Build / Test (best known)

- `cargo test --workspace`
