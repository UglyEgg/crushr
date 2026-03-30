<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Security

## Intent

This section defines crushr's security and assurance boundary.

crushr is a deterministic archive system that preserves and exposes data truth under failure. The purpose of this section is to make that claim operational: what the system trusts, what it verifies, what it refuses to infer, and how recovery remains bounded and explicit.

This is not a certification claim. It is a public description of trust boundaries, guarantees, and self-assessed control alignment within the project's actual scope.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

This security set defines:

- the architectural trust boundary between validation, verification, and extraction
- the conditions under which strict extraction is permitted or refused
- the conditions under which `extract --recover` may emit explicitly classified output
- the control principles used to keep the project auditable, reviewable, and non-ambiguous

crushr treats all archives as untrusted input until structural correctness and payload integrity have been established for the requested operation.

## Self-assessed control alignment

crushr is designed in alignment with ISO/IEC 27001 control principles (self-assessed) for controls relevant to a single-maintainer open-source archive project.

That alignment is evidenced through the published security set:

- policy and principle documents
- threat and risk documentation
- verification and recovery contracts
- access, release, and incident-handling boundaries
- architectural invariants and audit-style review material

This is an engineering-discipline claim, not a certification claim.

## Boundaries / Non-goals

This section does not claim certification, confidentiality guarantees, or operational availability guarantees beyond the documented archive behavior.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies
