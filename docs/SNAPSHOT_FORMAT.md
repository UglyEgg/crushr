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


`crushr-lab-salvage run-format10-pruning-comparison` emits `format10_comparison_summary.json` and `format10_comparison_summary.md` for bounded four-arm metadata-pruning comparisons (`full_current_experimental`, `payload_only`, `payload_plus_manifest`, `payload_plus_path`). These are deterministic research artifacts that audit metadata necessity and archive-size overhead; they are not canonical extraction snapshots.

`crushr-lab-salvage run-format11-extent-identity-comparison` emits `format11_comparison_summary.json` and `format11_comparison_summary.md` for bounded distributed extent-identity comparisons (`payload_only`, `payload_plus_manifest`, `full_current_experimental`, `extent_identity_only`). `extent_identity_only` is anonymous-by-design for this packet: local extent identity omits path/name and focuses on structure/integrity fields.


`crushr-lab-salvage run-file-identity-comparison` emits `file_identity_comparison_summary.json` and `file_identity_comparison_summary.md` for bounded four-arm targeted comparisons (old / redundant / format-02 experimental / format-03 file-identity extents). These are deterministic research artifacts only.


`crushr-lab-salvage run-format04-comparison` emits `format04_comparison_summary.json` and `format04_comparison_summary.md` (and compatibility aliases under file-identity summary names). These are deterministic research comparison aggregates only and are not extraction snapshots.


`crushr-lab-salvage run-format05-comparison` emits `format05_comparison_summary.json` and `format05_comparison_summary.md` for bounded five-arm targeted comparisons (old / redundant / format-02 experimental / format-03 file-identity / format-05 payload-block identity). These are deterministic research comparison aggregates only and are not extraction snapshots.


## Experimental metadata checkpoint placement (FORMAT-08)
- Optional packer surface: `--placement-strategy <fixed_spread|hash_spread|golden_spread>`.
- Valid only with experimental graph-supporting metadata writer flags (path checkpoints and/or file-manifest checkpoints).
- This setting influences checkpoint copy placement for `crushr-path-checkpoint.v1` and `crushr-file-manifest-checkpoint.v1` metadata only.
- Payload semantics, payload ordering, and strict extraction behavior are unchanged.
