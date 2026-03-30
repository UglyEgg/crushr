<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Phase Plan

This document defines the high-level development phases for crushr after the core archive architecture and preservation model were established.

The project is sequenced across three maturity bands:

- **0.x** — product proof, stabilization, preservation scope, introspection, benchmarking, and contract hardening
- **1.x** — stable preservation platform with locked contracts and workflow maturity
- **2.x** — evidence-grade extension layer (custody, signing, classification expansion)

## Current architectural identity

crushr is a deterministic archive system built around:

- explicit separation of payload integrity from metadata completeness
- deterministic validation and verification behavior
- explicit trust classification for non-canonical outcomes
- fail-closed strict extraction semantics
- bounded explicit recovery through `extract --recover`
- archive introspection without extraction
- Linux-first preservation fidelity layered onto an integrity-first model

crushr prioritizes data integrity, explicit truth, and bounded failure behavior over maximum compression ratio.

## Phase status

- [x] Phase 15 — Dictionary hardening and namespace factoring
- [x] Phase 16 — Benchmarking and compression evidence
- [ ] Documentation alignment pass — builder/control-doc and public-doc normalization
- [ ] Phase 17 — Introspection and truth-surface expansion
- [ ] Phase 18+ — subsequent roadmap phases as explicitly approved

## Documentation alignment pass (current)

### Goal

Bring `.ai/`, contract, and public documentation onto one canonical vocabulary and one coherent product surface.

### Required outcomes

- no current product language uses `salvage` as a live operator-facing term
- no current product language uses `fsck` as a live command or mental model
- wrapper binaries are not documented as canonical product surface
- validation vs verification are defined consistently
- recovery and trust classes are defined consistently
- public docs and builder-facing `.ai/` docs do not contradict each other

### Guardrails

- do not rewrite historical chronology as if it never happened
- do not invent new command surfaces
- do not broaden public claims beyond current product behavior
- do not let builder-facing docs drift into obsolete vocabulary

## Phase 17 — Introspection and truth surfaces (next after alignment)

### Goal

Expand archive introspection so container truth, entry truth, and structural visibility become richer without weakening fail-closed semantics.

### Focus areas

- deeper archive/container introspection
- richer `info --list` entry attributes and trust context
- improved archive-layout visibility
- explicit truth-surface reporting without extraction

### Guardrails

- introspection is explanatory, not repair behavior
- recovery remains explicit and bounded
- payload integrity and metadata completeness remain distinct

## 2026-03-30 — Active Step Update (P17S01f0)

- Completed: Phase 17 Step 01 fix 0 (`P17S01f0`).
- Implemented archive-level truth summary expansion for `crushr info` (human + JSON), including deterministic structural summary, verification summary, and explicit `strict_extraction_supported`.
- Removed wrapper binaries from `crushr` build targets and source files; command access is now via `crushr` subcommands.
- Updated tests and schema contracts to match the single-binary command surface and new `crushr info` JSON truth shape.
- Next: Phase 17 follow-on introspection packets (entry-level/propagation work remains out of scope for this step).


## 2026-03-30 — Active Step Update (P17S02f0)

- Completed: Phase 17 Step 02 fix 0 (`P17S02f0`).
- Added entry-level introspection commands:
  - `crushr info --entry <logical/path>` (exact path lookup; human + JSON)
  - `crushr info --find <query>` (deterministic substring search; human + JSON)
- Reserved forward-compatible search CLI shape:
  - `--find-mode substring` (currently enforced)
  - `--find-limit <n>`
- Added deterministic not-found and zero-match behavior for `--entry`/`--find`.
- Updated tests and docs for the expanded `info` introspection surface.
- Next: follow-on Phase 17 introspection packets (propagation/impact extensions remain separate scope).

## 2026-03-30 — Active Step Update (P17S02f1)

- Completed: Phase 17 Step 02 fix 1 (`P17S02f1`).
- Enriched `crushr info --entry` with additional deterministic proof/structure detail in both human and JSON outputs:
  - `payload_blake3`
  - `logical_range.start` / `logical_range.end`
  - `identity_source`
- Added human-readable entry rows for payload BLAKE3, logical range (`hex + decimal`), and identity source.
- Kept trust semantics unchanged (no new trust classes, no extraction behavior change, no archive-physical offsets exposed).
- Updated CLI presentation tests and guide documentation for the expanded `--entry` truth surface.
- Next: Phase 17 follow-on introspection packets outside this bounded step.


## 2026-03-30 — Active Step Update (P17S03f0)

- Completed: Phase 17 Step 03 fix 0 (`P17S03f0`).
- Hardened `crushr info --report propagation` into an explanatory truth surface that explicitly separates:
  - deterministic dependency graph (`nodes`, `edges`)
  - detected corruption inputs (`detected_corruption`)
  - currently activated impacts (`activated_impacts`)
  - per-entry dependency + impact classification support (`entry_impacts`)
- Added bounded, stable dependency reasons and direct/propagated entry dependency semantics.
- Added deterministic entry-level canonical blocking + trust-class support explanation aligned to existing trust classes.
- Updated propagation schema/contract/tests and guide docs for the expanded report shape.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

