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
