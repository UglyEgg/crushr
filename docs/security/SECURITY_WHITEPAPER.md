# crushr Security Whitepaper

## Overview

crushr is a **salvage-oriented archive format born from corruption testing**.

Its primary concern is not generic archiving convenience. Its primary concern is what an archive format is allowed to claim after damage, partial corruption, or structural inconsistency.

The system is designed around integrity, verifiability, and explicit failure behavior. It treats all input as potentially hostile and requires validation before data is trusted, returned, or written.

Near-`tar` functionality exists because preservation claims are incomplete if the format cannot also perform the ordinary archive tasks surrounding those claims. That functionality supports completeness. It is not the center of the project.

## Design Philosophy

crushr is guided by a small set of strict principles:

- integrity over convenience
- explicit behavior over implicit assumptions
- determinism over ambiguity
- verifiable output over best-effort recovery

These principles are enforced through architectural constraints rather than optional features.

## Threat Model Summary

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

## Integrity Model

Integrity is enforced through:
- integrity verification over archive components
- structural validation of archive layout
- consistency checks between references and data

Data is only considered valid if:
- structural checks pass
- required components for the requested operation are present and verifiable
- metadata and extraction targets satisfy the active policy

If these conditions fail, the data is rejected or isolated from trust-bearing paths.

## Verification Pipeline

Validation occurs in defined layers:

1. structural / parse validation
2. index, reference, and component validation
3. integrity verification of components required for the requested operation
4. metadata and policy assessment appropriate to strict or recover behavior
5. extraction safety and path-confinement validation before filesystem writes

Strict mode requires verified truth for the requested operation.
Recover / salvage mode may permit explicitly degraded handling for affected items, but only within published policy boundaries and never as silent success.

## Failure Semantics

crushr follows fail-closed behavior for trust-bearing decisions:

- corrupted or unverifiable data is never returned as valid
- strict/default operations terminate explicitly when required truth is unavailable
- recover/salvage paths may return only explicitly bounded degraded results where policy permits
- exit codes classify failure type
- structured output describes failure conditions

Ambiguous or silent failure modes are not permitted.

## Recovery Model

crushr supports a controlled salvage mode with strict constraints:

- recovery is opt-in only
- only independently verified data is recovered as verified
- metadata-degraded or partial outcomes are explicitly labeled
- corrupted or unverifiable regions are excluded from trust-bearing outputs

The system does not perform heuristic reconstruction or inference.

## Extraction Safety

To prevent filesystem compromise:
- paths are normalized prior to extraction
- absolute paths are rejected
- traversal sequences are rejected
- outputs are constrained to the intended destination
- affected entries are refused or skipped when safe materialization cannot be guaranteed

No archive input is allowed to influence filesystem behavior outside defined boundaries.

## Determinism

crushr aims for deterministic behavior:
- identical inputs produce identical trust decisions
- validation results are consistent across runs
- structured output is stable and machine-readable where contracts define it

This supports reproducibility, auditing, and forensic-style analysis after damage.

## Architectural Invariants

The system is governed by explicit invariants, including:
- no unverified data may be returned
- no silent data loss
- no heuristic reconstruction
- all failures must be observable and classifiable

These invariants are treated as constraints, not suggestions.

## Security Posture

crushr is designed for environments where:
- data integrity matters more than convenience
- input cannot be trusted
- failure must be explicit and diagnosable
- post-damage reasoning must be bounded and honest

The system prioritizes correctness and transparency over convenience-first behavior.

## Alignment with ISO/IEC 27001 Control Principles

crushr is **designed in alignment with ISO/IEC 27001 control principles (self-assessed)** for the subset of controls that meaningfully apply to a single-maintainer, open-source archive project.

This is an engineering and documentation claim, not a certification claim.

In practical terms, the project maintains public evidence for:
- defined trust boundaries and risk treatment
- policy-governed fail-closed behavior
- access and release authority boundaries
- incident handling expectations
- explicit verification and audit-oriented outputs
- documented architectural invariants and change-discipline expectations

Relevant supporting documents include:
- `docs/security/SECURITY_POLICY.md`
- `docs/security/THREAT_MODEL.md`
- `docs/security/RISK_REGISTER.md`
- `docs/security/SOA.md`
- `docs/security/ACCESS_CONTROL.md`
- `docs/security/INCIDENT_RESPONSE.md`
- `docs/security/ARCHITECTURAL_INVARIANTS.md`
- `docs/security/VERIFICATION_SPEC.md`
- `docs/security/SECURITY_ARCHITECTURE.md`

This alignment is scoped and self-assessed. It does **not** claim:
- ISO/IEC 27001 certification
- formal external audit
- applicability of every Annex A control to this project’s scope

## Limitations

crushr does not provide:
- confidentiality guarantees
- availability guarantees under adversarial conditions
- full recovery from corruption

Its guarantees are limited to the correctness of what it verifies and the honesty of what it refuses to claim.

## Conclusion

crushr is not designed to “try its best.”

It is designed to define, as precisely as possible, what remains trustworthy after damage and to fail clearly when that boundary is crossed.
