# Threat Model

## Purpose
Define the adversarial conditions under which crushr operates and the boundaries of its guarantees.

## Assets
- archive payload data
- archive structure
- integrity signals and verification outputs
- recovery outputs

## Trust Model

Trusted:
- crushr implementation as built from source
- verified archive data after validation

Untrusted:
- all input archives
- all external file paths
- all metadata until validated
- runtime environment

## Adversary Capabilities

The adversary may:
- craft malicious archives
- modify archive contents or structure
- inject malformed metadata
- attempt path traversal during extraction
- induce partial corruption
- attempt to cause silent data loss or misinterpretation

The adversary may not:
- modify verified data without detection, assuming integrity primitives hold

## Threat Categories

### T1: Integrity Subversion
Goal: modify archive contents without detection  
Mitigation:
- explicit integrity verification
- fail-closed on mismatch

### T2: Structural Manipulation
Goal: corrupt index or metadata to mislead extraction  
Mitigation:
- strict parsing and validation
- reject inconsistent structures

### T3: Path Traversal / Escape
Goal: write files outside intended extraction directory  
Mitigation:
- normalize paths
- reject absolute paths and traversal sequences

### T4: Silent Data Loss
Goal: cause partial recovery without user awareness  
Mitigation:
- explicit salvage mode
- mandatory reporting of missing or corrupt data

### T5: Malformed Input Exploitation
Goal: trigger undefined behavior or crashes  
Mitigation:
- defensive parsing
- explicit error handling
- no unsafe assumptions

## Trust Boundaries

1. Archive Input Boundary
   - all archives treated as hostile until verified

2. Verification Boundary
   - only verified data is eligible for extraction or reporting

3. Extraction Boundary
   - filesystem writes constrained and validated

## Security Guarantees

- no undetected modification of verified data
- no silent partial recovery
- no extraction outside intended directory
- no interpretation of unverified data as valid

## Non-Goals

- confidentiality guarantees
- availability guarantees under adversarial conditions
- recovery of corrupted data beyond verified extents

## Summary

crushr assumes hostile input and prioritizes integrity, explicit failure, and verifiable behavior.
