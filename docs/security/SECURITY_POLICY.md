# Security Policy

## Purpose
Define the security principles governing the design, build, and release of crushr.

## Scope
This policy applies to:
- crushr archive format and tooling
- build and release pipeline
- artifact integrity and verification mechanisms
- recovery, inspection, and salvage behavior

## Principles

1. Integrity First
crushr prioritizes verifiable correctness over convenience, performance, or compression ratio.

2. Fail-Closed Behavior
Operations must fail explicitly when integrity cannot be guaranteed. Silent degradation is prohibited.

3. Determinism
Builds, outputs, and verification processes must be reproducible and consistent where feasible.

4. Explicit Trust Boundaries
All inputs are treated as untrusted unless verified.

5. No Speculative Recovery
Corrupted data is never reconstructed heuristically. Only verified data is exposed.

6. Auditability
Critical operations produce machine-readable output suitable for verification and review.

## Responsibilities
- Maintainer: defines and enforces security controls
- Contributors: must not introduce behavior that weakens integrity guarantees

## Review
This policy is reviewed periodically or after major architectural changes.
