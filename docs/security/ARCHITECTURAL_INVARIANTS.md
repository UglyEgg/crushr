<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Architectural Invariants

## Intent

This page defines the non-negotiable properties of crushr's design that must hold across implementations, features, and changes.

These invariants must not be weakened without explicit architectural decision.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Core invariants

#### I1: Integrity before all other concerns
No operation may return trust-bearing data that has not been verified as required for that operation.

#### I2: Fail-closed semantics
On ambiguity or validation failure, the system must stop or explicitly refuse the operation.

Forbidden:

- silent fallback
- implicit recovery
- best-effort continuation without signaling

#### I3: No silent data loss
If data is missing, corrupted, skipped, or downgraded in classification, that outcome must be explicitly reported.

#### I4: Deterministic behavior
Given identical inputs, crushr must produce identical outputs, verification results, and error classifications within the defined contract.

#### I5: Verified-data boundary
Only data that has passed the required validation and verification checks may be returned, written, or used in further trust-bearing computation.

#### I6: Explicit trust model
No input is trusted by default.

#### I7: No heuristic reconstruction
Corrupted or missing data must not be guessed or inferred.

Allowed:

- recovery of independently verified payload data within explicit classification rules

#### I8: Structural consistency enforcement
All archive structures must be internally consistent.

#### I9: Extraction safety boundary
Filesystem writes must never escape the intended destination.

#### I10: Observable failure
All failure states must be explicit, classifiable, and describable.

### Derived invariants

#### D1: Verification is mandatory
All trust-bearing read paths must pass through validation and verification logic before returning data.

#### D2: Recovery is explicit and isolated
Partial or non-canonical recovery requires explicit user intent and must not affect default behavior.

#### D3: Output truthfulness
Outputs must reflect actual system state, not desired or inferred state.

#### D4: Minimal trust surface
Reduce reliance on external metadata, implicit assumptions, and non-validated dependencies.

## Invariant enforcement

Invariants are enforced through:

- validation-first code paths
- strict error handling
- corruption-focused testing
- review discipline that rejects guarantee erosion

## Boundaries / Non-goals

This page does not authorize alternate trust vocabularies, repair behavior, or silent downgrade paths.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Violation policy

Any change that weakens an invariant must:

1. be explicitly documented
2. include justification
3. update this document
4. include new risk analysis

## Summary

These invariants define crushr's identity:

- correctness over convenience
- explicit over implicit
- verifiable over assumed
