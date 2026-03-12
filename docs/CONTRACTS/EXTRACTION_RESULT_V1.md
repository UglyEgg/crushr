# Extraction Result Contract v1 (`crushr-extract --json`)

This document defines the current stable JSON result contract emitted by:

- `crushr-extract --json`

This contract formalizes the currently implemented minimal v1 extraction surface.

## Scope

This contract currently applies to:

- structurally valid minimal v1 archives
- regular-file extraction only
- one-block-per-file mapping

No speculative recovery, repair, or reconstruction behavior is part of this contract.

## Result envelopes

`crushr-extract --json` emits exactly one of the following envelopes.

### 1) Success envelope

Used when no files are refused.

```json
{
  "overall_status": "success",
  "maximal_safe_set_computed": true,
  "safe_files": [{"path": "..."}],
  "refused_files": [],
  "safe_file_count": 1,
  "refused_file_count": 0
}
```

### 2) Partial refusal envelope

Used when one or more files are refused because required blocks are corrupted.

```json
{
  "overall_status": "partial_refusal",
  "maximal_safe_set_computed": true,
  "safe_files": [{"path": "..."}],
  "refused_files": [{"path": "...", "reason": "corrupted_required_blocks"}],
  "safe_file_count": 1,
  "refused_file_count": 1
}
```

### 3) Error envelope

Used for structural/open/parse failures (exit code `2`).

```json
{
  "overall_status": "error",
  "error": "..."
}
```

The success/partial fields are not emitted for the error envelope.

## Field semantics

- `overall_status`
  - `success`: all extractable files extracted.
  - `partial_refusal`: some files extracted, some refused.
  - `error`: extraction did not produce a result set due to structural/open/parse failure.
- `maximal_safe_set_computed`
  - Always `true` for success/partial envelopes in current minimal v1 scope.
- `safe_files`
  - Deterministically ordered list of extracted files.
- `refused_files`
  - Deterministically ordered list of refused files.
  - `reason` currently has one stable value: `corrupted_required_blocks`.
- `safe_file_count` / `refused_file_count`
  - Exact counts of entries in `safe_files` / `refused_files`.

## Mode fields

`--mode` defaults to strict when omitted.

### Strict mode (`--mode strict` or default)

- `mode` is omitted.
- `salvage_decisions` is omitted.

### Salvage mode (`--mode salvage`)

- `mode` is present with value `"salvage"`.
- `salvage_decisions` is present as a deterministically ordered list with one entry per considered file:
  - `{"path": "...", "decision": "extracted_verified_extents"}`
  - `{"path": "...", "decision": "refused_corrupted_required_blocks"}`

Salvage mode remains integrity-first: files requiring corrupted blocks are refused.

## Deterministic ordering

For identical archive bytes and extraction flags:

- `safe_files` order is deterministic
- `refused_files` order is deterministic
- `salvage_decisions` order is deterministic (salvage mode only)

Current minimal v1 behavior orders file-path-based report entries lexicographically by stored path.
