# crushr — Project State

crushr is an integrity-first archival container for deterministic corruption analysis.

## Canonical product thesis

- Integrity-first archive design.
- Strict extraction semantics: extract verified-safe content, refuse unsafe content.
- Deterministic corruption impact reporting and experimentability.
- No speculative recovery, reconstruction, or automatic repair.

## Active toolchain

Active tools:

- `crushr-pack`
- `crushr-info`
- `crushr-fsck`
- `crushr-extract`
- `crushr-lab`

Active libraries:

- `crushr-format`
- `crushr-core`

`crates/crushr/` remains in-repo but is not authority for project direction.

## Archive/verification model

Minimal v1 layout:

`BLK3 blocks -> (optional DCT1) -> IDX3 -> FTR4`

Verification progression:

`FTR4 -> tail frame -> IDX3 -> BLK3 payload/hash checks -> per-file impact`

## Current implementation scope

- regular files only
- one block per file
- deterministic strict extraction reporting (`safe_files` / `refused_files`)

## Phase status

- Phase 1: complete.
- Cleanup packet series: complete.
- Phase 2: active engineering focus.
- Next required milestone: Phase 2.2 cross-format comparison and normalized result mapping.
- Next packet after audit: Phase 2.2 cross-format comparison and normalized result mapping.
- Canonical Phase 2 research workspace root: `PHASE2_RESEARCH/`.

## Phase-2 Evaluation Scope

Phase-2 trials evaluate the **baseline crushr format implementation**.

The following planned capabilities are intentionally excluded from the baseline evaluation:

- recoverable archive extraction
- true random-access extraction
- built-in deduplication

These capabilities remain planned future evolution features and will be evaluated after the white-paper trials.

One feature is included before trials:

- deterministic archive generation (minimal reproducibility rules)

## Deterministic Archive Generation (Pre-Paper Feature)

crushr archives should be reproducible.

Minimal deterministic rules:

1. deterministic file ordering
2. normalized timestamps
3. normalized permissions
4. deterministic compression parameters
5. deterministic metadata ordering

Goal:

Identical logical inputs should produce bit-identical archives.

This improves:

- reproducibility of white-paper datasets
- supply-chain verification
- artifact hashing and trust

Implementation must not alter archive structure or corruption semantics.

## White-Paper-Critical Research Capability

Phase 2 requires a formal experimental evidence pipeline.

Required components:

1. locked scenario manifest
2. raw per-run execution record
3. normalized result schema
4. trial completeness audit
5. reproducibility metadata

Purpose:

- every experimental scenario has a deterministic ID
- every run produces machine-readable records
- every summary result can be traced back to raw execution
- missing or duplicate runs are detected automatically
- the experiment can be rerun later with identical inputs

This capability governs research methodology only. It does not modify the crushr archive format.

## Planned Post-Paper Feature Set

The following capabilities are preserved as planned future work and are intentionally deferred until after the baseline white-paper trials:

- recoverable archive extraction
- true random-access extraction
- built-in deduplication

These are roadmap items, not features of the baseline implementation under current evaluation.

## V2 Architectural Direction

The v2 direction is now locked at a high level:

- content-addressed block identity
- deterministic on-disk indexing over content identities
- file records referencing verified block identities rather than positional-only storage

This direction is intended to support later implementation of:

- deduplication
- random-access extraction
- recoverable extraction

Detailed v2 design remains separate from the baseline white-paper implementation and must not be allowed to create ambiguity in current Phase-2 evaluation scope.
