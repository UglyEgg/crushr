# Handoff

Current boundary update (2026-03-17):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.

Next focus:
- Continue CRUSHR-HARDEN-03 CLI minimization and remove remaining internal fsck snapshot/schema sediment once downstream dependencies are migrated.
