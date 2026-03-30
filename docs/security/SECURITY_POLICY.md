<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Security Policy

## Intent

This policy defines the governing security principles for the design, build, release, and verification behavior of crushr.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Scope

This policy applies to:

- crushr archive format and canonical tooling
- build and release pipeline
- artifact integrity and verification mechanisms
- inspection, verification, extraction, and recovery behavior

### Principles

#### 1. Integrity First
crushr prioritizes verifiable correctness over convenience, performance, or compression ratio.

#### 2. Fail-Closed Behavior
Operations must fail explicitly when required truth cannot be established. Silent degradation is prohibited.

#### 3. Determinism
Builds, outputs, and verification behavior must be reproducible and consistent where the contract defines them.

#### 4. Explicit Trust Boundaries
All inputs are treated as untrusted unless the checks required for the requested operation succeed.

#### 5. No Speculative Recovery
Corrupted or unverifiable data is never reconstructed heuristically. Only verified payload data may enter trust-bearing recovery outputs.

#### 6. Auditability
Critical operations produce machine-readable output suitable for verification, review, and incident analysis.

## Responsibilities

- **Maintainer** — defines and enforces security controls
- **Contributors** — must not introduce behavior that weakens integrity guarantees, trust classification, or fail-closed semantics

## Boundaries / Non-goals

This policy does not claim certification and does not authorize repair behavior, inferred trust, or undocumented recovery paths.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Review

This policy is reviewed periodically or after major architectural changes.
