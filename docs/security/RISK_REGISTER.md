<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Risk Register

## Intent

This page records the primary qualitative risks relevant to crushr's trust and extraction model.

## Guarantees

- Risks are documented explicitly rather than implied informally
- Mitigations must map to code, policy, or review discipline
- Missing controls are visible rather than silently assumed
- Risk language must not contradict the published trust model
- Recovery and failure behavior remain auditable

## Risks

| ID | Risk | Impact | Likelihood | Mitigation |
|----|------|--------|------------|------------|
| R1 | Undetected archive corruption | High | Medium | explicit integrity verification and fail-closed behavior |
| R2 | Path traversal during extraction | High | Medium | strict path normalization, reject absolute and escaping paths |
| R3 | Silent non-canonical recovery | High | Medium | explicit recover mode, mandatory reporting, fail-closed defaults |
| R4 | Non-deterministic builds or outputs | Medium | Medium | reproducible process, stable output contracts, verification |
| R5 | Metadata inconsistency | Medium | Low | validation during parsing, reference checks, and explicit classification |
| R6 | Malicious archive payloads | High | Medium | treat all archives as untrusted, validate and verify before use |
| R7 | Reporting gaps during failure | Medium | Low | structured outputs and explicit error signaling |
| R8 | Developer change weakening guarantees | High | Medium | invariant-aware review discipline and regression testing |

## Notes

- Risks are evaluated qualitatively due to project scale
- Mitigations are expected to exist in code and documentation, not policy alone
- Risk language must remain consistent with the canonical recovery vocabulary
