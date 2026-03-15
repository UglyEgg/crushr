# crushr Development Status

Current Phase: Phase 3 — Salvage Planning Research Boundary

Current Step: CRUSHR-FORMAT-05 complete (self-identifying payload blocks + repeated verified path checkpoints + bounded format05 comparison)

Recent completed packet: CRUSHR-FORMAT-05 (payload-block identity fallback + repeated verified path checkpoints + format05 bounded comparison outputs)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- `crushr-pack` emits redundant verified file-map ledger metadata (`crushr-redundant-file-map.v1`) in LDG1 for new archives.
- `crushr-salvage` uses primary IDX3 mapping first, then redundant metadata only when primary mapping is unusable and redundant metadata fully verifies.
- `crushr-lab-salvage` now supports `run-redundant-map-comparison` for deterministic old-style vs new-style targeted comparison runs and emits compact `comparison_summary.json` and `comparison_summary.md`.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation.
- Comparison workflow is bounded (24 deterministic scenarios), storage-conscious, and does not rerun the full Phase 2 matrix.

## Next actions

1. Preserve strict extraction interfaces/semantics untouched.
2. Keep redundant map fallback strict all-or-nothing and deterministic.
3. Preserve deterministic comparison ordering/classification and compact grouped metrics outputs.
4. Keep Phase 2 corpus and frozen artifacts unchanged.
5. Keep file-identity extent path experimental and opt-in only.

## Latest packet: CRUSHR-FORMAT-03-f2

- Added bounded `crushr-pack --help` support and usage text documenting both experimental writer flags.
- Preserved and validated the experimental writer contract used by `crushr-lab-salvage` comparison workflows (`--experimental-self-describing-extents`, `--experimental-file-identity-extents`).
- Added focused packer regression tests for help discoverability and experimental flag acceptance/archive emission.



## Latest packet: CRUSHR-FORMAT-04

- Added experimental distributed bootstrap anchors (`crushr-bootstrap-anchor.v1`) for file-identity archives, placed across non-tail regions and verified via strict metadata block verification.
- Added deterministic fallback salvage path that scans verified metadata when footer/index are unavailable, enabling bounded header/index/tail-loss recovery without speculative reconstruction.
- Added strict anonymous verified naming fallback when path maps are missing: `anonymous_verified/file_<file_id>.bin` with `FILE_IDENTITY_EXTENT_PATH_ANONYMOUS` provenance.
- Added `bootstrap_anchor_analysis` to salvage plan v3 output and updated schema to keep deterministic contract validation.
- Added `run-format04-comparison` command and `format04_comparison_summary.json/.md` outputs while keeping legacy file-identity summary aliases for compatibility.


## Latest packet: CRUSHR-FORMAT-05

- Added explicit `crushr-pack --experimental-self-identifying-blocks` writer mode with per-payload identity records (`crushr-payload-block-identity.v1`) and repeated verified path checkpoints (`crushr-path-checkpoint.v1`).
- Extended strict salvage fallback with deterministic payload-block identity planning and named/anonymous verified recovery provenance (`PAYLOAD_BLOCK_IDENTITY_PATH`, `PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS`).
- Added bounded `crushr-lab-salvage run-format05-comparison` workflow and required compact outputs: `format05_comparison_summary.json` and `format05_comparison_summary.md`.
