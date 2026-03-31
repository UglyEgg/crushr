<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Technical reference

## Intent

This section defines crushr's reference surfaces: on-disk structure, metadata model, classification semantics, and archive behavior that can be described precisely without extraction.

Reference material explains how crushr represents payload truth, metadata completeness, and deterministic classification. It does not redefine the product vocabulary established by the README, `docs/index.md`, and the security documentation.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

Reference documentation describes the archive model in terms of:

- payload integrity
- metadata completeness
- structural correctness
- explicit trust classification

crushr's core architectural distinction is that payload integrity is independent from metadata completeness. This separation allows the system to preserve verified data even when required identity or structural context is incomplete.

Where a reference page describes recovery behavior, it refers to the current operator-facing recovery surface: `crushr extract --recover` and its explicit trust classes.

### Legacy terminology

Some reference material may retain legacy naming where required for schema continuity or historical accuracy.

Legacy terms do not redefine current product language. Canonical product terminology uses **recover** / **recovery** and the current trust classes:

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

## Boundaries / Non-goals

This section does not define alternate command surfaces, repair workflows, or historical operator vocabulary.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Included reference pages

1. [Archive format boundary](./archive-format.md)
2. [Extent identity](./extent-identity.md)
3. [Dictionary system](./dictionary-system.md)
4. [Recovery classification model](./recovery-model.md)
5. [Benchmark contract](./benchmarking.md)
6. [Benchmark baseline (v0.4.15)](./benchmark-baseline.md)

## Documentation roles

- **Guide** — operator-facing usage and workflow material
- **Whitepaper** — design rationale and evaluation
- **Technical reference** — current architecture and behavior boundary
- **Security and assurance** — guarantees, invariants, and trust model
- **Chronicles** — historical narrative, not normative behavior

Canonical behavior and guarantees are defined by the README, `docs/index.md`, and the security documentation.
