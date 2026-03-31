<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Recovery classification model

This page documents the recovery classification model used by crushr.

## Intent

This page defines how crushr classifies extraction outcomes when strict canonical extraction is not possible.

The model is built on a core architectural boundary: payload integrity and metadata completeness are independent dimensions.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Core model

crushr does not treat metadata loss as equivalent to payload corruption.

A payload may remain cryptographically verified even when its original identity, path, or surrounding structural metadata is incomplete. Recovery classification exists to make that distinction explicit.

### Trust classes

Recovery and extraction results are classified into the following trust classes:

- `canonical` — payload integrity is verified and required metadata is intact
- `metadata_degraded` — payload integrity is verified, but metadata or structure is incomplete
- `recovered_named` — payload integrity is verified and identity has been reconstructed within defined constraints
- `recovered_anonymous` — payload integrity is verified but no reliable identity remains
- `unrecoverable` — payload integrity cannot be proven to required standards

These classes describe what can be proven from surviving archive evidence. They do not imply reconstruction, repair, or guessed certainty.

### Recovery behavior

- `crushr extract` remains strict and requires canonical extraction conditions
- `crushr extract --recover` allows classified output when strict canonical extraction is not possible
- Recovery never reconstructs, infers, or repairs missing data
- Anonymous recovery uses deterministic naming and structured manifest output

### Deterministic anonymous naming

Anonymous recovered files follow the deterministic naming policy:

- high-confidence classification → `file_<id>.<ext>`
- medium-confidence classification → `file_<id>.probable-<type>.bin`
- low/unknown confidence → `file_<id>.bin`

The recovery manifest preserves classification and identity metadata for all recovered outputs.

## Boundaries / Non-goals

This model does not describe repair, best-effort reconstruction, or hidden failure smoothing.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Schema contract

Machine-readable recovery output is defined by `schemas/crushr-extract-result.v1.schema.json`.
