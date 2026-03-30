<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Propagation Graph v1 (`crushr info --json --report propagation`)

## Intent

This contract defines a deterministic explanatory graph for archive-impact reasoning.

It is explanatory and bounded. It is not a repair, reconstruction, or recovery plan.

## Scope

Applies to the current graph-reporting surface for archive structures, required blocks, and affected file entries.

## Command surface

- `crushr info <archive> --json --report propagation`

## Semantics

- direct dependency is encoded as an edge with a bounded reason
- propagated impact explains which files depend on which required structures or blocks
- actual impact is the subset activated by currently detected corruption

## Determinism

For identical archive bytes:

- nodes are deterministic
- edges are deterministic
- per-file impact lists are deterministic
- reason values are bounded and stable

## Limits and non-inferences

- this contract does not imply recovery or repair availability
- it does not estimate speculative survivability
- it does not override extraction trust classification or fail-closed behavior
