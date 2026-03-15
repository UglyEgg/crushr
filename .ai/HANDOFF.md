# .ai/HANDOFF.md

## Immediate next packet

Next packet (TBD): evaluate CRUSHR-FORMAT-03 targeted outcomes and decide whether to iterate file-identity extent design or park the branch as negative evidence.


CRUSHR-SALVAGE-08 added `crushr-lab-salvage run-redundant-map-comparison --output <comparison_dir>` with compact deterministic `comparison_summary.json`/`.md` outputs and per-scenario improvement classes.

## First actions for a fresh instance

1. Read startup order from `AI_BOOTSTRAP.md`.
2. Confirm `STATUS.md` and `PHASE_PLAN.md` show CRUSHR-SALVAGE-07 complete and salvage remains separate from strict extraction.
3. Keep strict extraction (`crushr-extract`) semantics unchanged.
4. Treat salvage as a separate experimental executable only.
5. Run workspace gates (`fmt`, `test`, `clippy`).

## Gotchas

- Do not add salvage modes/flags to `crushr-extract`.
- Do not claim salvage output is canonical extraction.
- Salvage output must be labeled unverified research output.
- CRUSHR-SALVAGE export is research-only: no reconstruction, no guessed bytes, no canonical extraction claims.

## Recently completed

- Phase 2 execution corpus is frozen.
- Phase 2 normalization artifacts are frozen.
- Phase 2 comparison/ranking artifacts are complete and frozen.

## Completed CRUSHR-SALVAGE-03 outputs

- verified block-analysis states in `crushr-salvage` JSON
- schema version bump to salvage-plan v2
- deterministic tests for decode/hash/dictionary/file downgrade behavior

- optional `--export-fragments` artifact emission with deterministic ordering
- block/extent/full-file export gating only from content-verified data
- salvage-plan v2 `exported_artifacts` references when export mode is enabled


## Completed CRUSHR-SALVAGE-04 outputs

- deterministic `crushr-lab-salvage` orchestration over `.crushr` archive directories
- stable per-run directories with salvage plan capture + run metadata
- top-level `experiment_manifest.json` with run ordering and unverified research label
- optional delegated fragment export integration via `crushr-salvage --export-fragments`


## Completed CRUSHR-SALVAGE-05 outputs

- compact deterministic experiment summaries at `<experiment_dir>/summary.json` and `<experiment_dir>/summary.md`
- stable run-level outcome categories: `NO_VERIFIED_EVIDENCE`, `ORPHAN_EVIDENCE_ONLY`, `PARTIAL_FILE_SALVAGE`, `FULL_FILE_SALVAGE_AVAILABLE`
- `--resummarize <experiment_dir>` mode to regenerate summaries from existing manifest/run metadata without rerunning salvage


## Completed CRUSHR-SALVAGE-06 outputs

- compact deterministic grouped analysis files at `<experiment_dir>/analysis.json` and `<experiment_dir>/analysis.md`
- grouped outcome/export-mode/profile views plus deterministic evidence rankings
- `--resummarize <experiment_dir>` regenerates `summary.json`/`summary.md` and `analysis.json`/`analysis.md` from existing experiment artifacts


## Completed CRUSHR-SALVAGE-07 outputs

- deterministic `crushr-salvage` binary resolution in `crushr-lab-salvage` without relying on global PATH (sibling binary lookup + test env + explicit override)
- archive input discovery switched from extension-only filtering to bounded format-identity checks (`BLK3` leading magic or valid `FTR4` + `IDX3` tail markers)
- deterministic harness coverage for `.crushr`, `.crs`, extensionless archives, non-archive sidecar rejection, ordering stability, and binary resolution failures


## Completed CRUSHR-FORMAT-01 outputs

- `crushr-pack` writes redundant file-map metadata (`crushr-redundant-file-map.v1`) into LDG1
- `crushr-salvage` validates redundant mapping metadata and uses it only when IDX3 mapping is unavailable/invalid
- salvage-plan schema bumped to v3 with `redundant_map_analysis` and per-file `mapping_provenance`
- focused regression tests cover fallback success, fallback rejection, backward compatibility, and determinism


## Completed CRUSHR-FORMAT-02 outputs

- explicit experimental writer flag in `crushr-pack`
- self-describing extent + distributed checkpoint metadata emission in experimental archives
- deterministic salvage fallback provenance for checkpoint/self-describing metadata
- bounded three-arm comparison command with compact experimental summary outputs


## Completed CRUSHR-FORMAT-03 outputs

- explicit `crushr-pack --experimental-file-identity-extents` writer path
- per-extent file-identity records (`crushr-file-identity-extent.v1`) + verified path-map (`crushr-file-path-map.v1`)
- strict salvage fallback provenance `FILE_IDENTITY_EXTENT_PATH` after primary/redundant/checkpoint paths
- bounded four-arm comparison artifacts `file_identity_comparison_summary.json`/`.md`
