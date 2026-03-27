<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Inspecting archives

Use `crushr info` when you want to understand an archive without extracting it.

Basic forms:

```bash
crushr info <archive.crs>
crushr info <archive.crs> --list
```

## `crushr info`

This shows the archive contract and structure in human-readable form.

Typical sections include:

- preservation profile
- metadata visibility
- entry-kind summary
- format/structural information

### Example

```bash
crushr info archive.crs
```

## What the sections mean

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

### Metadata

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

`info --list` intentionally focuses on regular file listing readability.

Non-regular entry kinds are still represented at the archive-summary level and through scope/context notes, rather than turning the list output into a cluttered dump.

That is by design.

## `info` versus extraction outcomes

This is one of the most important distinctions in crushr.

### `info` tells you:
- what the archive contains
- what the archive profile promised
- which metadata classes are in scope

### extraction tells you:
- what the target environment successfully restored
- whether results stayed canonical
- whether anything became `metadata_degraded`

That is why `info` may show:
- ACLs present
- SELinux labels present

while extraction may still yield metadata-degraded results if your system cannot apply them.

## Common workflow

Inspect first:

```bash
crushr info archive.crs
crushr info archive.crs --list
```

Then choose:
- `crushr verify` if you want archive-level integrity
- `crushr extract` for strict restoration
- `crushr extract --recover` for best available proven recovery

## Summary

Use `info` to answer:
- what profile was used?
- what kinds of entries are here?
- what metadata classes are included?
- what can be listed without extraction?

Use extraction to answer:
- what actually restored?
