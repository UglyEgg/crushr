# Architecture

This document describes the current implemented boundary without speculative maturity claims.

## System thesis alignment

- integrity-first archive semantics
- strict extraction (verified-safe only)
- deterministic corruption impact reporting
- no recovery/salvage/reconstruction path inside canonical `crushr-extract` semantics
- separate experimental salvage planning executable (`crushr-salvage`) is allowed as unverified research output

## Crate boundaries

- `crushr-format`
  - byte-level format encoding/decoding (`BLK3`, `IDX3`, optional `DCT1`, `FTR4`)
  - strict structural parsing helpers

- `crushr-core`
  - verification, structural interpretation, impact reporting, extraction result modeling
  - JSON/report models used by tools

- `crushr-cli-common`
  - shared CLI output/error helpers

- `crushr-tui`
  - live/snapshot visualization path (non-authoritative for format/contracts)

- `crushr-lab`
  - deterministic experiment harness support for corruption research

- `crushr`
  - legacy integration surface still present in-repo; not the authority for active Phase 2 direction

## Tool boundaries

- `crushr-pack`: create archives
- `crushr-info`: read/report archive state
- `crushr-fsck`: verify/analyze corruption and emit bounded diagnostics
- `crushr-extract`: strict safe extraction + refusal reporting
- `crushr-lab`: experiment orchestration support
- `crushr-lab-salvage`: salvage experiment orchestration (`experiment_manifest.json` + per-run metadata), delegating salvage execution to `crushr-salvage`

## Current implementation constraints

- minimal v1: regular files only
- one block per file
- deterministic refusal reporting for unsafe files
- baseline `crushr-pack` archive generation is deterministic for identical logical inputs:
  - files are archived in lexicographic relative-path order
  - timestamps are normalized to `mtime=0`
  - permissions are normalized to `mode=0`
  - xattrs are not emitted (empty metadata list)
  - zstd compression uses fixed encoder settings (single-thread, checksum off, content size on, dict id off)

## Active phase

- Phase 1 complete
- Phase 2 execution/normalization/comparison are complete and frozen
- active implementation workstream: standalone deterministic salvage planning (`crushr-salvage`)
- strict extraction semantics remain unchanged
- salvage verification/export in `crushr-salvage` is deterministic research evidence only and does not authorize canonical extraction
- salvage experiment orchestration remains research-only and must not mutate archives or frozen Phase 2 corpus artifacts
- optional `crushr-salvage --export-fragments <dir>` emits verified block/extents artifacts labeled `UNVERIFIED_RESEARCH_OUTPUT`
