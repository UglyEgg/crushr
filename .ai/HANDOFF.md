# Handoff

Current boundary update (2026-03-17):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.

Next focus:
- CRUSHR-HARDEN-03B contract repair is complete: salvage output now aligns with `crushr-salvage-plan.v3` enums and typed reason-code vocabulary.
- Continue CRUSHR-HARDEN-03 follow-ups to normalize any remaining lab/report legacy classification labels (`*_VERIFIED`, `ORPHAN_EVIDENCE_ONLY`, `NO_VERIFIED_EVIDENCE`) where they still appear outside the salvage-plan contract path.
