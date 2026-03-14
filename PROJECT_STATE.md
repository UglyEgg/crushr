# PROJECT_STATE

## Product thesis (active)

crushr is an integrity-first archive system.

Non-negotiable baseline:

- strict extraction only for `crushr-extract`
- deterministic verification and reporting
- no speculative recovery/reconstruction in canonical extraction

## Active tool boundary

- `crushr-pack` — archive creation
- `crushr-info` — archive inspection/reporting
- `crushr-fsck` — strict verification/corruption analysis
- `crushr-extract` — strict verified extraction with deterministic refusal reporting
- `crushr-lab` — controlled research harness
- `crushr-salvage` — separate experimental salvage-planning executable (unverified research output only)

`crushr-salvage` must not change or weaken `crushr-extract` semantics.

## Current implementation scope

- regular files only
- one block per file
- deterministic strict extraction reporting (`safe_files` / `refused_files`)

## Phase status (authoritative summary)

- Phase 1: complete.
- Phase 2 execution matrix: complete and frozen.
- Phase 2 normalization: complete and frozen.
- Phase 2 comparison/ranking analysis: complete and frozen.
- Current salvage baseline: `CRUSHR-SALVAGE-06` deterministic research harness (`crushr-lab-salvage`) that orchestrates standalone salvage runs and emits experiment manifests, per-run metadata, compact summaries (`summary.json`, `summary.md`), and compact grouped analysis views (`analysis.json`, `analysis.md`).

Canonical Phase 2 workspace root remains `PHASE2_RESEARCH/`.

## `CRUSHR-SALVAGE-06` boundary

This packet adds deterministic salvage experiment orchestration and must:

- keep standalone `crushr-salvage` separation
- emit deterministic experiment output layout with machine-readable salvage plan/metadata JSON plus compact deterministic experiment summaries and grouped analysis views
- avoid speculative reconstruction or guessed mappings
- never modify archives
- label outputs as unverified research output

Out of scope for this packet (unchanged):

- speculative stitching/reconstruction
- guessed byte emission
- mutation of archives
- integration into `crushr-extract`


`--resummarize <experiment_dir>` regenerates summary and analysis outputs from existing experiment artifacts and does not rerun salvage.
