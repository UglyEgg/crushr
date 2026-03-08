# Snapshot Stability Contract

JSON snapshots are versioned contracts consumed by tools and the TUI.

Rules:
- snapshots carry a schema version
- snapshots carry an archive fingerprint
- snapshots may only be merged when archive fingerprints match
- additive evolution is preferred; breaking changes require a version bump
