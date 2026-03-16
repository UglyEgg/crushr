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

## Phase 3 — Standalone Salvage Planning and Recovery-Graph Research (active)

- [x] 3.1 CRUSHR-SALVAGE-01 (plan-only)
- [x] 3.2 CRUSHR-SALVAGE-02 (verified block analysis, still plan-only)
- [x] 3.3 CRUSHR-SALVAGE-03 (verified fragment export, research-only)
- [x] 3.4 CRUSHR-SALVAGE-04 (deterministic salvage experiment harness, research-only)
- [x] 3.5 CRUSHR-SALVAGE-05 (deterministic compact experiment summaries, research-only)
- [x] 3.6 CRUSHR-SALVAGE-06 (deterministic grouped analysis views, research-only)
- [x] 3.7 CRUSHR-SALVAGE-07 (harness hardening for deterministic local research execution)
- [x] 3.8 CRUSHR-FORMAT-01 (redundant verified file-map metadata fallback)
- [x] 3.9 CRUSHR-SALVAGE-08 (bounded redundant-map before/after comparison, research-only)
- [x] 3.10 CRUSHR-FORMAT-02 (experimental self-describing extents + distributed checkpoints)
- [x] 3.11 CRUSHR-FORMAT-03 (experimental file-identity anchored extents)
- [x] 3.12 CRUSHR-FORMAT-03-f1 (lab-salvage comparison dispatch/help repair)
- [x] 3.13 CRUSHR-FORMAT-03-f2 (packer experimental writer/help contract repair)
- [x] 3.14 CRUSHR-FORMAT-04 (experimental bootstrap-anchor + file-identity survivability hardening)
- [x] 3.15 CRUSHR-FORMAT-05 (self-identifying payload blocks + repeated path checkpoints + bounded format05 comparison)
- [x] 3.15-f1 CRUSHR-FORMAT-05-f1 (format05 comparison runner/packer flag-contract + packer-bin resolution hardening + regression coverage)
- [x] 3.15-f2 CRUSHR-FORMAT-05-f2 (remove `crushr-pack --help` probe dependency from format05 comparison; enforce direct writer-flag contract + regression coverage)
- [x] 3.15-f3 CRUSHR-FORMAT-05-f3 (convert format05 contract regressions to behavioral runner/packer shim checks; self-check contract mismatch guard)
- [x] 3.15-f4 CRUSHR-FORMAT-05-f4 (restore runnable `cargo run` format05 comparison command by auto-building unresolved sibling salvage/pack binaries)
- [x] 3.16 CRUSHR-SCRUB-02 (reject duplicate logical archive paths at pack time before archive emission)
- [x] 3.16-f1 CRUSHR-SCRUB-02-f1 (stabilize duplicate-collision source ordering + expand collision-mode test coverage)
- [x] 3.16-f2 CRUSHR-SCRUB-03 (modularize `crushr-salvage` + `crushr-lab-salvage` internals; preserve behavior + add regression coverage)
- [x] 3.17 CRUSHR-FORMAT-06 (verified file manifest checkpoints as the next recovery-graph layer)
  - build the file-truth layer on top of payload block identity
  - validate full named / full anonymous / partial ordered recovery rules
  - target header/index/tail cases that FORMAT-05 still leaves as orphan evidence
  - keep the work experimental and opt-in only
- [x] 3.17 CRUSHR-FORMAT-07 (graph-aware salvage reasoning + explicit recovery-class determination + format07 comparison command wiring)
  - build verified relationship graph from surviving records (block->extent->manifest->path)
  - classify recovery from connected verified evidence using ordered FORMAT-07 classes
  - extend lab comparison harness with `run-format07-comparison` JSON/Markdown outputs and delta reporting
- [ ] 3.18 Future: placement-strategy experiments (deferred)
  - deterministic distributed-hash checkpoint placement
  - deterministic low-discrepancy / golden-ratio placement
  - only after recovery-graph layers stabilize enough for a meaningful bakeoff


## Security scrub track
- [x] CRUSHR-SCRUB-01 extraction confinement hardening (shared validator + canonical/legacy/API alignment + hostile tests)
- [x] CRUSHR-PLAN-LEGACY-01 extraction authority boundary enforcement (canonical strict surface lock + legacy API/CLI extraction quarantine + regression tests)
- [x] CRUSHR-PLAN-LEGACY-01-f1 post-review test clarity hardening (rename quarantine MVP test + add positive canonical `crushr-extract` roundtrip integration coverage)
- [x] CRUSHR-PLAN-LEGACY-01-f2 preferred extraction authority alignment (root/API delegation to strict authoritative implementation + regression integration updates)

- [x] CRUSHR-FORMAT-06-f1 — manifest-sourced plan synthesis + digest-aware classification + format06 classification deltas

- [x] 3.17-f1 CRUSHR-FORMAT-06-f1 dispatch regression fix (lock top-level `run-format06-comparison` dispatch behavior + subcommand-vs-path regression coverage + help discoverability assertion)
