<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Packing archives

`crushr pack` creates a new archive.

Basic form:

```bash
crushr pack <input>... -o <archive.crs>
```

## Common examples

Archive one directory with full preservation:

```bash
crushr pack ./project -o project.crs
```

Archive multiple inputs:

```bash
crushr pack ./docs ./scripts ./README.md -o bundle.crs
```

Choose a preservation profile:

```bash
crushr pack ./project -o project.crs --preservation basic
```

Pick a compression level:

```bash
crushr pack ./project -o project.crs --level 10
```

## Important options

| Option | Meaning |
|---|---|
| `-o, --output <archive>` | Output archive path |
| `--preservation <profile>` | `full`, `basic`, or `payload-only` |
| `--level <n>` | Compression level |
| `--silent` | Emit concise machine-friendly output |

## Preservation profiles in practice

### `full`

Use when you want the archive to preserve Linux metadata and special entry behavior as completely as crushr currently supports.

That includes things like:
- ownership
- xattrs
- ACLs
- SELinux labels
- capabilities
- sparse files
- special file types

Use this for:
- backups
- system snapshots
- serious restore workflows

### `basic`

Use when you want a more portable archive and do not care about system-heavy metadata.

It keeps:
- files
- directories
- symlinks
- hard links
- mode
- mtime
- sparse semantics

It omits:
- xattrs
- ownership
- ACLs
- SELinux labels
- capabilities
- FIFOs
- device nodes

### `payload-only`

Use when you only care about file content and paths.

This is intentionally the lightest contract.

!!! note "Why this matters"
    The preservation profile becomes part of the archive contract. Later, `info` and extraction both interpret the archive through that profile.

## What warnings during pack mean

crushr may warn and omit entries when the selected profile does not permit them.

Examples:
- FIFO encountered while using `basic`
- device node encountered while using `payload-only`

This is expected behavior. crushr will not silently flatten those entries into something else.

!!! warning "No quiet fabrication"
    If an entry kind is excluded by the chosen profile, crushr warns and omits it. It does not pretend a FIFO is a regular file or a device node is harmless data.

## What normal successful output looks like

Interactive output typically shows:
- target archive
- preservation profile
- progress phases
- final size and timing summary

In `--silent` mode, expect a concise single-line style instead of the full presentation.

## Common mistakes

### Forgetting the profile
If you care about Linux metadata, do not assume `basic` or `payload-only` will keep it. Use `full`.

### Picking `payload-only` for a system tree
That is usually the wrong choice. It is good for content transport, not faithful Linux restoration.

### Treating warnings as decoration
Warnings during pack usually mean the resulting archive contract is lighter than the source tree. Read them.

## Summary

Use `pack` when you want to create the archive.

Choose the preservation profile based on what you need later:
- `full` for fidelity
- `basic` for lighter sharing
- `payload-only` for content-only packaging
