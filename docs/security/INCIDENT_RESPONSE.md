<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Incident Response

## Intent

This page defines what constitutes a security-relevant incident in crushr and how such incidents are handled.

## Guarantees

- Incidents are surfaced explicitly rather than hidden behind convenience behavior
- Unverified or unsafe output is contained before trust-bearing use
- Analysis is driven by structured evidence where available
- Resolution must improve determinism, reporting, or boundaries
- No repair behavior is implied by incident handling

## Behavior

### Incident definition

An incident is any condition where:

- archive integrity is compromised
- verification fails unexpectedly
- recovery behavior violates documented guarantees
- system behavior contradicts published invariants

### Response process

1. **Detect**
   - failure surfaced via verification, inspection, extraction, recovery, or user report

2. **Contain**
   - prevent further processing of unverified data
   - fail the operation immediately where required

3. **Analyze**
   - use structured outputs to determine failure scope
   - identify affected regions, invariants, or behaviors

4. **Report**
   - provide explicit machine-readable output describing failure
   - no silent handling

5. **Resolve**
   - correct the underlying implementation or documentation defect
   - add regression tests or documentation updates as needed

## Boundaries / Non-goals

Incident response does not include automatic repair of corrupted data, heuristic reconstruction, or silent recovery.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Principle

All incidents must result in increased determinism, better reporting, or clearer system boundaries.
