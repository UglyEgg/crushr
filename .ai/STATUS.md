# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-FORMAT-05-f1 complete** (format05 comparison runner/packer flag contract repair + packer-help/contract regression tests)

Immediate Next Step: **CRUSHR-FORMAT-06** (verified file manifest checkpoints as the next recovery-graph layer)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
- Current experimental evidence says payload-adjacent file identity is the first real recovery direction that improved outcomes.
- The architectural direction is now locked toward a **content-addressed recovery graph**.
- The inversion principle is active for resilience work: prefer verified payload-adjacent truth over centralized metadata authority.

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
