# Snapshot Format

This is the canonical snapshot contract boundary for analysis-oriented JSON outputs.

## Scope

Snapshots exist for deterministic analysis, regression fixtures, and offline inspection.
They are not a recovery protocol and do not imply reconstruction capabilities.

## Common envelope

All snapshots include:

- `schema_version` (integer)
- `tool` (string)
- `tool_version` (string)
- `generated_at_utc` (RFC3339 string)
- `archive_fingerprint` (string)
- `payload` (object)

## Fingerprint rule

`archive_fingerprint` is the merge key across snapshots. Mismatched fingerprints must not be merged.

## `crushr-info` snapshot boundary

Expected payload families:

- archive summary
- tail/index structural observations
- optional file/block listing
- propagation-style impact reporting when requested

## `crushr-fsck` snapshot boundary

Expected payload families:

- verification result envelope
- corruption impact/blast-zone observations
- optional dump path references (raw compressed bytes always; decompressed bytes only when verified)

## Determinism and compatibility

- field semantics are versioned by schema
- deterministic ordering is required where schemas/contracts require ordered arrays
- schema changes must be additive or version-bumped

## Current phase alignment

- Phase 1 contracts are complete and active
- Phase 2 used these snapshots as experiment harness inputs
- Phase 2 execution/normalization/comparison are complete and frozen
- salvage planning output uses dedicated schemas (`crushr-salvage-plan.v1` legacy, `crushr-salvage-plan.v2` superseded, `crushr-salvage-plan.v3` active) and remains unverified research output


## Salvage experiment summary/analysis artifacts (research-only)

`crushr-lab-salvage` also emits compact deterministic experiment-level derived files from run metadata: `summary.json`, `summary.md`, `analysis.json`, and `analysis.md`. These remain research-only aggregates and are not canonical extraction snapshots or reconstruction semantics.

`analysis.json`/`analysis.md` provide deterministic grouped outcome/export/profile views and compact evidence rankings without inlining full run metadata or salvage-plan blobs.


`crushr-lab-salvage run-redundant-map-comparison` emits dedicated compact research artifacts: `comparison_summary.json` and `comparison_summary.md`. These are deterministic comparative aggregates (old-style vs new-style redundant map paths) and are not canonical extraction snapshots.


## Salvage harness input identity (research harness behavior)

`crushr-lab-salvage` discovers candidate archives by bounded on-disk identity checks rather than filename extension. A file is accepted when it either starts with `BLK3` magic or has a parseable `FTR4` footer whose referenced index region begins with `IDX3` magic. This allows valid `.crushr`, `.crs`, and extensionless archives while safely rejecting sidecars/unrelated files.


`crushr-lab-salvage run-experimental-resilience-comparison` emits `experimental_comparison_summary.json` and `experimental_comparison_summary.md` for bounded three-arm comparisons (old / redundant-map / experimental). These are deterministic research artifacts only.
