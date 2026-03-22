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

crushr’s current architecture is built around:

- distributed extent identity
- mirrored naming dictionaries
- deterministic recovery classification
- fail-closed naming semantics
- anonymous fallback when naming proof is unavailable

## Documentation

The public product and whitepaper documentation lives under `docs/`.

Primary entry points:

- `docs/index.md` — site landing page
- `docs/why-crushr.md` — positioning and legitimacy
- `docs/whitepaper/index.md` — technical whitepaper
- `docs/format-evolution.md` — design-branch / decision history
- `docs/reference/index.md` — concise technical reference

The published docs site now targets **Zensical** via `zensical.toml`. `mkdocs.yml` remains only as a transition artifact until the docs pipeline fully drops it.

## Internal project control

The repository also contains internal planning and control material under:

- `.ai/` — active project-control documents
- `.ai/contracts/` — policy and interface contracts used during development

These files are not part of the website and should be treated as internal engineering/project-control material.

## Canonical runtime boundary

The canonical public tool boundary is:

- `crushr-pack`
- `crushr-info`
- `crushr-extract`
- `crushr-extract --verify`
- `crushr-salvage` (experimental, separate from canonical extraction)
- `crushr-lab` (research harness, not product surface)

`crushr-extract` remains strict and deterministic.

Wrapper entrypoints map to canonical `crushr` commands:

- `crushr-pack` → `crushr pack ...`
- `crushr-extract` → `crushr extract ...`
- `crushr-info` → `crushr info ...`
- `crushr-salvage` → `crushr salvage ...`

Each wrapper provides the same baseline control mechanics: `--help`, `--version`, and `about`.
These wrapper controls are only recognized as the first argument (`crushr-pack --help`, not `crushr-pack <arg> --help`).

## API boundary truth

Current boundary classes:

- **Stable product surface:** CLI behavior and machine-readable outputs of `crushr-pack`, `crushr-info`, `crushr-extract --verify`, and `crushr-extract`
- **Bounded internal surface:** workspace Rust crates/modules used to implement the tool suite (`crushr`, `crushr-core`, `crushr-format`, `crushr-lab`)
- **Experimental/lab-only surface:** `crushr-salvage`, `crushr-lab`, FORMAT comparison workflows, and research schemas/artifacts
- **Removed accidental exposure:** internal extraction-path and verify-report assembly details are no longer public library API

Treat these boundaries as canonical unless explicitly revised by a future decision.

## Unified CLI presentation contract

Public-facing runtime commands now share one operator-facing presentation grammar:

- Header: `== <tool> | <action> ==`
- Sections: `-- <section> --`
- Status markers: bounded vocabulary (`VERIFIED`, `OK`, `COMPLETE`, `PARTIAL`, `REFUSED`, `FAILED`, `RUNNING`, `SCANNING`, `WRITING`, `FINALIZING`)
- Final outcome lines are deterministic and status-prefixed.

### Silent/scriptable mode

`crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` support `--silent`.

- Silent mode suppresses multi-line presentation output.
- Silent mode emits one deterministic summary line:
  - `crushr-pack status=COMPLETE ...`
  - `crushr-extract status=COMPLETE|PARTIAL ...`
  - `crushr-extract status=VERIFIED|REFUSED ...` for `--verify`
  - `crushr-salvage status=PARTIAL ...` (research-only salvage surface)

### Evidence-package lifecycle alignment

crushr presentation/reporting now aligns to an evidence-review workflow:

1. Damaged media is imaged in an external evidence format.
2. Recovery/analysis tooling may produce intact files, partial files, carved fragments, and sidecars/logs.
3. Those outputs are packaged into crushr archives.
4. Verification/salvage reporting records deterministic classifications for verified, partial, and rejected/unresolved outcomes.
5. Later reviewers can rerun verification and recover the same typed result model.

## Product version governance

- Root `VERSION` is the canonical product version source (strict SemVer only, no `v` prefix).
- Human version bumps should edit `VERSION` only, then run `./scripts/sync-version.sh` to propagate `workspace.package.version`.
- Validate drift with `./scripts/check-version-sync.sh` (used by tests/tooling).

## License

Code in this repository is dual-licensed under **MIT OR Apache-2.0**.

- You may use, modify, and distribute code under either license at your option.
- Contributions are accepted under the same dual-license terms unless explicitly stated otherwise.

Documentation and diagrams (Markdown and visual assets) are licensed under **CC-BY-4.0**.

This repository is structured for REUSE compliance with SPDX headers and `REUSE.toml` metadata.
