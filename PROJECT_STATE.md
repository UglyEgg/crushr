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
- Current salvage baseline: `CRUSHR-FORMAT-01` redundant verified file-map metadata in tail-frame ledger (`LDG1`) for deterministic salvage fallback when the primary IDX3 mapping path is unusable, plus retained deterministic research harness behavior from CRUSHR-SALVAGE-07.

Canonical Phase 2 workspace root remains `PHASE2_RESEARCH/`.

## `CRUSHR-FORMAT-01` boundary

This packet adds bounded mapping survivability redundancy and must:

- keep standalone `crushr-salvage` separation
- keep `crushr-extract` strict-only and unchanged
- emit compact redundant file-map metadata in new archives via tail-frame ledger JSON
- allow salvage to use redundant mapping only when primary mapping is unusable and redundant metadata verifies fully
- reject partial/inconsistent redundant map metadata
- preserve deterministic salvage output and explicit research labeling
- avoid speculative reconstruction or guessed mappings
- never modify archives in place

Out of scope for this packet (unchanged):

- speculative stitching/reconstruction
- guessed byte emission
- mutation of archives
- integration into `crushr-extract`


`--resummarize <experiment_dir>` regenerates summary and analysis outputs from existing experiment artifacts and does not rerun salvage.


## `CRUSHR-SALVAGE-08` boundary

- `crushr-lab-salvage run-redundant-map-comparison --output <comparison_dir>` now runs a bounded deterministic old-vs-new salvage comparison for redundant map archives.
- Comparison emits compact `comparison_summary.json` and `comparison_summary.md` only (plus per-scenario rows embedded in JSON).
- The workflow is research-only and keeps strict extraction semantics unchanged.


## `CRUSHR-FORMAT-02` boundary

- Adds an explicit experimental writer path: `crushr-pack --experimental-self-describing-extents`.
- Experimental archives embed per-extent self-describing metadata blocks and distributed checkpoint snapshot blocks.
- Salvage precedence for experimental recovery remains strict and deterministic: primary IDX3, then verified checkpoint metadata, then verified self-describing extent metadata.
- Verification-only rule is unchanged: unverifiable metadata is rejected with no guessed mappings.
- Strict extraction (`crushr-extract`) remains unchanged.


## `CRUSHR-FORMAT-03` boundary

- Adds explicit experimental writer flag: `crushr-pack --experimental-file-identity-extents`.
- Emits per-extent file-identity metadata (`crushr-file-identity-extent.v1`) plus verified path-map records (`crushr-file-path-map.v1`).
- `crushr-salvage` fallback precedence now includes `FILE_IDENTITY_EXTENT_PATH` after primary/redundant/checkpoint paths, with strict path-linkage verification and deterministic refusal on inconsistencies.
- `crushr-lab-salvage run-file-identity-comparison --output <dir>` emits compact deterministic `file_identity_comparison_summary.json` and `.md` for bounded four-arm targeted runs.
- Distributed-hash and low-discrepancy placement strategies remain future research directions and are not active in this packet.
