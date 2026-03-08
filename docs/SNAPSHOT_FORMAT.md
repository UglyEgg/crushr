# Snapshot Format

This document defines the **normative** JSON snapshot contracts consumed by `crushr-tui`.

Snapshots exist to support:

- Offline inspection and sharing (without requiring direct archive access)
- Deterministic repro cases and regression tests (golden snapshots)
- A stable data model boundary between tools and the TUI

Snapshots are not an IPC protocol; tools still call libraries in-process. Snapshots are
serialized outputs intended for storage and later analysis.

## Common envelope

All snapshot documents MUST include the following top-level fields:

- `schema_version` (integer): snapshot schema version.
- `tool` (string): tool name (e.g., `crushr-info`, `crushr-fsck`).
- `tool_version` (string): tool/package version.
- `generated_at_utc` (string): RFC 3339 timestamp.
- `archive_fingerprint` (string): stable identifier used to validate snapshots belong to the same archive.
- `payload` (object): tool-specific snapshot content.

### Archive fingerprint

`archive_fingerprint` MUST be derived from archive content that is stable across paths and machines.

Recommended definition (v1):

- `BLAKE3( "crushr:fingerprint:v1" || last_valid_ftr4.footer_hash || last_valid_ftr4.index_hash )`

If no valid footer exists, tools should emit a fingerprint of `"unknown"` and the TUI MUST
refuse to merge snapshots with mismatched fingerprints.

## `crushr-info` snapshot

Tool: `crushr-info --json`

### Payload shape (v1)

- `summary`: archive-level counts and configuration summary
- `tail_frames`: list of discovered tail frames (valid and invalid) with parse/verify status
- `dicts`: dictionary table entries for the selected tail frame (if present)
- `files`: optional file/index listing (may be gated by flags)
- `blocks`: optional block map (may be gated by flags)

Notes:

- `crushr-info` MUST NOT modify archives.
- When `files`/`blocks` are omitted, the TUI should still render summary/tail/dict views.

## `crushr-fsck` snapshot

Tool: `crushr-fsck --json`

### Payload shape (v1)

- `verify`: structural verification results and reason codes
- `blast_radius`: computed blast zones (bad blocks, invalid tail frames) and impacted files
- `salvage_plan`: optional (what would be extractable under strict vs salvage)
- `dump_paths`: optional paths when `--dump-blast-zone` was used

### Blast-zone dump contract

When `--dump-blast-zone DIR` is used:

- Tools MUST dump **raw compressed payload bytes** for impacted blocks.
- Tools MUST dump **decompressed bytes only when verification passes**.
- Tools MUST NOT attempt best-effort decompression of corrupted blocks.

The dump directory MUST include a machine-readable index describing what was emitted.

## TUI merge rules

When multiple snapshots are loaded:

- Snapshots with matching `archive_fingerprint` MAY be merged.
- Snapshots with differing `archive_fingerprint` MUST NOT be merged.
- The TUI should present mismatched datasets side-by-side with an explicit warning.

