<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Phase Plan

This document defines the high-level development phases for crushr after the
format experiments (FORMAT-11 through FORMAT-14A) established the core archive
architecture.

The project is transitioning from experimental exploration to productization.
Future work prioritizes architectural stability, documentation clarity, and
bounded feature additions.

---------------------------------------------------------------------

Current Architectural Identity

crushr is a salvage-oriented archive format built around:

- distributed extent identity
- mirrored naming dictionaries
- deterministic recovery classification
- fail-closed naming semantics
- anonymous fallback when naming proof is unavailable

The format prioritizes recoverability and deterministic verification over
maximum compression ratio.

---------------------------------------------------------------------

[ - ] PHASE 15 — Dictionary Hardening and Namespace Factoring

Goal

Promote the mirrored dictionary architecture to the canonical format while
improving robustness and dictionary efficiency.

Key work

- generation-aware dictionary copy headers
- mirrored dictionary validation rules
- factored namespace dictionary
  - directory prefix dictionary
  - basename dictionary
  - file binding table
- new comparison harness runs
- baseline + stress corpus validation

Deliverables

- [x] CRUSHR-HARDEN-03C — add schema-backed contracts for active FORMAT-12/13/14A/15 comparison artifacts and test coverage; defer broader decomposition to 03E.
- [x] CRUSHR-HARDEN-03E — decompose `crushr-lab-salvage` comparison engine into bounded modules (`common`, `experimental`, `format06_to12`, `format13_to15`) while keeping comparison command semantics stable and schema-backed checks passing.
- [x] CRUSHR-HARDEN-03F — decompose `crushr-pack` into explicit layout/emission stages, isolate dictionary planning, and tighten canonical vs experimental writer boundary without changing archive semantics.
- [x] CRUSHR-HARDEN-03G — extract experimental metadata-record JSON construction into helper builders to reduce emission-loop coupling while preserving deterministic writer semantics.
- [x] CRUSHR-HARDEN-03A — finalize API boundary truth by removing accidental public module exposure, tightening visibility, and aligning runtime/library/lab boundary docs with supported surfaces.
- [~] CRUSHR-HARDEN-03G — canonical verification model + typed metadata completion + salvage classification lint cleanup (in progress; canonical verify model + lint fix landed, remaining typed metadata conversion follow-up tracked in STATUS).
- [x] CRUSHR-HARDEN-03H — enforce verification model purity by removing verify output bypass/duplicate truth path in `crushr-extract` and projecting report output through canonical `VerificationReportView`.
- [x] CRUSHR-HARDEN-03I — complete typed pack/salvage metadata conversion and bind remaining salvage classification flows to the canonical verification/report truth model (active salvage metadata flow in `metadata.rs` now typed; dynamic `Vec<Value>` classification path removed).
- [x] 3.25.1 CRUSHR-FORMAT-14A-FIX1 (repair dictionary-corruption classification/outcome reporting; rerun required FORMAT-14A artifacts)
- [x] 3.25.2 CRUSHR-FORMAT-14A-FIX2 (restore header+tail dual-copy one-loss named recovery; rerun FORMAT-14A artifacts)
- [x] CRUSHR-FORMAT-15 — Harden mirrored dictionary identity/generation semantics and add factored namespace dictionary with FORMAT-15 baseline/stress comparison commands + artifacts.
- [x] CRUSHR-FORMAT-15-FIX1 — repair FORMAT-15 regression causing false-negative canonical-candidate judgments (restore scenario-authoritative fail-closed gating + v2 full-path dictionary parsing).
- [x] CRUSHR-LAB-FIX-01 — repair failing Phase 2 comparison/normalization shape-contract tests and restore explicit deterministic normalized scenario ordering checks.
- [x] CRUSHR-LICENSE-01 — repository-wide license unification (MIT OR Apache-2.0 code, CC-BY-4.0 docs), SPDX header sweep, Cargo metadata alignment, and REUSE compliance verification.
- [x] CRUSHR-LICENSE-01-FIX1 — migrate REUSE metadata from `.reuse/dep5` to `REUSE.toml` to remove tooling deprecation warning while keeping licensing classification unchanged.
- [x] CRUSHR-UI-01 — establish a unified CLI presentation contract and standardized `--silent` mode across `crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` before benchmark-harness work.
- [x] CRUSHR-UI-01-FIX1 — restore workspace Cargo manifest validity (`package.name` fields), rerun blocked formatting/test commands, and complete representative runtime validation for pack/extract/verify/salvage and `--silent`.
- [x] CRUSHR-UI-02 — realign public CLI surface to canonical preservation suite (`pack/extract/verify/info`) with bounded salvage/lab demotion and repair strict verify structural-failure presentation so parser internals do not leak in normal user output.
- [x] CRUSHR-UI-03 — ship minimalist section-based CLI presentation templates across pack/verify/info/salvage, default `crushr-info` to human-readable mode, and lock deterministic golden outputs for verify success/failure + pack/info/salvage.
- FORMAT-15 comparison results
- FORMAT-15 stress comparison results
- updated SNAPSHOT_FORMAT.md
- updated format-evolution.md

