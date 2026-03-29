# Integrity Guarantees

## Purpose
Define what crushr guarantees about data correctness and where those guarantees stop.

## Core Guarantee

If crushr reports data as valid, that data:
- has passed the relevant integrity checks
- is structurally consistent within the archive
- has passed all applicable validation checks

## Verification Model

Integrity is enforced through:
- integrity checks over archive components
- structural validation
- consistency checks during inspection and recovery

Current implementations may name specific primitives in technical reference documents. The guarantee surface is correctness of verified data, not attachment to a single implementation detail forever.

## Operational Guarantees

### G1: No Silent Corruption
Corrupted data will not be presented as valid.

### G2: Fail-Closed Behavior
If integrity cannot be verified, the operation fails or the affected region is excluded with explicit reporting.

### G3: Explicit Salvage Mode
Partial recovery is only possible when explicitly requested and is always reported.

### G4: Deterministic Validation
Given the same input, validation results are consistent.

### G5: No Implicit Trust
All data is treated as untrusted until verified.

## Recovery Guarantees

In salvage mode:
- only verified extents are recovered
- corrupted or unverifiable data is excluded
- results are explicitly marked as partial

## What is Not Guaranteed

- recovery of all original data in the presence of corruption
- preservation of interpretation where structure is materially damaged
- correctness of unverifiable regions

## Failure Semantics

When validation fails:
- exit codes reflect failure class
- structured output describes the issue
- no ambiguous success state is emitted

## Summary

crushr guarantees correctness of what it returns, not completeness of what corruption destroyed.
