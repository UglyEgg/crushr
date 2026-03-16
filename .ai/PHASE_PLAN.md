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
- [x] 3.15 CRUSHR-FORMAT-05 (self-identifying payload blocks + repeated path checkpoints)
- [x] 3.15-f1 CRUSHR-FORMAT-05-f1
- [x] 3.15-f2 CRUSHR-FORMAT-05-f2
- [x] 3.15-f3 CRUSHR-FORMAT-05-f3
- [x] 3.15-f4 CRUSHR-FORMAT-05-f4
- [x] 3.16 CRUSHR-SCRUB-02 (duplicate logical path hard-fail before archive emission)
- [x] 3.16-f1 CRUSHR-SCRUB-02-f1 (stable collision ordering + expanded coverage)
- [x] 3.16-f2 CRUSHR-SCRUB-03 (salvage/lab-salvage internal modularization)
- [x] 3.17 CRUSHR-FORMAT-06 (verified file manifest checkpoints / file-truth layer)
- [x] 3.17-f1 CRUSHR-FORMAT-06-f1 (manifest-sourced plan synthesis + digest-aware classification + dispatch hardening)
- [x] 3.18 CRUSHR-FORMAT-07 (graph-aware salvage reasoning + explicit recovery classes)
- [x] 3.19 CRUSHR-FORMAT-08 (metadata placement strategy experiment: `fixed_spread`, `hash_spread`, `golden_spread`)
- [x] 3.20 CRUSHR-FORMAT-09 (curated corruption grid / survivability evaluation harness)
  - stress truth-layer loss rather than only coarse region damage
  - test named -> anonymous and ordered -> unordered downgrade behavior
  - measure whether weak duplicated metadata surfaces should be retained or pruned
  - keep the packet format-neutral: evaluation harness/reporting only
- [x] 3.21 CRUSHR-FORMAT-10 (metadata pruning experiment + four-variant recovery/size comparison harness)
- [ ] 3.22 CRUSHR-FORMAT-11 (metadata keep/prune decision lock from FORMAT-10 evidence)

## Security scrub track

- [x] CRUSHR-SCRUB-01 extraction confinement hardening
- [x] CRUSHR-PLAN-LEGACY-01 extraction authority boundary enforcement
- [x] CRUSHR-PLAN-LEGACY-01-f1 post-review test clarity hardening
- [x] CRUSHR-PLAN-LEGACY-01-f2 preferred extraction authority delegation

## Phase 4 — Product-completeness track (planned)

- [ ] 4.1 Unix metadata preservation envelope
  - file type
  - mode
  - uid/gid
  - optional uname/gname policy
  - mtime policy
  - symlink target
  - xattrs
- [ ] 4.2 Advanced Unix metadata decision
  - ACLs
  - capabilities
  - device metadata
  - SELinux labels
  - hardlink identity

## Phase 5 — Compression optimization track (planned)

- [ ] 5.1 Revisit dictionary strategy after resilience/placement/grid results settle
- [ ] 5.2 Compare archive-global vs clustered vs explicit dictionary-object approaches
- [ ] 5.3 Keep dictionary dependencies verifiable and recovery-honest
