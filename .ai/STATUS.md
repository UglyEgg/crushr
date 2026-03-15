# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-SCRUB-02-f1 complete** (duplicate-collision source ordering is now deterministic, with expanded collision-mode regression coverage)

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
