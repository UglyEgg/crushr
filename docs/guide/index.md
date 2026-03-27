<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# User guide

This section is the practical side of crushr.

It is written for Linux users who want to understand:
- which command to run
- what the output means
- what is normal
- what should make them stop and look closer

| Guide goal | What it means |
|---|---|
| Audience | Linux users who are comfortable at the command line but do not want to reverse-engineer the format to use it correctly |
| Tone | Practical, direct, and explicit |
| Scope | Day-to-day usage, command behavior, output interpretation, and common workflows |

!!! note "Start here"
    If you are new to crushr, read this section before the technical reference. The reference explains the architecture. This guide explains how to use the tool without tripping over the trust model.

## What crushr does

crushr is a Linux-first archive tool with three priorities:

1. preserve filesystem meaning as honestly as possible
2. verify what can still be trusted
3. recover what can still be proven without guessing

That makes it feel similar to `tar + zstd` in daily use, but it is stricter about integrity and much clearer about degraded results.

## Read this section in order

1. [Getting started](index.md)
2. [Packing archives](pack.md)
3. [Extracting archives](extract.md)
4. [Verifying archives](verify.md)
5. [Inspecting archives](info.md)

## Quick start

Create an archive:

```bash
crushr pack ./my-folder -o archive.crs
```

Verify it:

```bash
crushr verify archive.crs
```

Extract it:

```bash
crushr extract archive.crs -o ./out
```

Inspect it without extracting:

```bash
crushr info archive.crs
crushr info archive.crs --list
```

## Core ideas you need before using crushr

### Preservation profiles

Every archive is created with a preservation profile.

| Profile | Meaning | Good default use |
|---|---|---|
| `full` | Preserve everything crushr knows how to preserve | Backups, system trees, serious archival use |
| `basic` | Preserve structure and common filesystem behavior, but omit system-heavy metadata | Sharing projects, moving data between machines |
| `payload-only` | Preserve only file content and logical tree reconstruction | Simple content transport |

!!! tip "Default behavior"
    If you do not specify a profile, crushr uses `full`.

### Extraction trust classes

crushr does not collapse all outcomes into "worked" and "failed".

| Result class | Meaning |
|---|---|
| `canonical` | Restored exactly as required by the archive's preservation profile |
| `metadata_degraded` | File data and identity are correct, but some required metadata could not be restored |
| `recovered_named` | Data was recovered and the name/path is evidence-backed, but not fully canonical |
| `recovered_anonymous` | Data was recovered, but original identity could not be proven |
| `unrecoverable` | crushr could not recover the entry |

!!! warning "Important distinction"
    `metadata_degraded` is not the same thing as corruption inside the archive. It often means the archive contained the right metadata, but the target system could not apply it because of permissions, filesystem support, or security policy.

### `info` versus `extract`

`crushr info` tells you what the archive **contains**.

`crushr extract` tells you what actually **happened during restoration**.

Those are not the same question.

Examples:
- `info` can tell you that ACLs are present in the archive
- only extraction can tell you whether those ACLs were actually restored successfully

## Common workflow examples

### Full-fidelity archive

```bash
crushr pack ./project -o project.crs --preservation full
crushr verify project.crs
crushr extract project.crs -o ./restore
```

### Lighter archive for sharing

```bash
crushr pack ./project -o project.crs --preservation basic
```

### Inspect before extracting

```bash
crushr info archive.crs
crushr info archive.crs --list
```

## What is expected versus not expected

### Expected

- clear profile labeling
- explicit preservation scope
- explicit warnings when metadata restoration fails
- non-canonical results called out directly

### Not expected

- silent guessing
- hidden downgrade from canonical to "close enough"
- fake restoration of unsupported entry types
- "it probably worked" behavior

## Where to go next

- [Packing archives](pack.md)
- [Extracting archives](extract.md)
- [Verifying archives](verify.md)
- [Inspecting archives](info.md)
