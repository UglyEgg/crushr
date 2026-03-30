<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Inspecting archives

Use `crushr info` when you want to inspect an archive without extracting it.

Basic forms:

```bash
crushr info <archive.crs>
crushr info <archive.crs> --list
crushr info <archive.crs> --entry <logical/path>
crushr info <archive.crs> --find <query>
```

`info` is an introspection surface. It does not extract files, mutate the archive, or write to the filesystem.

## `crushr info`

This shows the archive contract and archive-level truth surface in human-readable form.

Typical sections include:

- archive path
- format version
- global flags
- preservation profile
- structural summary
- verification summary
- whether strict extraction is supportable

### Example

```bash
crushr info archive.crs
```

## What the sections mean

### Archive

This identifies the archive being inspected.

### Preservation

```text
Preservation
  profile               full
```

This tells you what the archive intended to preserve.

Profiles:
- `full`
- `basic`
- `payload-only`

### Structure

Structure rows summarize the archive container itself.

Typical fields may include:

- format version
- global flags
- entry count (when derivable)
- extent count
- dictionary count
- tail-frame presence

This is archive-level structure, not extraction output.

### Verification

Verification rows summarize whether the components required for inspection and strict extraction remain acceptable.

Typical fields may include:

- extent validity summary
- dictionary validity summary
- tail-frame validity summary
- `strict_extraction_supported`

`strict_extraction_supported` is the archive-level answer to:

> If I run `crushr extract` in strict/default mode, is the archive in a state that supports that path?

If the answer is `false`, inspection remains possible, but strict extraction is not supportable from current surviving evidence.

### Metadata visibility

Metadata rows use states such as:

- `present`
- `not present`
- `omitted by profile`

This is an important distinction.

| State | Meaning |
|---|---|
| `present` | The archive contains that metadata class |
| `not present` | The archive supports the class, but this archive did not contain any entries using it |
| `omitted by profile` | The archive profile intentionally excluded that class |

!!! warning "Do not confuse omission with damage"
    If `info` says `omitted by profile`, that is not corruption and it is not loss. The archive never promised to carry that metadata.

### Entry kinds

This section tells you what kinds of objects appear in the archive, such as:
- regular files
- directories
- symlinks
- sparse files
- special files

This is a summary, not a full listing.

## `crushr info --list`

This lists archive contents without extracting them.

```bash
crushr info archive.crs --list
```

### What it is based on

Listing is:
- metadata/index-driven
- fail-closed
- deterministic

If crushr cannot prove the listing, it does not guess.

### What it currently focuses on

`info --list` intentionally focuses on readable entry introspection rather than turning output into a low-level archive dump.

Use `--entry` for full detail on one path.
Use `--find` for deterministic name/path search.

## `crushr info --entry`

Use this for exact logical-path lookup without extraction.

```bash
crushr info archive.crs --entry src/main.rs
crushr info archive.crs --entry src/main.rs --json
```

`--entry` reports one deterministic truth surface for the requested path.

Typical fields include:

- logical path
- trust class
- payload verification status
- metadata completeness status
- extent count
- size bytes
- payload BLAKE3
- logical range (`start..end`) in both hex and decimal form
- identity source
- strict extraction supportability for that entry
- non-canonical reason when applicable

### Trust classes

`--entry` uses the canonical user-visible trust classes:

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

These classes reflect crushr's core distinction:

- payload integrity is independent from metadata completeness

### Payload BLAKE3

`payload_blake3` is reported when the entry's payload proof can be produced deterministically for this introspection surface.

If that proof cannot be produced for the entry in its current state, the value is reported deterministically as unavailable rather than guessed.

### Logical range

The logical range describes the entry in logical file space.

It is not an archive-physical address.

Human-readable output may show the range in both hexadecimal and decimal form. JSON output should be treated as the precise machine-readable source.

### Identity source

`identity_source` explains where the current entry identity came from, such as canonical metadata or another bounded non-speculative source.

This is especially useful when inspecting degraded or recovered entries.

### Not found behavior

If the path is missing:

- human mode reports an explicit not-found result
- JSON mode returns a deterministic not-found object rather than silently falling back to search behavior

## `crushr info --find`

Use this to search known logical identities without extraction.

```bash
crushr info archive.crs --find src
crushr info archive.crs --find .rs --json
```

Baseline matching is deterministic substring search with deterministic lexical ordering by logical path.

### Search scope

Search is bounded to stable known identities:

- `canonical`
- `metadata_degraded`
- `recovered_named`

Anonymous or invented identities are not searched.

`unrecoverable` entries may appear only when stable logical identity already exists in surviving archive evidence.

### Output behavior

By default, `--find` uses the standard crushr CLI presentation system.

Use `--json` when you want raw machine-readable output.

### Reserved forward-compatible flags

`--find` already reserves compatible CLI shape for future extension:

- `--find-mode substring` (currently the only supported mode)
- `--find-limit <n>`


## `crushr info --report propagation`

Use this when you need deterministic dependency and impact explanation without extraction.

```bash
crushr info archive.crs --json --report propagation
```

The propagation report separates:

- dependency graph (`nodes`, `edges`)
- currently detected corruption (`detected_corruption`)
- activated impact from current corruption (`activated_impacts`)
- per-entry dependency + impact explanation (`entry_impacts`)

Per-entry explanation includes:

- direct vs propagated dependencies
- bounded dependency reasons (for example `requires_index`, `requires_block_payload`)
- whether canonical extraction is currently blocked
- trust classes supported by current evidence only

The report is explanatory only. It does not imply repair behavior and does not make speculative survivability claims.

Any unsupported `--find-mode` value is a deterministic error.

## `info` versus `verify` versus extraction

This distinction matters.

### `info` tells you:
- what the archive contains
- what the archive profile promised
- what structure and verification state are visible
- what can be inspected without extraction
- whether strict extraction is supportable from current archive evidence

### `verify` tells you:
- whether required integrity and consistency checks support strict extraction
- whether inconsistency requires `crushr extract --recover`

### extraction tells you:
- what the target environment actually restored
- whether results stayed canonical
- whether anything became `metadata_degraded`, `recovered_named`, `recovered_anonymous`, or `unrecoverable`

That is why `info` may show:
- ACLs present
- SELinux labels present

while extraction may still yield non-canonical results if your target environment cannot apply them or required extraction conditions are not satisfied.

## Common workflow

Inspect first:

```bash
crushr info archive.crs
crushr info archive.crs --list
```

Then use targeted introspection if needed:

```bash
crushr info archive.crs --entry src/main.rs
crushr info archive.crs --find src
```

Then choose:
- `crushr verify` if you want strict-extraction viability and archive integrity assessment
- `crushr extract` for strict canonical restoration
- `crushr extract --recover` for explicit bounded recovery when strict extraction is not supportable

## Summary

Use `info` to answer:
- what profile was used?
- what kinds of entries are here?
- what metadata classes are included?
- what can be inspected without extraction?
- is strict extraction supportable?
- what do we know about this specific entry?
- which known entries match this search?

Use `verify` to answer:
- do current checks support strict extraction?

Use extraction to answer:
- what actually restored?

