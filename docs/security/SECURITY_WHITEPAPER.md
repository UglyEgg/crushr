<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Security Whitepaper

## Intent

crushr is a deterministic archive system that preserves and exposes data truth under failure.

This whitepaper explains the security rationale behind that model: what the system is allowed to claim after corruption, partial damage, structural inconsistency, or metadata loss.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Design philosophy

crushr is guided by a small set of strict principles:

- integrity over convenience
- explicit behavior over implicit assumptions
- determinism over ambiguity
- verifiable output over permissive recovery behavior

These principles are enforced through architectural constraints rather than optional features.

### Threat model summary

crushr assumes:

- all input archives are untrusted
- archive structure and metadata may be malicious or corrupted
- damage may be partial, localized, or deliberately induced

The system is explicitly designed to resist:

- undetected data modification
- structural manipulation
- path traversal during extraction
- silent partial recovery

crushr does not claim confidentiality guarantees and does not assume trusted environments.

### Integrity model

Integrity is enforced through:

- structural validation of archive layout and references
- integrity verification of payload-bearing or truth-bearing components
- policy checks tied to the requested operation

Data is only considered trustworthy for a requested operation if:

- structural checks pass
- required components for that operation are present and acceptable
- required payload or truth-bearing components verify successfully
- metadata and extraction targets satisfy the active policy

If these conditions fail, the result is refused or classified explicitly under recovery policy.

### Verification pipeline

Assessment occurs in defined layers:

1. structural validation
2. index, reference, and component validation
3. integrity verification for the requested operation
4. metadata and policy assessment appropriate to strict or recover behavior
5. extraction safety and path-confinement validation before filesystem writes

Strict mode requires canonical extraction conditions.
Recover mode permits explicitly classified non-canonical outcomes only within published policy boundaries and never as silent success.

### Recovery boundary

crushr supports a controlled recovery surface with strict constraints:

- only verified payload data may enter trust-bearing recovery outputs
- metadata-degraded or partial outcomes are explicitly classified
- unverifiable regions are excluded from trust-bearing output
- filesystem safety rules remain mandatory regardless of mode

The system does not perform heuristic reconstruction, inferred repair, or hidden failure smoothing.

## Boundaries / Non-goals

This whitepaper does not claim confidentiality, availability, or formal certification. It describes integrity, trust classification, and bounded extraction behavior.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Summary

crushr's security posture is built around one principle: the system may only claim what surviving evidence can justify for the requested operation.
