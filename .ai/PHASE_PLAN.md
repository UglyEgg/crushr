<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Phase Plan

This document defines the high-level development phases for crushr after the
format experiments established the core archive architecture.

The project is now explicitly sequenced across three maturity bands:

- **0.x** — product proof, stabilization, benchmarking, and compression/performance evidence
- **1.x** — stable preservation platform with locked contracts and workflow maturity
- **2.x** — evidence-grade extension layer (custody/signing/classification expansion)

The immediate focus is 0.x.

---------------------------------------------------------------------

Current Architectural Identity

crushr is a salvage-oriented preservation/archive format built around:

- distributed extent identity
- mirrored naming dictionaries
- deterministic recovery / verification reporting
- fail-closed naming semantics
- anonymous fallback when naming proof is unavailable
- structured, explicit preservation of imperfect result sets

The format prioritizes recoverability and deterministic verification over
maximum compression ratio.

---------------------------------------------------------------------

[~] PHASE 15 — Dictionary Hardening and Namespace Factoring

Goal

Finalize the mirrored dictionary direction, close hardening debt, and complete
the transition from experimental format work into product-ready baseline
architecture.

Key work

- generation-aware dictionary copy headers
- mirrored dictionary validation rules
- factored namespace dictionary
- canonical verification/report truth cleanup
- operator-facing CLI unification
- benchmark harness kickoff readiness

Deliverables

- [x] CRUSHR-HARDEN-03C — schema-backed contracts for active FORMAT comparison artifacts.
- [x] CRUSHR-HARDEN-03E — decomposition of `crushr-lab-salvage` comparison engine.
- [x] CRUSHR-HARDEN-03F — staged `crushr-pack` layout/emission pipeline.
- [x] CRUSHR-HARDEN-03A — API boundary truth / visibility cleanup.
- [x] CRUSHR-HARDEN-03H — verification model purity enforcement.
- [x] CRUSHR-HARDEN-03I — typed metadata completion in active pack/salvage verification paths.
- [x] CRUSHR-LAB-FIX-01 — repair comparison/normalization contract tests and deterministic scenario ordering.
- [x] CRUSHR-LICENSE-01 — unified licensing + REUSE compliance.
- [x] CRUSHR-CRATE-01 — lock workspace MSRV/edition/resolver policy and enforce explicit publish intent + crates.io metadata drift checks.
- [~] CRUSHR-UI-01 — unified CLI presentation contract and standardized `--silent` behavior across pack/extract/verify/salvage (pending final runtime validation / acceptance).
- [x] CRUSHR-CHECK-02 — unified policy gate baseline (secrets, audit, MSRV, style, and VERSION drift enforcement) plus SECURITY.md and workflow-backed README badge alignment.
- [ ] CRUSHR-BENCH-01 — implement benchmark harness foundation using manifest-driven deterministic corpus/variant/corruption execution and schema-backed raw/summary output.
- [ ] CRUSHR-BENCH-02 — initial canonical corpus generation, baseline runs, and first quantitative benchmark artifacts.
- [ ] CRUSHR-BENCH-03 — benchmark result review, compression/performance tuning priorities, and whitepaper/data-surface integration.

Exit criteria

- mirrored dictionary / namespace direction is canonical and documented
- verification/report truth is deterministic and typed
- unified CLI/operator identity is accepted
- benchmark harness foundation exists and produces schema-valid deterministic output

---------------------------------------------------------------------

[ ] PHASE 16 — 0.x Benchmarking and Compression Evidence

Goal

Establish quantitative evidence for crushr’s 0.x product thesis and use the
results to guide compression and performance refinement.

Key work

- deterministic corpus families and corruption families
- manifest-driven benchmark execution
- archive size, overhead, and verified recovery measurement
- classification stability measurement
- verification/report reproducibility measurement
- compression/performance analysis from real benchmark data
- update docs/whitepaper with measured results rather than narrative claims

Deliverables

- benchmark harness implementation
- raw benchmark records
- normalized benchmark outputs
- schema-valid summary artifacts
- first whitepaper-ready charts/tables
- compression/performance tuning plan driven by measured data

Exit criteria

- benchmark harness is reproducible and schema-backed
- crushr has quantitative evidence for recovery, overhead, and determinism
- compression work is guided by data rather than intuition

---------------------------------------------------------------------

[ ] PHASE 17 — 0.x Product Envelope Completion

Goal

Complete the bounded metadata and workflow surface needed for a serious 0.x
preservation product.

Key work

- file mode support
- uid/gid support
- mtime preservation
- symbolic link handling
- extended attribute support (xattrs)
- archive identity and version policy
- stable machine-readable verify/report contracts
- post-recovery preservation workflow documentation

Constraints

- metadata support must not weaken salvage guarantees
- metadata must remain verifiable and deterministic

Deliverables

- metadata envelope implementation
- stable verify/report schemas
- workflow docs/examples
- compatibility rules

Exit criteria

- crushr can represent the common POSIX filesystem model
- verification/report outputs are stable enough for workflow automation
- 0.x product story is coherent and documented

---------------------------------------------------------------------

[ ] PHASE 18 — 1.x Preservation Platform

Goal

Graduate crushr from promising 0.x product into a stable preservation platform
that others can build workflows around.

Key work

- lock on-disk format commitments appropriate for 1.x
- stabilize public CLI/report contracts
- release/process hardening
- reproducible build verification
- stronger packaging and workflow integration
- public-facing docs/tutorial maturity

Deliverables

- 1.x contract review
- stable format/reference docs
- reproducible build and release process
- workflow-ready product surface

Exit criteria

- format and verification contracts are stable
- operators can build repeatable workflows around crushr
- release quality is defensible

---------------------------------------------------------------------

[ ] PHASE 19 — 2.x Evidence and Custody Layer

Goal

Add evidence-grade extension capabilities on top of the stabilized preservation
platform without prematurely coupling legal/custody semantics into the core
format.

Key work

- evidence manifest sidecar design and implementation
- signature scope over archive + canonical verify report
- append-only custody event log
- signer identity / tool-version provenance
- extension-point review for future embedded evidence semantics, if ever justified

Constraints

- sidecar-first design preferred
- no premature legal-ceremony claims without validated process/tool support
- core format should remain stable and minimally burdened

Deliverables

- evidence manifest schema
- custody event schema / append-only model
- signing / verification workflow
- docs describing evidentiary scope and limitations

Exit criteria

- crushr can preserve and later attest to packaging / verification events
- custody/signing layer is coherent and externally explainable
- evidence features remain separable from core archive semantics

---------------------------------------------------------------------

[ ] PHASE 20 — 2.x Deterministic Internal Classification Expansion

Goal

Expand crushr from preserving declared truth toward deriving more categories of
truth internally where the tool can prove them safely.

Key work

- deterministic classification expansion where provable
- stronger internal structural / naming derivation rules
- refusal and fallback logic review
- richer evidence/preservation reporting categories

Constraints

- no semantic overreach
- every derived claim must remain provable and deterministic
- do not invent “confidence theater”

Deliverables

- expanded internal classification rules
- updated schemas/reports
- validation corpus and review artifacts

Exit criteria

- crushr can derive more preservation truth itself without weakening trust
- every new category has a defensible proof model

---------------------------------------------------------------------

Latest priority doctrine

- Finish acceptance of the unified CLI/operator surface.
- Start benchmark harness implementation immediately after.
- Use benchmark data to steer compression and performance work.
- Keep evidence/custody features on the long-range roadmap rather than inside the current 0.x core.
