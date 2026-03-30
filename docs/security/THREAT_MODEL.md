<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Threat Model

## Intent

This page identifies the primary threats crushr is designed to resist within its actual scope.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Assumptions

Treat as untrusted until validated or verified as required by the operation:

- all archive bytes
- all archive structure and metadata
- all paths and extraction targets
- all externally supplied archives regardless of source

### Primary threats

1. **Undetected archive corruption**
   - payload or truth-bearing data modified without operator awareness

2. **Structural manipulation**
   - malformed indexes, invalid offsets, impossible references, or inconsistent layout intended to confuse processing

3. **Extraction path escape**
   - absolute paths, traversal sequences, or unsafe targets intended to escape the destination boundary

4. **Silent non-canonical output**
   - degraded or partial recovery presented as ordinary success

5. **Malicious metadata influence**
   - names, paths, or metadata attempting to influence extraction or reporting outside the validated boundary

### Trust boundaries

1. **Archive input boundary**
   - all archives treated as hostile until the requested checks succeed

2. **Verification boundary**
   - only verified payload or truth-bearing data is eligible for trust-bearing output

3. **Extraction boundary**
   - filesystem writes remain constrained and validated regardless of mode

## Security guarantees

- no undetected modification of verified payload data within the archive model
- no silent partial recovery
- no extraction outside the intended directory
- no interpretation of unverified data as trustworthy

## Boundaries / Non-goals

This model does not claim confidentiality guarantees, availability guarantees under adversarial conditions, or recovery of unverifiable data.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Summary

crushr assumes hostile input and prioritizes integrity, explicit failure, and verifiable behavior.
