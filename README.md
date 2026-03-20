<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr

**crushr** is a salvage-oriented archive format built for the failure case, not merely the happy path.

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
`crushr-fsck` is retained only as a temporary deprecated compatibility shim that directs users to `crushr-extract --verify`.

## API boundary truth

Current boundary classes:

- **Stable product surface:** CLI behavior and machine-readable outputs of `crushr-pack`, `crushr-info`, `crushr-extract --verify`, and `crushr-extract`
- **Bounded internal surface:** workspace Rust crates/modules used to implement the tool suite (`crushr`, `crushr-core`, `crushr-format`, `crushr-cli-common`)
- **Experimental/lab-only surface:** `crushr-salvage`, `crushr-lab`, FORMAT comparison workflows, and research schemas/artifacts
- **Removed accidental exposure:** internal extraction-path and verify-report assembly details are no longer public library API

Treat these boundaries as canonical unless explicitly revised by a future decision.

## License

Code in this repository is dual-licensed under **MIT OR Apache-2.0**.

- You may use, modify, and distribute code under either license at your option.
- Contributions are accepted under the same dual-license terms unless explicitly stated otherwise.

Documentation and diagrams (Markdown and visual assets) are licensed under **CC-BY-4.0**.

This repository is structured for REUSE compliance with SPDX headers and `REUSE.toml` metadata.
