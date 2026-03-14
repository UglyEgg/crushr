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
- Current salvage baseline: `CRUSHR-SALVAGE-03` deterministic verified block/extent research-artifact export from standalone salvage output.

Canonical Phase 2 workspace root remains `PHASE2_RESEARCH/`.

## `CRUSHR-SALVAGE-03` boundary

This packet adds optional research artifact export and must:

- keep standalone `crushr-salvage` separation
- emit deterministic machine-readable salvage plan JSON with verification-backed block/file status
- avoid speculative reconstruction or guessed mappings
- never modify archives
- label outputs as unverified research output

Out of scope for this packet:

- speculative stitching/reconstruction
- guessed byte emission
- mutation of archives
- integration into `crushr-lab`
