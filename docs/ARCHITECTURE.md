# Architecture

This document describes the current implemented boundary without speculative maturity claims.

## System thesis alignment

- integrity-first archive semantics
- strict extraction (verified-safe only)
- deterministic corruption impact reporting
- no recovery/salvage/reconstruction product path

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

## Current implementation constraints

- minimal v1: regular files only
- one block per file
- deterministic refusal reporting for unsafe files

## Active phase

- Phase 1 complete
- Phase 2 active
- next packet: controlled corruption matrix manifest/schema
