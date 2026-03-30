<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Propagation Graph v1 (`crushr info --json --report propagation`)

## Intent

This contract defines a deterministic explanatory propagation surface for archive-impact reasoning.

It is explanatory and bounded. It is not a repair, reconstruction, or recovery plan.

## Scope

Applies to:

- structure/block dependency graph (`nodes`, `edges`)
- detected corruption inputs (`detected_corruption`)
- required cross-entry structures (`required_structures`)
- activated impacts caused by currently detected corruption (`activated_impacts`)
- per-entry dependency and impact explanation (`entry_impacts`)

## Command surface

- `crushr info <archive> --json --report propagation`

## Semantics

- direct and propagated per-entry dependencies are explicit with bounded reasons
- graph structure is distinct from currently activated impact
- entry-level canonical blocking and trust-class support are evidence-only
- trust classes remain bounded to:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

## Determinism

For identical archive bytes:

- nodes are deterministic
- edges are deterministic
- detected corruption ordering is deterministic
- activated impact ordering is deterministic
- entry impact ordering is deterministic
- reason values are bounded and stable

## Limits and non-inferences

- this contract does not imply repair or speculative reconstruction availability
- it does not estimate speculative survivability
- it does not override extraction trust classification or fail-closed behavior
