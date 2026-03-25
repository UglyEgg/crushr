<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr

[![Policy Gate](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml/badge.svg)](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![REUSE status](https://api.reuse.software/badge/github.com/UglyEgg/crushr)](https://api.reuse.software/info/github.com/UglyEgg/crushr)

A Linux-first archive tool built to replace `tar + zstd` for workflows that care about preservation fidelity, honest recovery, and what actually happens when archives fail.

Its core design question is simple:

> When an archive is damaged, what can still be proven and recovered without guessing?

That is the foundation.

Everything else — tar-class preservation, archive inspection, future layout visibility, benchmarking, and later reproducible archive modes — sits on top of that principle rather than replacing it.

## Why crushr exists

Most archive tooling assumes the happy path:

- metadata is intact
- structure is trustworthy
- extraction either works or fails cleanly
- archives remain black boxes until you unpack them

crushr was built around the opposite assumption:

- things fail
- data gets damaged
- partial truth still matters
- and the tool should say clearly what is canonical, what is degraded, what was recovered, and what is lost

The goal is not just to compress files.

The goal is to preserve Linux filesystem state honestly, recover what can be proven, and avoid pretending certainty where none exists.

## What crushr is now

crushr has moved beyond a format experiment.

Today, the project provides:

- archive creation with `crushr pack`
- strict verification with `crushr verify`
- strict extraction with `crushr extract`
- recovery-aware extraction with `crushr extract --recover`
- archive inspection with `crushr info`
- pre-extraction archive listing with `crushr info --list`
- a shared, product-grade CLI surface across the canonical commands

crushr archives are identified by crushr format markers, not by filename extension.

The canonical default extension is:

- `.crs`

If no extension is supplied for `pack -o`, `.crs` is appended automatically.

## Canonical command surface

The canonical public tool surface is:

- `crushr pack`
- `crushr verify`
- `crushr extract`
- `crushr extract --recover`
- `crushr info`
- `crushr info --list`
- `crushr about`

Thin wrapper binaries are retained for convenience and map to the canonical `crushr` commands:

- `crushr-pack` → `crushr pack ...`
- `crushr-extract` → `crushr extract ...`
- `crushr-info` → `crushr info ...`

Each wrapper provides the same baseline control mechanics: `--help`, `--version`, and `about`.

## Core design principles

### Prove, don't guess

If a path, file identity, or recovery outcome cannot be proven from surviving archive evidence, crushr does not invent certainty.

### Separate trust classes explicitly

Recovery and extraction outcomes distinguish between:

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

This prevents degraded or partial output from being confused with fully canonical extraction.

### Fail closed by default

Strict commands refuse clearly when canonical guarantees cannot be met. Recovery is explicit.

### Linux-first honesty

crushr is designed first for real Linux archival workflows. Other platforms may be supported later, but they are not allowed to redefine the core metadata model.

### Archives should be inspectable

Archives should not remain opaque until extraction. Listing, structural inspection, metadata visibility, and later spatial introspection are part of the product direction.

## Recovery model

`crushr extract` is strict by default.

If strict canonical extraction cannot be completed, the command should refuse clearly and direct the operator to recovery mode:

- `crushr extract ...` → strict canonical extraction only
- `crushr extract --recover ...` → recovery-aware extraction

Recovery-aware extraction separates output by trust class:

- `canonical/`
- `metadata_degraded/`
- `recovered_named/`
- `_crushr_recovery/anonymous/`
- `_crushr_recovery/manifest.json`

Recovery results are reported explicitly as:

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

Anonymous recovered files follow a deterministic naming policy:

- high-confidence classification → `file_<id>.<ext>`
- medium-confidence classification → `file_<id>.probable-<type>.bin`
- low/unknown confidence → `file_<id>.bin`

The recovery manifest preserves structured classification and identity metadata for all recovered outputs.

## Linux-first preservation model

crushr's foundational philosophy is recoverability and truthful inspection. Tar-style preservation is a secondary but increasingly important capability layered onto that foundation.

### Preservation profiles

`crushr pack` supports explicit archive preservation contracts:

- `--preservation full` (default)
- `--preservation basic`
- `--preservation payload-only`

The selected preservation profile is recorded in archive metadata and shown by `crushr info`.

#### full

Preserves the complete Linux-first metadata and entry-kind set crushr currently supports.

#### basic

Preserves regular files, directories, empty directories, symlinks, hard links, mode, mtime, and sparse semantics.

Intentionally omits:

- xattrs
- uid/gid
- uname/gname
- ACLs
- SELinux labels
- Linux capabilities
- FIFOs
- device nodes

#### payload-only

Preserves only regular-file payload bytes plus logical tree reconstruction directories.

Intentionally omits:

- symlink semantics
- hard link semantics
- mode
- mtime
- sparse semantics
- xattrs
- ownership
- ACLs
- SELinux labels
- Linux capabilities
- FIFOs
- device nodes

If a selected profile excludes an entry kind, crushr warns and omits it rather than fabricating an alternate representation.

### Current Linux-first preservation scope

With `--preservation full`, crushr currently preserves:

- regular files
- directories
- empty directories
- symlinks and link targets
- hard links
- sparse files
- FIFOs
- char/block device nodes
- file mode / permissions
- modification time (`mtime`)
- extended attributes (`xattrs`)
- numeric ownership (`uid` / `gid`)
- optional ownership names (`uname` / `gname`) when available
- POSIX ACL metadata (`system.posix_acl_access`, `system.posix_acl_default`)
- SELinux label metadata (`security.selinux`)
- Linux file capability metadata (`security.capability`)

Where preservation or restoration cannot be applied due to platform or permission constraints, crushr degrades honestly and warns rather than silently pretending success.

### Long-term preservation goal

crushr’s long-term goal is broad Linux-first archive fidelity suitable for serious tar-based workflows.

That means, over time, supporting as much tar-class behavior as is practical and honest, including metadata and entry classes beyond simple payload preservation.

This is a staged roadmap goal, not a claim that crushr already has full tar parity in every environment.

## Archive introspection

crushr archives are no longer black boxes.

`crushr info --list` provides pre-extraction logical archive listing using archive metadata rather than payload extraction.

Current behavior is intentionally fail-closed:

- if archive structure can be proven, crushr lists it
- if metadata needed for listing is unavailable, crushr does not invent structure
- directories in listing output are derived from stored logical paths rather than treated as independent authoritative archive objects

This introspection line is expected to expand further in the 0.4.x series, including richer metadata surfacing and deeper archive/layout visibility.

## Documentation

Public product, reference, and historical material lives under `docs/`.

Primary entry points:

- `docs/index.md` — site landing page
- `docs/why-crushr.md` — positioning and legitimacy
- `docs/whitepaper/index.md` — technical whitepaper
- `docs/reference/index.md` — concise technical reference
- `docs/chronicles/index.md` — historical project milestones and public writing

The published docs site targets **Zensical** via `zensical.toml`.

## Internal project control

The repository also contains internal planning and control material under:

- `.ai/` — active project-control documents
- `.ai/contracts/` — policy and interface contracts used during development

These files are not part of the public documentation site and should be treated as internal engineering/project-control material.

## Product boundary

Current boundary classes:

- **Stable product surface:** user-facing CLI behavior and machine-readable outputs of `crushr pack`, `crushr verify`, `crushr extract`, `crushr extract --recover`, `crushr info`, `crushr info --list`, and thin wrappers over those commands
- **Bounded internal surface:** workspace Rust crates/modules used to implement the tool suite
- **Experimental/lab-only surface:** `crushr lab`, corruption research workflows, format-comparison tooling, and research schemas/artifacts
- **Removed primary surface:** standalone salvage as a normal operator-facing command

Treat these boundaries as canonical unless explicitly revised by a future decision.

## CLI presentation

Public-facing commands share one operator-facing presentation system:

- consistent title/header structure
- consistent section and summary layout
- shared status vocabulary
- shared semantic color usage
- shared progress rendering for long-running operations
- restrained motion only for active work
- stable final summaries suitable for terminal use and copy/paste

The CLI is designed to feel calm, explicit, and trustworthy rather than flashy.

### Silent/scriptable mode

Script-oriented paths support quiet machine-friendly execution where applicable.

Silent mode suppresses interactive multi-line presentation and emits deterministic concise summaries suitable for automation.

## Evidence-oriented workflow

crushr is designed to fit evidence-aware and failure-aware workflows:

1. Media or source material is acquired externally.
2. Files are packaged into crushr archives.
3. Verification establishes what remains trustworthy.
4. Strict extraction returns only canonical outputs.
5. Recovery-aware extraction returns outputs with explicit trust segregation.
6. Later reviewers can rerun verification and recovery against the same archive and receive deterministic classifications.

## Roadmap direction

Near-term priorities continue along these lines:

- finish Linux-first preservation and recovery semantics
- improve archive inspection and metadata visibility
- add deeper introspection of container/layout structure
- begin benchmark and compression analysis once core semantics stabilize
- explore reproducible archive mode in the future

## Product version governance

- Root `VERSION` is the canonical product version source (strict SemVer only, no `v` prefix).
- Human version bumps should edit `VERSION` only, then run `./scripts/sync-version.sh` to propagate `workspace.package.version`.
- Validate drift with `./scripts/check-version-sync.sh`.

## License

Code in this repository is dual-licensed under **MIT OR Apache-2.0**.

- You may use, modify, and distribute code under either license at your option.
- Contributions are accepted under the same dual-license terms unless explicitly stated otherwise.

Documentation and diagrams (Markdown and visual assets) are licensed under **CC-BY-4.0**.

This repository is structured for REUSE compliance with SPDX headers and `REUSE.toml` metadata.
