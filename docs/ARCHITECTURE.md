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
  - emits primary IDX3 mapping plus compact LDG1 redundant file-map metadata (`crushr-redundant-file-map.v1`) for mapping survivability
- `crushr-info`: read/report archive state
- `crushr-fsck`: verify/analyze corruption and emit bounded diagnostics
- `crushr-extract`: strict safe extraction + refusal reporting
- `crushr-lab`: experiment orchestration support
- `crushr-lab-salvage`: salvage experiment orchestration (`experiment_manifest.json` + per-run metadata + compact `summary.json`/`summary.md` + compact grouped `analysis.json`/`analysis.md`), delegating salvage execution to deterministically resolved `crushr-salvage`; archive discovery is format-identity based (not extension-based) and supports summary/analysis regeneration via `--resummarize <experiment_dir>`

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


## Redundant map fallback path (CRUSHR-FORMAT-01)

- Primary mapping authority remains IDX3.
- New archives include compact redundant per-file extent mapping in LDG1 JSON (`crushr-redundant-file-map.v1`).
- `crushr-salvage` may use this metadata only when IDX3 mapping is unavailable/invalid and redundant metadata fully verifies.
- Verification is strict and all-or-nothing: schema validity, unique paths, contiguous/non-overlapping extents, exact file-size coverage, mapped block references, and per-extent bounds against verified block raw lengths.
- If redundant metadata is missing/corrupt/inconsistent, salvage does not guess mappings and remains in orphan-evidence outcomes where appropriate.

## Redundant-map comparison workflow (CRUSHR-SALVAGE-08)

- `crushr-lab-salvage run-redundant-map-comparison --output <comparison_dir>` runs a bounded deterministic targeted comparison corpus.
- For each scenario it compares: old-style archive path (redundant metadata removed) vs new-style archive path (redundant metadata present).
- Corruption coverage is intentionally bounded across dataset classes (`smallfiles`, `mixed`, `largefiles`), targets (header/index/payload/tail), and magnitudes (small/medium).
- Output remains compact (`comparison_summary.json`, `comparison_summary.md`) and research-only.
- This workflow does not weaken strict extraction or introduce speculative recovery.
