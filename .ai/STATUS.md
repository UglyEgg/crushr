# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-FORMAT-05-f4 complete** (format05 comparison now auto-builds sibling salvage/pack binaries when running via `cargo run` to preserve end-to-end command operability)

Immediate Next Step: **CRUSHR-FORMAT-06** (verified file manifest checkpoints as the next recovery-graph layer)

Security step note: **CRUSHR-SCRUB-01 complete** (extraction path confinement unified across canonical/legacy/API; unsafe paths now hard-fail; symlink extraction disabled in hardened mode).

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- Current experimental evidence says payload-adjacent file identity is the first real recovery direction that improved outcomes.
- The architectural direction is now locked toward a **content-addressed recovery graph**.
- The inversion principle is active for resilience work: prefer verified payload-adjacent truth over centralized metadata authority.
- FORMAT-05 comparison now runs end-to-end without relying on `crushr-pack --help`; the runner invokes the canonical writer flag directly.
- FORMAT-05 comparison now auto-builds sibling `crushr-pack`/`crushr-salvage` binaries when launched from `cargo run -p crushr --bin crushr-lab-salvage`, so the documented command executes end-to-end without manual binary path wiring.
- `crushr-pack` now rejects duplicate logical archive paths before any archive bytes are written; collisions are explicit hard failures listing the logical path and source inputs.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation.
- Comparison workflows remain bounded and storage-conscious; do not rerun the full Phase 2 matrix without explicit instruction.
- Distributed-hash / low-discrepancy checkpoint placement is deferred backlog research, not the active experiment.

## Active recovery-graph layering

1. payload truth
2. extent/block identity truth
3. file manifest truth
4. path truth

Recovery should degrade in reverse order:
1. full named recovery
2. full anonymous recovery
3. partial ordered recovery
4. orphan evidence

## Next actions

1. Preserve strict extraction interfaces/semantics untouched.
2. Keep experimental work focused on metadata-independent reconstruction.
3. Implement FORMAT-06 as the next graph layer: verified file manifest checkpoints.
4. Preserve deterministic comparison ordering/classification and compact grouped metrics outputs.
5. Keep Phase 2 corpus and frozen artifacts unchanged.
6. Treat payload identity + manifest truth as the active priority before checkpoint-placement strategy experiments.


## CRUSHR-SCRUB-01 closeout
- Added shared archive-path confinement helper used by all file-writing extraction surfaces.
- Canonical `crushr-extract`, legacy extraction, and API extraction now reject path escape inputs with explicit deterministic errors.
- Hardened mode rejects symlink extraction to prevent escape reintroduction.
- Added hostile tests for safe relative path, traversal rejection, absolute rejection, normalization escape rejection, public API alignment, legacy alignment, symlink rejection, and root confinement regression.


## CRUSHR-SCRUB-02 closeout
- Added deterministic duplicate logical-path detection in `crushr-pack` after canonical logical-path normalization and before output file creation.
- Packing now hard-fails on collisions with explicit error text containing colliding logical archive path and conflicting source inputs.
- Added focused packer tests for success on distinct paths, basename collision failure, normalization-only collision failure, walked-tree collision failure, three-way collision failure, stable ordered error surface, and no partial archive emission.


## CRUSHR-SCRUB-02-f1 closeout
- Stabilized duplicate-collision source listing order by sorting input files with `(rel_path, abs_path)` and sorting conflicting source vectors before formatting errors.
- Added regression coverage for walked-tree vs walked-tree collisions and three-way collisions with explicit ordered-source error assertions.


## CRUSHR-PLAN-LEGACY-01 closeout
- Root `crushr extract` is now an explicit quarantined legacy surface and fails with a clear unsupported error for both all-entry and path-filtered modes.
- `crates/crushr/src/api.rs` extraction (`extract_all`) is now explicitly quarantined and returns a clear unsupported error instead of silently routing legacy semantics.
- Regression tests now cover root CLI quarantine behavior and mvp extraction flow now uses canonical `crushr-extract`, preventing silent fallback to legacy semantics.


## CRUSHR-PLAN-LEGACY-01-f1 closeout
- Renamed the MVP extraction test to match its true quarantine purpose for root `crushr extract`.
- Added a positive integration test that performs a real `crushr-pack` -> `crushr-extract` roundtrip and asserts extracted content.
- This preserves explicit authority boundary evidence: legacy root extract is quarantined while canonical `crushr-extract` still functions end-to-end.


## CRUSHR-PLAN-LEGACY-01-f2 closeout
- Replaced quarantine behavior with preferred delegation behavior: root `crushr extract` now executes authoritative strict extraction for both all-entry and path-filtered modes.
- API extraction (`extract_all`) now delegates to the same strict implementation instead of returning unsupported errors.
- Added/updated integration tests proving root and canonical extraction surfaces both roundtrip correctly from canonical `crushr-pack` archives.


## CRUSHR-SCRUB-03 closeout
- Decomposed `crushr-salvage` into internal modules (`cli`, `discovery`, `metadata`, `artifacts`) with behavior preserved.
- Decomposed `crushr-lab-salvage` into internal modules (`cli`, `runner`, `comparison`) with behavior preserved.
- Added regression test coverage for salvage binary resolution precedence while preserving existing deterministic ordering/comparison workflow tests.
