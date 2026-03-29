# Verification Specification

## Purpose
Define the criteria for determining whether an archive or its contents are valid for a requested operation.

## Validation Phases

### Phase 1: Structural / Parse Validation
- verify archive identifiers and required structure
- validate component layout
- ensure required components are present for further processing

Failure: reject archive

### Phase 2: Index and Reference Validation
- confirm references match actual regions
- validate offsets and lengths
- ensure no invalid ranges or impossible references exist

Failure: reject archive or mark required component unusable for the requested operation

### Phase 3: Integrity Verification
- validate integrity signals for components relevant to the requested operation
- reject mismatches in strict/default paths
- isolate unusable components where recovery-capable policy allows explicit degraded handling

Failure: reject the requested strict operation or refuse the affected output

### Phase 4: Metadata and Policy Assessment
- validate file paths
- ensure no duplicates after normalization where required
- confirm logical structure matches validated references
- determine whether the active mode requires full metadata truth or permits explicit degraded routing

Failure:
- strict/default mode: refuse extraction
- recover/salvage mode: refuse affected output or route to explicit degraded handling, depending on policy

### Phase 5: Extraction Safety
- normalize paths
- reject absolute paths
- reject traversal sequences
- reject invalid filesystem targets
- enforce destination confinement before writes occur

Failure: skip or reject the affected extraction output

## Valid Archive Definition

An archive is considered valid for a requested operation if:
- all structural checks pass
- all referenced components required for that operation validate
- metadata and extraction targets satisfy the active policy
- no integrity mismatch exists on required truth-bearing paths

## Recover / Salvage Mode Behavior

When enabled:
- attempt recovery of individually valid extents or entries according to policy
- exclude failed components from trust-bearing outputs
- produce explicit report of recovered, degraded, skipped, and failed items
- never label partial recovery as full success

## Exit Code Model

- 0: success, fully valid for requested operation
- 1: usage error
- 2: structural or validation failure

## Determinism

Verification should produce:
- identical results for identical inputs
- consistent structured output for identical failure conditions

## Non-Ambiguity Requirement

At no point should:
- corrupted data be labeled valid
- degraded recovery be indistinguishable from full recovery
- required truth failures be hidden behind convenience behavior

## Summary

Verification is binary at the point of trust:
- data is either proven valid for the requested use
- or explicitly rejected / degraded under published recovery policy
