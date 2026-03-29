# Architectural Invariants

## Purpose
Define the non-negotiable properties of crushr’s design that must hold across implementations, features, and changes.

These invariants must not be weakened without explicit architectural decision.

## Core Invariants

### I1: Integrity Before All Other Concerns
No operation may return data that has not been verified.

### I2: Fail-Closed Semantics
On ambiguity or validation failure, the system must stop or explicitly reject the operation.

Forbidden:
- silent fallback
- implicit recovery
- best-effort continuation without signaling

### I3: No Silent Data Loss
If data is missing, corrupted, or skipped, this must be explicitly reported.

### I4: Deterministic Behavior
Given identical inputs, crushr must produce identical outputs, verification results, and error classifications within the defined contract.

### I5: Verified Data Boundary
Only data that has passed validation may be returned, written, or used in further computation.

### I6: Explicit Trust Model
No input is trusted by default.

### I7: No Heuristic Reconstruction
Corrupted or missing data must not be guessed or inferred.

Allowed:
- recovery of independently verified extents

### I8: Structural Consistency Enforcement
All archive structures must be internally consistent.

### I9: Extraction Safety Boundary
Filesystem writes must never escape the intended destination.

### I10: Observable Failure
All failure states must be explicit, classifiable, and describable.

## Derived Invariants

### D1: Verification is Mandatory
All trust-bearing read paths must pass through validation logic before returning data.

### D2: Salvage is Explicit and Isolated
Partial recovery requires explicit user intent and must not affect default behavior.

### D3: Output Truthfulness
Outputs must reflect actual system state, not desired or inferred state.

### D4: Minimal Trust Surface
Reduce reliance on external metadata, implicit assumptions, and non-validated dependencies.

## Invariant Enforcement

Invariants are enforced through:
- validation-first code paths
- strict error handling
- corruption-focused testing
- review discipline that rejects guarantee erosion

## Violation Policy

Any change that weakens an invariant must:
1. be explicitly documented
2. include justification
3. update this document
4. include new risk analysis

## Summary

These invariants define crushr’s identity:
- correctness over convenience
- explicit over implicit
- verifiable over assumed
