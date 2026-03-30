<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Inspection and recovery contract

## Intent

This page defines the public behavioral boundary for crushr's current inspection and bounded-recovery surfaces.

## Guarantees

- Inspection does not mutate archive contents or the filesystem
- Recovery remains explicit, bounded, and classified
- No repair or fixer-tool semantics are implied
- Trust-bearing output remains subject to validation, verification, and extraction-safety checks
- Non-canonical outcomes are never flattened into ordinary success

## Behavior

Current public surfaces:

- `crushr info`
- `crushr info --list`
- `crushr verify`
- `crushr extract --recover`

crushr exposes inspection and bounded recovery. It does not claim a generic checker/fixer or repair-tool model.

### Inspection

- `crushr info` provides archive-level introspection
- `crushr info --list` provides entry-level introspection without extraction
- `crushr verify` determines whether strict extraction requirements are satisfied or whether recovery mode is required

### Recovery

- `crushr extract --recover` allows classified output where strict extraction cannot succeed
- only verified payload data may enter trust-bearing recovery outputs
- metadata loss or structural incompleteness must remain visible through explicit classification

## Boundaries / Non-goals

This contract does not define repair behavior, alternate command surfaces, or hidden recovery heuristics.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies
