<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Integrity Guarantees

## Intent

This page defines what crushr guarantees about payload integrity, metadata completeness, and the boundary between them.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Validation vs verification

crushr separates:

- **Validation** — structural correctness of archive components, references, and extraction targets
- **Verification** — integrity correctness of payload-bearing or truth-bearing data via cryptographic proof

These are related but not interchangeable.

### Core integrity boundary

If crushr reports payload data as trustworthy for a requested operation, that result means:

- the required structural validation checks passed
- the required integrity verification checks passed
- the output remained within the active policy boundary for that operation

### Payload and metadata

crushr does not treat metadata loss as equivalent to payload corruption.

A payload may remain verified even when identity, path, or surrounding metadata is incomplete. In those cases, the result must be classified explicitly rather than presented as canonical.

### Operational guarantees

#### G1: No Silent Corruption
Corrupted or unverifiable payload data is never presented as trustworthy.

#### G2: Fail-Closed Strict Behavior
If required truth cannot be established for strict extraction, the operation is refused.

#### G3: Explicit Recovery Only
Partial or metadata-degraded output is only possible when recovery is explicitly requested.

#### G4: Deterministic Assessment
Given the same input and requested operation, validation, verification, and classification results are consistent.

#### G5: No Implicit Trust
All archive input is treated as untrusted until the checks required for the requested operation succeed.

### Recovery guarantees

In `extract --recover`:

- only cryptographically verified payload data may enter trust-bearing output classes
- unverifiable payload data is excluded from trust-bearing output
- output classes must make metadata degradation, recovered identity, or unrecoverability explicit

## Boundaries / Non-goals

This page does not guarantee completeness after corruption and does not authorize guessed identity, heuristic reconstruction, or silent fallback.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Failure semantics

When required checks fail:

- exit codes reflect the failure class
- structured output describes the issue
- no ambiguous success state is emitted

## Summary

crushr guarantees correctness of what it returns, not completeness of what corruption destroyed.
