# crushr Development Status

Current Phase: Phase 2 — Comparative Corruption Research

Current Step: 2.1 complete (CRUSHR-P2.1-A manifest-first experiment contract and scenario enumeration)

Recent completed packet: CRUSHR-P2.1-A (manifest-first experiment contract and deterministic scenario enumeration)

## Current truth

- Phase 1 is complete.
- Phase 2.1 packet CRUSHR-P2.1-A is complete: `crushr-lab` now has typed Phase 2 manifest/scenario structures, deterministic locked-core scenario enumeration (2160 runs), stable scenario IDs, and a dedicated manifest schema.
- Cleanup packets CRUSHR-CLEANUP-2.0-C and CRUSHR-CLEANUP-2.0-D are complete.
- Active machine-readable schemas are now tightened contracts for: `crushr-info` snapshot, `crushr-fsck` snapshot, `crushr-impact` report, extraction result, and propagation graph.
- Integration tests now perform real JSON Schema instance-vs-schema validation for active outputs.

## Active constraints

- Minimal v1 scope: regular files, one block per file.
- No speculative recovery/reconstruction/repair.
- Strict extraction semantics remain canonical.
- `schemas/crushr-impact.v1.schema.json` remains active as a nested contract dependency (used by fsck blast-radius payload and direct impact report typing); no obsolete schema was deleted in this packet.

## Next action

Start Phase 2.2 execution harness packet (consume manifest scenarios for command execution and raw result capture; preserve deterministic ordering/IDs).
