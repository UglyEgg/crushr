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
- Phase 2 uses these snapshots as experiment harness inputs
- next required milestone: Phase 2 pre-trial audit over `PHASE2_RESEARCH/` flow
- next packet after audit: Phase 2.2 cross-format comparison and normalized result mapping
