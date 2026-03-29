# Risk Register

| ID | Risk | Impact | Likelihood | Mitigation |
|----|------|--------|------------|------------|
| R1 | Undetected archive corruption | High | Medium | Explicit integrity verification and fail-closed behavior |
| R2 | Path traversal during extraction | High | Medium | strict path normalization, reject absolute and escaping paths |
| R3 | Silent partial recovery | High | Medium | explicit salvage mode, mandatory reporting, fail-closed defaults |
| R4 | Non-deterministic builds or outputs | Medium | Medium | reproducible process, stable output contracts, verification |
| R5 | Metadata inconsistency | Medium | Low | validation during parsing and structural checks |
| R6 | Malicious archive payloads | High | Medium | treat all archives as untrusted, validate before use |
| R7 | Reporting gaps during failure | Medium | Low | structured outputs and explicit error signaling |
| R8 | Developer change weakening guarantees | High | Medium | invariant-aware review discipline and regression testing |

## Notes
- Risks are evaluated qualitatively due to project scale.
- Mitigations are expected to exist in code and documentation, not policy alone.
