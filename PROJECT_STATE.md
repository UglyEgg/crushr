# PROJECT_STATE

## Product thesis (active)

crushr is an integrity-first archive system.

Non-negotiable baseline:

- strict extraction only for `crushr-extract`
- deterministic verification and reporting
- no speculative recovery/reconstruction in canonical extraction

## Active tool boundary

- `crushr-pack` — archive creation
- `crushr-info` — archive inspection/reporting
- `crushr-fsck` — strict verification/corruption analysis
- `crushr-extract` — strict verified extraction with deterministic refusal reporting
- `crushr-lab` — controlled research harness
- `crushr-salvage` — separate experimental salvage-planning executable (unverified research output only)

`crushr-salvage` must not change or weaken `crushr-extract` semantics.

## Current implementation scope

- regular files only
- one block per file in canonical v1 behavior
- deterministic strict extraction reporting (`safe_files` / `refused_files`)

## Phase status (authoritative summary)

- Phase 1: complete.
- Phase 2 execution matrix: complete and frozen.
- Phase 2 normalization: complete and frozen.
- Phase 2 comparison/ranking analysis: complete and frozen.
- Current experimental direction after FORMAT-05: self-identifying payload blocks with repeated verified path checkpoints are the best-performing bounded experimental recovery arm so far.
- Next active packet: **CRUSHR-FORMAT-06** verified file manifest checkpoints.

Canonical Phase 2 workspace root remains `PHASE2_RESEARCH/`.

## Locked resilience direction

Two architectural locks are now active for resilience-oriented experimental work:

1. **Inversion principle**
   - prefer verified payload-adjacent structures as reconstructive truth
   - treat centralized metadata as an accelerator, not sole authority
   - build recovery upward from surviving verified payload rather than downward from fragile roots

2. **Content-addressed recovery graph direction**
   - payload truth
   - extent/block identity truth
   - file manifest truth
   - path truth

Recovery should degrade in reverse order:
- full named recovery
- full anonymous recovery
- partial ordered recovery
- orphan evidence

## Active experimental boundary

### `CRUSHR-FORMAT-05` boundary

- Adds explicit experimental writer flag: `crushr-pack --experimental-self-identifying-blocks` (opt-in only).
- Experimental archives emit per-block payload identity records (`crushr-payload-block-identity.v1`) and repeated verified path checkpoints (`crushr-path-checkpoint.v1`) in separated regions.
- `crushr-salvage` fallback precedence is extended with payload identity recovery after file-identity extent recovery: `PRIMARY_INDEX_PATH` → `REDUNDANT_VERIFIED_MAP_PATH` → `CHECKPOINT_MAP_PATH` → `FILE_IDENTITY_EXTENT_PATH` → `PAYLOAD_BLOCK_IDENTITY_PATH` → `SELF_DESCRIBING_EXTENT_PATH`.
- Named recovery requires verified path checkpoint linkage; deterministic anonymous verified naming is used otherwise (`anonymous_verified/file_<file_id>.bin`) with `PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS` provenance.
- Added `crushr-lab-salvage run-format05-comparison` and required `format05_comparison_summary.json/.md` outputs for bounded five-arm targeted comparisons.

### `CRUSHR-FORMAT-06` next-step boundary

- FORMAT-06 will add verified file manifest checkpoints as the next graph layer on top of payload block identity.
- Purpose: establish **file truth** (file size, expected members/ordinals, completeness) independent from centralized IDX3 survival.
- It must improve confidence for:
  - full named recovery
  - full anonymous recovery
  - partial ordered recovery
- It remains experimental and opt-in only.

## Deferred-not-active research directions

These remain explicitly deferred until payload identity + file manifest truth have been tested:

- deterministic distributed-hash checkpoint placement
- deterministic low-discrepancy / golden-ratio checkpoint placement
- generalized graph-engine abstraction beyond bounded packet needs

## Out-of-scope invariants (unchanged)

- no speculative stitching/reconstruction
- no guessed byte emission
- no archive mutation in place
- no integration of experimental recovery semantics into `crushr-extract`

## 2026-03-15 security hardening update (CRUSHR-SCRUB-01)
- Extraction path confinement is now an explicit locked security boundary across canonical, legacy, and API extraction surfaces.
- Shared archive-path validation rejects absolute paths, parent traversal, empty/degenerate paths, and Windows-style path prefixes; unsafe paths now hard-fail deterministically.
- Hardened mode rejects symlink extraction to avoid reintroducing escape semantics.


## 2026-03-15 pack hardening update (CRUSHR-SCRUB-02)
- `crushr-pack` now rejects duplicate final logical archive paths before archive emission (hard fail, deterministic error, no auto-rename).
- Duplicate detection runs after logical path normalization (`\` → `/`) and reports colliding logical path plus conflicting source inputs.
- On duplicate collision, no archive output file is created.


## 2026-03-15 extraction authority alignment update (CRUSHR-PLAN-LEGACY-01)
- Supported extraction surface is now explicit and singular: `crushr-extract` strict extraction.
- Legacy extraction entry points in root `crushr` CLI (`crushr extract`) and `crates/crushr/src/api.rs` (`extract_all`) are quarantined and return explicit unsupported errors instead of silently using legacy semantics.
- Regression tests now guard both quarantine paths so supported extraction behavior cannot silently drift back to legacy extraction semantics.


## 2026-03-15 extraction authority delegation follow-up (CRUSHR-PLAN-LEGACY-01-f2)
- Preferred implementation is now applied: root `crushr extract` and API `extract_all` delegate to the same strict authoritative extraction implementation used by `crushr-extract`.
- Legacy extraction surfaces are no longer quarantined; they are compatibility entry points with strict semantics parity.
- Integration coverage now proves both root `crushr extract` and canonical `crushr-extract` roundtrip correctly from canonical `crushr-pack` archives.


## Update: CRUSHR-FORMAT-08 complete
- Added experimental metadata placement strategy selection for graph-supporting metadata checkpoints: `fixed_spread`, `hash_spread`, `golden_spread`.
- Strategy scope is limited to metadata layers (path checkpoints + file manifest checkpoints); payload layout semantics are unchanged.
- Added bounded `run-format08-placement-comparison` workflow; Phase-09 will apply a richer corruption grid on the same strategy surfaces.
