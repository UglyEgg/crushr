<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Project Scope

## Intent

crushr is a deterministic archive system that preserves and exposes data truth under failure.

It is built around one core distinction: payload integrity is independent from metadata completeness.

## In scope

- deterministic validation and verification
- strict fail-closed extraction
- explicit bounded recovery through `extract --recover`
- archive and entry introspection without extraction
- explicit trust classification for non-canonical outcomes
- Linux-first preservation fidelity layered onto an integrity-first model

## Out of scope

- parity reconstruction
- speculative decompression
- heuristic recovery logic
- automatic archive repair
- fixer-tool semantics
- hidden failure smoothing

## Trust-bearing output classes

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

## Scope rule

The system guarantees correctness of what it returns, not completeness of what corruption destroyed.
