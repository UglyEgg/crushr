<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Verification Specification

## Intent

This page defines the criteria used to determine whether an archive or its contents are acceptable for a requested operation.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Phase 1: Structural validation

- verify archive identifiers and required structure
- validate component layout
- ensure required components are present for further processing

Failure: reject archive for the requested operation.

### Phase 2: Index and reference validation

- confirm references match actual regions
- validate offsets and lengths
- ensure no invalid ranges or impossible references exist

Failure: reject archive or mark the affected component unusable for the requested operation.

### Phase 3: Integrity verification

- verify integrity signals for components relevant to the requested operation
- reject mismatches in strict/default paths
- isolate unusable components where recovery policy allows explicit classified output

Failure: reject the requested strict operation or refuse the affected output.

### Phase 4: Metadata and policy assessment

- validate file paths
- ensure no duplicates after normalization where required
- confirm logical structure matches validated references
- determine whether the active mode requires full metadata truth or permits explicit classified routing

Failure:

- **strict/default** — refuse extraction
- **recover** — refuse the affected output or route it into an explicit non-canonical trust class, depending on policy and surviving evidence

### Phase 5: Extraction safety

- normalize paths
- reject absolute paths
- reject traversal sequences
- reject invalid filesystem targets
- enforce destination confinement before writes occur

Failure: skip or reject the affected extraction output.

## Validity for a requested operation

An archive is acceptable for a requested operation if:

- all structural checks required for that operation pass
- all referenced components required for that operation validate structurally
- all required truth-bearing components verify successfully
- metadata and extraction targets satisfy the active policy

## Recover mode behavior

When `extract --recover` is enabled:

- individually verified payload-bearing components may be emitted according to policy
- unverifiable components are excluded from trust-bearing output
- output classification must distinguish canonical, metadata-degraded, recovered, and unrecoverable outcomes explicitly
- partial or degraded recovery is never labeled as full success

## Exit code model

- `0` — success, acceptable for requested operation
- `1` — usage error
- `2` — structural or validation failure

## Determinism

Verification must produce:

- identical results for identical inputs and requested operations
- consistent structured output for identical failure conditions

## Non-ambiguity requirement

At no point may:

- corrupted data be labeled trustworthy
- degraded recovery be indistinguishable from canonical extraction
- required truth failures be hidden behind convenience behavior

## Boundaries / Non-goals

This page does not authorize repair behavior, heuristic reconstruction, or alternate user-facing state systems.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Summary

Trust is binary at the point of use: required truth is either established for the requested operation, or the result is explicitly refused or classified under recovery policy.
