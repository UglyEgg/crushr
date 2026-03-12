# crushr Development Status

Current Phase: Phase 2 — Comparative Corruption Research

Current Step: 2.1 packet preparation (controlled corruption matrix manifest/schema)

## Current truth

- Phase 1 is complete.
- Cleanup packet CRUSHR-CLEANUP-2.0-C is complete.
- Active machine-readable schemas are now tightened contracts for: `crushr-info` snapshot, `crushr-fsck` snapshot, `crushr-impact` report, extraction result, and propagation graph.
- Integration tests now perform real JSON Schema instance-vs-schema validation for active outputs.

## Active constraints

- Minimal v1 scope: regular files, one block per file.
- No speculative recovery/reconstruction/repair.
- Strict extraction semantics remain canonical.
- `schemas/crushr-impact.v1.schema.json` remains active as a nested contract dependency (used by fsck blast-radius payload and direct impact report typing); no obsolete schema was deleted in this packet.

## Next action

Start and execute Phase 2.1 manifest/schema packet for controlled corruption matrix runs.
