# crushr Phase Plan

## Phase 1 — Integrity Intelligence

- [x] 1.1 Corruption Propagation Graph
- [x] 1.1 hardening closeout (CRUSHR-1.1-B)
- [x] 1.2 Maximum Safe Extraction Formalization
- [x] 1.3 Extraction Result Formalization

## Phase 2 — Comparative Corruption Research

- [x] 2.0-A Legacy recovery/salvage surface deletion
- [x] 2.0-B Documentation/control cleanup and canonicalization
- [x] 2.0-C Schema contract tightening and validator-backed tests
- [x] 2.0-D Shared core report/classification centralization
- [x] 2.1-A Controlled Corruption Matrix manifest/schema (CRUSHR-P2.1-A)
- [x] 2.1-B Dataset fixture + archive build foundation (CRUSHR-P2.1-B)
- [x] 2.1-C Deterministic corruption injection engine (CRUSHR-P2.1-C)
- [x] 2.1-D Phase 2 execution runner + raw result capture + completeness auditing (CRUSHR-P2.1-D)
- [x] 2.1 pre-trial reproducibility prep — deterministic baseline archive generation (CRUSHR-P2-PRETRIAL-DET-01)
- [x] 2.1 pre-trial audit milestone (CRUSHR-P2-TRIAL-READY-01)
- [x] 2.1 execution freeze + foundation + execution evidence (CRUSHR-P2-EXEC-01/02/03A/03B)
- [x] 2.2 execution normalization + recovery accounting enrichment (CRUSHR-P2-EXEC-04/06A)
- [x] 2.2 cross-format comparison (CRUSHR-P2-ANALYSIS-01)

## Phase 3 — Standalone Salvage Planning (active)

- [x] 3.1 CRUSHR-SALVAGE-01 (plan-only)
- [x] 3.2 CRUSHR-SALVAGE-02 (verified block analysis, still plan-only)
  - standalone executable: `crushr-salvage`
  - deterministic salvage planning over damaged archives
  - machine-readable salvage plan JSON output
  - deterministic BLK3 candidate scan + authoritative-mapping-aware file classification
  - no speculative recovery
  - no guessed reconstruction
  - no fragment emission/output carving directories
  - no archive mutation

- [x] 3.3 CRUSHR-SALVAGE-03 (verified fragment export, research-only)
  - `--export-fragments <dir>` optional artifact emission
  - deterministic block/extent/full-file export gating on content verification
  - SALVAGE_RESEARCH_OUTPUT marker + `UNVERIFIED_RESEARCH_OUTPUT` sidecar labels
  - optional `exported_artifacts` section in salvage-plan v2 output
