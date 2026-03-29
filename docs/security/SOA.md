# Statement of Applicability (SoA)

This document maps selected **ISO/IEC 27001-style control principles** to their implementation in crushr.

Scope notes:
- this mapping is **self-assessed**
- it is limited to controls that meaningfully apply to a single-maintainer, open-source archive project
- it is not a certification statement and does not claim full Annex A applicability

## Governance, Policy, and Risk Treatment
Applicable: Yes  
Implementation:
- published security policy and architectural invariants
- documented threat model and qualitative risk register
- internal audit and review artifacts
Justification:
- the project needs explicit boundaries for what it guarantees, what it refuses to claim, and how change is governed

## Access Control
Applicable: Yes  
Implementation:
- controlled repository access
- release authority limited to maintainer
- contributor changes mediated through review paths
Justification:
- prevent unauthorized modification of source, release process, and published artifacts

## Cryptographic and Integrity Controls
Applicable: Yes  
Implementation:
- explicit integrity verification in the archive model
- current implementation uses BLAKE3 for integrity checks
- verified-data boundary enforced in strict and recovery-capable flows
Justification:
- integrity is a core system guarantee rather than an optional feature

## Secure Development
Applicable: Yes  
Implementation:
- deterministic build and output expectations where contracts define them
- explicit error handling and fail-closed logic
- invariant-aware review discipline
Justification:
- prevent undefined, ambiguous, or guarantee-eroding behavior

## Logging, Monitoring, and Auditability
Applicable: Yes  
Implementation:
- structured outputs from inspection, verification, and recovery tooling
- deterministic reporting of corruption and failure conditions
- machine-readable classification of outcomes and exit behavior
Justification:
- enables auditability, incident analysis, and post-failure reasoning

## Incident Management
Applicable: Yes  
Implementation:
- defined failure modes and incident-response expectations
- explicit error signaling via exit codes and structured output
- documented containment and analysis expectations
Justification:
- failures must be diagnosable and non-ambiguous

## Operational Resilience / Business Continuity
Applicable: Limited  
Implementation:
- inspection and bounded recovery tooling
- verified extraction only in strict mode
- explicit degraded handling in recover-capable paths
Justification:
- the project focuses on data-integrity reasoning and post-damage analysis, not service uptime or operational availability guarantees

## Supplier / Dependency Risk
Applicable: Limited  
Implementation:
- preference for deterministic, auditable components
- minimize trust surface where possible
- avoid hidden external dependencies in trust-bearing decisions
Justification:
- reduce external trust requirements and opaque risk transfer

## Human Resource Security
Applicable: Limited / Not primary  
Implementation:
- single-maintainer operating model
- review discipline documented through policy and invariants rather than staffing controls
Justification:
- personnel controls are not a meaningful primary control domain at current project scale

## Physical Security
Applicable: No  
Implementation:
- not infrastructure-bound within project scope
Justification:
- project scope is software design and tooling behavior, not hosting or facility controls
