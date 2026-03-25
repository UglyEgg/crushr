<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr

[![Policy Gate](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml/badge.svg)](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![REUSE status](https://api.reuse.software/badge/github.com/UglyEgg/crushr)](https://api.reuse.software/info/github.com/UglyEgg/crushr)

Failure-aware archival and recovery tooling with deterministic verification semantics.

Its core design question is simple:

> When an archive is damaged, what can still be proven and recovered without guessing?

crushr is built around:

- integrity-first extraction
- deterministic verification
- explicit trust segregation during recovery
- fail-closed naming semantics
- anonymous fallback when original identity cannot be proven
- recovery outputs that preserve evidence without pretending certainty

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

These files are not part of the website and should be treated as internal engineering/project-control material.

## Canonical runtime boundary

The canonical public tool surface is:

- `crushr pack`
- `crushr verify`
- `crushr extract`
- `crushr extract --recover`
- `crushr info`
- `crushr about`

Thin wrapper binaries are retained for convenience and map to the canonical `crushr` commands:

- `crushr-pack` → `crushr pack ...`
- `crushr-extract` → `crushr extract ...`
- `crushr-info` → `crushr info ...`

Each wrapper provides the same baseline control mechanics: `--help`, `--version`, and `about`.

## Recovery model

`crushr extract` is strict by default.

If strict canonical extraction cannot be completed, the command should refuse clearly and direct the operator to recovery mode:

- `crushr extract ...` → strict canonical extraction only
- `crushr extract --recover ...` → recovery-aware extraction

Recovery-aware extraction separates output by trust class:

- `canonical/`
- `recovered_named/`
- `_crushr_recovery/anonymous/`
- `_crushr_recovery/manifest.json`

Recovery results are reported explicitly as:

- `canonical`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

Anonymous recovered files follow a deterministic naming policy:

- high-confidence classification → `file_<id>.<ext>`
- medium-confidence classification → `file_<id>.probable-<type>.bin`
- low/unknown confidence → `file_<id>.bin`

The recovery manifest preserves structured classification and identity metadata for all recovered outputs.

## Linux-first filesystem preservation

`crushr pack` / `crushr extract` now preserve baseline Linux filesystem metadata for canonical paths:

- regular files and directory paths
- symlink entries and link targets
- mode/permission bits
- modification time (`mtime`)
- empty directories
- extended attributes (xattrs)

Current scope is intentionally Linux-first. Non-Linux platforms may degrade with explicit warnings for unsupported metadata restoration (especially xattrs) rather than silent metadata fabrication.

Current limitations in this baseline packet:

- uid/gid numeric ownership preservation is intentionally deferred.
- Permission-denied xattr restore warning paths are implemented, but deterministic CI coverage for that specific denied-path scenario is not guaranteed in every environment.

## Product boundary

Current boundary classes:

- **Stable product surface:** user-facing CLI behavior and machine-readable outputs of `crushr pack`, `crushr verify`, `crushr extract`, `crushr extract --recover`, `crushr info`, and thin wrappers over those commands
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

The CLI is designed to be calm, explicit, and trustworthy rather than flashy.

### Silent/scriptable mode

Script-oriented paths support quiet machine-friendly execution where applicable.

Silent mode suppresses interactive multi-line presentation and emits deterministic concise summaries suitable for automation.

## Evidence-oriented workflow

crushr is designed to fit evidence-aware and failure-aware workflows:

1. Media or source material is acquired externally.
2. Files are packaged into crushr archives.
3. Verification establishes what remains trustworthy.
4. Strict extraction returns only canonical outputs.
5. Recovery-aware extraction returns recoverable outputs with explicit trust segregation.
6. Later reviewers can rerun verification and recovery against the same archive and receive deterministic classifications.

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
