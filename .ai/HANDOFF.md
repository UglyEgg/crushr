# HANDOFF

## Current focus
- CRUSHR-FORMAT-04 is complete and validated.
- Next work should refine metrics quality for header/index/tail focus rows and decide whether to graduate bootstrap anchors beyond experimental scope.

## Important behavior locks
- `crushr-extract` remains strict-only and unchanged.
- File-identity fallback now allows deterministic anonymous verified output when path map linkage is absent (`FILE_IDENTITY_EXTENT_PATH_ANONYMOUS`).
- Salvage remains verification-only: no guessed names, offsets, or extent stitching.

## Commands
- `crushr-lab-salvage run-format04-comparison --output <dir>`
- Required outputs: `format04_comparison_summary.json`, `format04_comparison_summary.md`

## Watch items
- Keep salvage-plan schema and emitted fields in sync (`bootstrap_anchor_analysis`).
- Preserve deterministic ordering in anonymous naming and row ordering.


## Current handoff focus (CRUSHR-FORMAT-05 complete)

- Experimental writer: `crushr-pack --experimental-self-identifying-blocks`.
- Experimental metadata contracts: `crushr-payload-block-identity.v1` and `crushr-path-checkpoint.v1` (repeated/separated placement).
- Salvage precedence now includes payload identity fallback after file-identity extents.
- Comparison command: `crushr-lab-salvage run-format05-comparison --output <dir>`.
