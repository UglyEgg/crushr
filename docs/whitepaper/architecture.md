<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Format Architecture

crushr is built around a single principle:

> Keep truth closest to the data.

## Components

- Extent identity (primary truth)
- Dictionary system (secondary naming layer)
- Tail frame (optimization, not authority)

## Rejected designs

| Design | Reason |
|-------|-------|
| central manifest | single point of failure |
| metadata-heavy | poor corruption resilience |
| format leadership | invalid under partial damage |

## Final model

- distributed identity
- mirrored naming
- optional coordination structures

This is not stylistic. It is empirically derived.
