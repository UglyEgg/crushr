# crushr Development Status

Current Phase: Phase 1 — Integrity Intelligence

Active Objective:

Step 1.1 — Corruption Propagation Graph (completed, fix iteration: lint debt cleanup)

Goal status:

Implemented deterministic minimal-v1 corruption propagation graph modeling and reporting via `crushr-info --json --report propagation`, with schema/docs/tests and extraction-consistency assertions.

What changed:

- Added `crushr-core::propagation` with a typed deterministic corruption propagation model and report builder for minimal-v1 required structures, block dependencies, and per-file impact causes.
- Workspace lint debt cleanup completed: `cargo clippy --workspace --all-targets -- -D warnings` now passes with lint-only code changes (no behavior/API changes).
- Extended `crushr-info` with `--report propagation` JSON output, backed by real IDX3 file/block dependencies and real payload-hash corruption detection.
- Added propagation contract documentation: `docs/CONTRACTS/PROPAGATION_GRAPH_V1.md`.
- Added propagation report schema: `schemas/crushr-propagation-graph.v1.schema.json`.
- Added integration tests for graph shape/determinism and consistency with extraction refusal behavior: `crates/crushr-core/tests/propagation_graph_v1.rs`.

Active constraints:

- Minimal v1 propagation graph remains limited to regular files and current required structures (`FTR4`, tail frame, `IDX3`) plus required BLK3 block IDs.
- No speculative recovery, reconstruction, repair, or hole-filling behavior exists.
- Strict mode remains default; salvage mode remains explicit.

Next actions:

- Proceed to next bounded milestone in Phase 1 (or Phase plan reorder decision if requested by maintainer).
- Keep new lint baseline green for workspace-wide clippy `-D warnings`.
