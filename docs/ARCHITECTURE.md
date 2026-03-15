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
  - `--help` now prints bounded usage including experimental writer modes
  - duplicate final logical archive paths are rejected before archive emission (hard fail; no rename/overwrite semantics)
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
  - logical path normalization is deterministic (`\` → `/`) and duplicate detection runs against normalized final paths

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


## Experimental resilience path (CRUSHR-FORMAT-02)

- `crushr-pack --experimental-self-describing-extents` is an explicit opt-in writer path.
- Experimental archives add:
  - `crushr-self-describing-extent.v1` metadata blocks colocated through the payload region.
  - `crushr-checkpoint-map-snapshot.v1` metadata blocks written at multiple separated positions (periodic checkpoints plus end checkpoint).
- `crushr-salvage` uses these only when primary IDX3 mapping is unusable and metadata verifies strictly.
- Deterministic fallback precedence: `PRIMARY_INDEX_PATH` → `REDUNDANT_VERIFIED_MAP_PATH` (when present/valid) → `CHECKPOINT_MAP_PATH` → `SELF_DESCRIBING_EXTENT_PATH`.
- No speculative reconstruction, guessed mappings, or changes to `crushr-extract` semantics.


## Experimental file-identity path (CRUSHR-FORMAT-03)

- `crushr-pack --experimental-file-identity-extents` is explicit opt-in and does not alter default writer behavior.
- Experimental archives add per-extent `crushr-file-identity-extent.v1` records: `file_id`, logical offset/length, full file size, extent ordinal, block id, content identity hashes, and path digest linkage.
- Path names are recovered only when `crushr-file-path-map.v1` verifies (`file_id` + `path` + `path_digest_blake3` matches computed digest).
- Salvage precedence: `PRIMARY_INDEX_PATH` → `REDUNDANT_VERIFIED_MAP_PATH` → `CHECKPOINT_MAP_PATH` → `FILE_IDENTITY_EXTENT_PATH` → `SELF_DESCRIBING_EXTENT_PATH`.
- Strict boundary unchanged: no guessed names, offsets, extents, or speculative reconstruction.


## Experimental resilience path (CRUSHR-FORMAT-04)

- `crushr-pack --experimental-file-identity-extents` now emits distributed bootstrap anchors (`crushr-bootstrap-anchor.v1`) and per-entry path map records (`crushr-file-path-map-entry.v1`) alongside file-identity extent records.
- `crushr-salvage` can deterministically recover via verified metadata scanning when footer/index are unavailable, with explicit `bootstrap_anchor_analysis` diagnostics.
- Path recovery rule: named recovery when verified path-map linkage exists; otherwise deterministic anonymous verified naming (`anonymous_verified/file_<file_id>.bin`) with `FILE_IDENTITY_EXTENT_PATH_ANONYMOUS` provenance.
- Strict boundary remains unchanged: verification-only, no speculative reconstruction, no guessed names/offsets/extents.


## Experimental payload-block identity path (CRUSHR-FORMAT-05)

- Writer surface: `crushr-pack --experimental-self-identifying-blocks` (opt-in only).
- Payload block metadata (`crushr-payload-block-identity.v1`) includes archive identity token, file identity, block index/total, full file size, logical offset/length, codec, payload length, scan offset, and payload/raw hash bindings.
- Repeated path checkpoints (`crushr-path-checkpoint.v1`) carry `file_id`, canonical path bytes, path digest, full file size, and total block count; checkpoints are emitted in separated regions (early/mid/late + final checkpoint).
- Salvage fallback precedence: `PRIMARY_INDEX_PATH` → `REDUNDANT_VERIFIED_MAP_PATH` → `CHECKPOINT_MAP_PATH` → `FILE_IDENTITY_EXTENT_PATH` → `PAYLOAD_BLOCK_IDENTITY_PATH` → `SELF_DESCRIBING_EXTENT_PATH`.
- Recovery remains strict: named recovery only with verified checkpoint linkage, deterministic anonymous verified recovery otherwise, and no guessed names/offsets/ordering.

## Extraction confinement boundary (CRUSHR-SCRUB-01)

All file-materializing extraction surfaces now route through a shared confinement utility (`extraction_path::resolve_confined_path`).

Enforced rules:
- archive entry path must be non-empty and relative
- absolute paths are rejected
- parent traversal (`..`) is rejected
- path-prefix/drive-style forms are rejected
- resulting destination must remain under the extraction root

Policy: unsafe paths hard-fail (no rewrite/strip/rename).

Symlink policy in hardened mode: extraction rejects symlink entries (fail closed).