Exit criteria

- factored mirrored dictionary is confirmed smaller or equal to the existing
  mirrored dictionary approach
- recovery semantics remain deterministic and fail-closed

---------------------------------------------------------------------

[x] PHASE 16 — Architecture Hardening and De-cruft

Goal

Convert the repository from an experimental lab environment into a clean,
coherent product codebase.

Key work

- hostile architectural review
- remove abandoned experimental variants
- isolate research harness into `lab/`
- consolidate salvage pipeline logic
- simplify configuration and variant flags
- align CLI surface with product behavior
- documentation rewrite to reflect canonical architecture

Deliverables

- hostile review report
- repository refactor patch
- updated architecture documentation
- clean module boundaries between runtime and lab code

Exit criteria

- canonical runtime code contains no experimental variant logic
- research harness exists but is isolated from production paths
- CLI surface matches documented product behavior

---------------------------------------------------------------------

[ ] PHASE 17 — Archive Envelope Completion

Goal

Complete the archive metadata envelope required for a production archive tool.

Key work

- file mode support
- uid/gid support
- mtime preservation
- symbolic link handling
- extended attribute support (xattrs)
- archive identity and version policy
- compatibility rules for future format revisions

Constraints

- metadata support must not weaken salvage guarantees
- metadata must remain verifiable and deterministic

Deliverables

- metadata envelope implementation
- documentation updates
- compatibility rules

Exit criteria

- crushr can represent the common POSIX filesystem model
- metadata integrity survives salvage operations

---------------------------------------------------------------------

[ ] PHASE 18 — Compression Intelligence

Goal

Improve compression efficiency while preserving recovery semantics.

Key work

- compression dictionary experimentation
- corpus benchmarking
- dictionary persistence strategies
- interaction between compression dictionaries and extent identity
- bounded compression improvements without destabilizing recovery

Constraints

- compression must not compromise deterministic recovery
- compression metadata must remain verifiable

Deliverables

- compression comparison results
- updated compression pipeline
- documentation updates

Exit criteria

- measurable compression improvement over baseline
- salvage behavior unchanged

---------------------------------------------------------------------

[ ] PHASE 19 — Verification and Tooling Excellence

Goal

Make verification and recovery tooling first-class capabilities.

Key work

- strict verification UX/reporting improvements (`crushr-extract --verify`)
- recovery diagnostics
- structured output formats
- clearer salvage reporting
- improved error explanations
- enhanced archive introspection

Deliverables

- improved CLI tooling
- richer strict verification reports
- improved info command output

Exit criteria

- operators can easily diagnose archive state
- salvage outcomes are transparent and well explained

---------------------------------------------------------------------

[ ] PHASE 20 — CLI and Documentation Polish

Goal

Prepare crushr for public release by improving usability and documentation.

Key work

- CLI consistency review
- man-page style help text
- usage examples
- README rewrite
- tutorials and workflow documentation
- archive format reference documentation

Deliverables

- polished CLI
- improved documentation
- examples and tutorials

Exit criteria

- new users can understand crushr from documentation alone
- CLI commands are predictable and consistent

---------------------------------------------------------------------

[ ] PHASE 21 — Release Candidate

Goal

Produce the first release candidate of the crushr archive format and toolset.

Key work

- final hostile review
- compatibility audit
- reproducible build verification
- packaging preparation
- release documentation

Deliverables

- release candidate tag
- reproducible builds
- checksums and verification files
- compatibility notes
- roadmap for post-v1 improvements

Exit criteria

- archive format is stable
- documentation is complete
- tooling is production-ready


## Latest completion
- **CRUSHR-HARDEN-03D is complete**: audited reader/open/parse boundary, aligned strict verification semantics under `crushr-extract --verify`, tightened permissive legacy reader checks, and refreshed active docs/help naming for the locked tool boundary.


## 2026-03-18 — CRUSHR-HARDEN-03B

- [x] Reconciled `crushr-salvage` emitted `mapping_provenance` and `recovery_classification` vocabularies with `schemas/crushr-salvage-plan.v3.schema.json`.
- [x] Added typed enum boundaries in salvage output path for classification/provenance and typed reason-code emission for contract-level reason arrays.
- [x] Added schema-conformance coverage for enum vocabulary parity and reason-code closure.

## 2026-03-18 — CRUSHR-HARDEN-03D

- [x] Audited reader/open/parse behavior and documented strict-vs-permissive boundary findings in control updates.
- [x] Hardened `crushr-extract --verify` to run strict extraction semantics and emit deterministic refusal reasons from canonical refusal paths.
- [x] Tightened permissive legacy reader checks that could blur strict boundary expectations.
- [x] Updated active boundary docs and stale help strings to reduce public `fsck` naming drift.
