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

[x] PHASE 15 — Dictionary Hardening and Namespace Factoring

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
- [x] CRUSHR-UI-01 — unified CLI presentation contract and standardized `--silent` behavior across pack/extract/verify/salvage.
- [x] CRUSHR-CHECK-02 — unified policy gate baseline (secrets, audit, MSRV, style, and VERSION drift enforcement) plus SECURITY.md and workflow-backed README badge alignment.
- [x] CRUSHR-STYLE-FIX-01 — repo-wide Clippy style-debt cleanup to reestablish zero warnings under enforced `-D warnings` policy gate.
- [x] CRUSHR_CLI_UNIFY_01 — canonical shared CLI app boundary with in-process `crushr` command dispatch for pack/extract/verify/info/about/lab, plus legacy top-level dispatch cleanup.
- [x] CRUSHR_CLI_UNIFY_02 — retained companion binaries unified as thin wrappers over shared CLI/app dispatch (`pack`/`extract`/`info`/`salvage`) with wrapper-owned presentation/parsing duplication removed.
- [x] CRUSHR_CLI_UNIFY_03 — CLI contract enforcement pass: added integration tests for canonical taxonomy/help/about/version/exit-code/shared-flag invariants and removed remaining undocumented alias branches.
- [x] CRUSHR_CLI_UNIFY_04 — production-vs-lab pack boundary lock: remove experimental format/layout controls from public `pack`, relocate retained experimentation to `crushr lab`, and align parser/help/tests/docs.
- [x] CRUSHR_PACK_SCALE_01 — bound production `pack` planning memory by eliminating whole-run file payload pre-materialization and moving payload compression/hashing to serialization-time streaming.
- [x] CRUSHR_PACK_STREAMING_01 — remove recurring production pack whole-run raw-byte retention in hard-link reuse/manifest emission so memory stays bounded to active working-set state.
- [x] CRUSHR_VERIFY_SCALE_01 — bound production `verify` memory by removing temp extraction/materialization from verify path and adding deterministic real-phase progress visibility in human verify output.
- [x] CRUSHR_RECOVERY_MODEL_01 — add `extract --recover` trust-class recovery model, segregated output contract, anonymous naming policy, and required recovery manifest schema/output generation.
- [x] CRUSHR_RECOVERY_MODEL_02 — integrate salvage planning into `extract --recover`, add required recover progress phases, and emit required recovery Result/Trust summary contract.
- [x] CRUSHR_RECOVERY_MODEL_03 — implement confidence-tiered anonymous content classification with modular signature/structure validation, manifest integration, and enforced naming tiers.
- [x] CRUSHR_RECOVERY_MODEL_04 — build deterministic corruption corpus and end-to-end strict/recover extraction validation with manifest/naming/classification truth assertions.
- [x] CRUSHR_RECOVERY_MODEL_05 — polish recover-mode user presentation so phased progress, final summary labels, and trust/notes clearly separate canonical extraction, recovered outputs, and unrecoverable loss without salvage/lab jargon.
- [x] CRUSHR_RECOVERY_MODEL_06 — harden recovery classification correctness boundaries (zip-family high-confidence gating), deterministic naming collision guarantees, and clean-archive zero-recovery-artifact assertions.
- [x] CRUSHR_RECOVERY_MODEL_07 — add explicit `metadata_degraded` trust class across strict/recover classification, output layout, manifest schema, and recover CLI summaries; strict mode now refuses metadata-degraded outcomes.
- [x] CRUSHR_RECOVERY_MODEL_08 — complete profile-aware metadata-degraded + strict fail-closed semantics for non-regular canonical entry kinds (directory/symlink/FIFO/char-device/block-device) and align recover manifest/report truth.
- [x] CRUSHR_UI_POLISH_01 — centralize CLI visual token semantics + calm status vocabulary across pack/extract/verify/info/salvage and document trust-class visual semantics contract.
- [x] CRUSHR_UI_POLISH_02 — add shared title/section/key-value/progress/banner/result presentation primitives and migrate verify/extract/recover/pack/info to one stable structural hierarchy.
- [x] CRUSHR_UI_POLISH_03 — define restrained motion contract and implement shared active-phase animation/state-settlement primitives for pack/extract/verify with TTY-safe no-motion fallbacks.
- [x] CRUSHR_UI_POLISH_04 — apply motion polish to core commands by adding live pack serialization detail updates, enforcing non-TTY motion cleanliness checks, and settling progress rows into stable final summaries.
- [x] CRUSHR_UI_POLISH_06 — finalize CLI visual consistency (about color/style/divider/newline/alignment) and expand `info` into a product-grade archive inspection summary with user-facing dictionary/compression reporting.
- [x] CRUSHR_UI_POLISH_07 — finalize CLI trust polish: shared-token help colorization, default `.crs` output extension behavior, truthful pack phase progression (`compression`/`serialization`/`finalizing`) with N/N completion, pack runtime/compression metrics, and `info` compression method/level inspection rows.
- [x] CRUSHR_UI_POLISH_08 — lock stable pack phase row identity (`compression` + `serialization` persistent rows, explicit `finalizing` transition) and correct `info` Structure terminology to file-level truth (`files`, `compressed units`, `file mappings`, `block model`).
- [x] CRUSHR_INTROSPECTION_01 — implement `crushr info --list` archive content introspection (tree + `--flat`) with corruption-aware, index-proven listing semantics.
- [x] CRUSHR_INTROSPECTION_01-FIX1 — expose omitted non-regular entry counts in list output, add degraded-path salvage guidance, and align version to `0.4.1`.
- [x] CRUSHR_INTROSPECTION_01-FIX2 — treat omission-only list behavior as informational/complete while keeping structural-proof failures degraded.
- [x] CRUSHR_INTROSPECTION_02 — expand/polish `info` + `info --list` readability with profile contract clarity, metadata omission-vs-presence wording, entry-kind summary visibility, and calm fail-closed listing context.
- [x] CRUSHR-BENCH-01 — implement deterministic benchmark contract foundation with reproducible datasets, explicit comparator commands/profiles, and schema-backed benchmark output.
- [x] CRUSHR-BENCH-02 — initial canonical corpus generation, baseline runs, and first quantitative benchmark artifacts.
- [x] CRUSHR-BENCH-03 — pack pipeline attribution instrumentation (`--profile-pack`), deterministic phase timing visibility, and benchmark-operator capture guidance for medium/large datasets.
- [x] CRUSHR_OPTIMIZATION_01 — profile-aware discovery capture gating + duplicate stat removal in production `pack` to reduce discovery-phase filesystem overhead without changing archive semantics.
- [x] CRUSHR_OPTIMIZATION_02 — optimize production `pack` compression/emission via buffered archive writes and reusable compression output buffers while preserving profile semantics, mutation detection, and truthful phase attribution.
- [x] CRUSHR_OPTIMIZATION_03 — optimize production `pack` compression hot path by reusing zstd compression context/state across payload+metadata units while preserving deterministic output semantics, profile truth boundaries, and fail-closed correctness checks.
- [x] CRUSHR_HOSTILE_REVIEW_01 — perform hostile enterprise structural review (duplication/layering/drift/vibe-residue) and publish prioritized cleanup roadmap before additional capability expansion (report refreshed with explicit question-by-question answers on 2026-03-27).
- [x] CRUSHR_CLEANUP_02 — unify pack preservation-profile authority in one canonical planning decision layer, remove discovery/emission policy ownership, and centralize warning emission from plan decisions.
- [x] CRUSHR_CLEANUP_03 — deduplicate recover metadata-degraded routing with one canonical authority and one shared metadata-degraded manifest/entry assembly path.
- [x] CRUSHR_CLEANUP_04 — unify strict/recover metadata restoration mechanics into one shared restoration core with explicit policy inputs for strict vs recover handling.
- [x] CRUSHR_CLEANUP_05 — decompose pack command/module internals into explicit ownership layers for discovery, planning, emission, and orchestration boundaries without behavior drift.
- [x] CRUSHR_CLEANUP_06 — info/introspection operator-truth authority centralization (profile/fallback/metadata/archive-state reporting).
- [x] CRUSHR_CLEANUP_07 — recover extract orchestration dead pre-analysis removal / authority clarification (no computed-and-discarded pre-pass).
- [x] CRUSHR_CLEANUP_08 — restore selective discovery metadata capture through canonical profile-derived requirements without reintroducing discovery-owned omission policy.
- [x] CRUSHR_CLEANUP_09 — physically decompose pack command into bounded files/modules (`pack.rs` orchestration + `pack/{discovery,planning,emission}.rs`) while preserving canonical profile/planning authority and behavior.
- [x] CRUSHR_CLEANUP_10 — unify strict/recover shared extraction payload/materialization mechanics (entry-byte reads, block raw payload access, regular/sparse write helpers) behind one internal mechanism while preserving explicit policy boundaries.

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

- [x] CRUSHR_PHASE16_01 — benchmark/profiling harness normalization: deterministic dataset identity controls, canonical harness entrypoint, centralized assumptions, and schema-backed metadata embedding.
- [x] CRUSHR_PHASE16_02 — controlled dictionary experiment path: deterministic dictionary training/cohort model, explicit dictionary identity/provenance metadata, schema-distinct dictionary-assisted runs, and benchmark-only boundary documentation.
- [x] CRUSHR_PHASE16_03 — controlled zstd-native experiment path: deterministic zstd level/strategy matrices, explicit comparator/run metadata for zstd parameters, canonical harness integration, and benchmark-only boundary documentation.
- [x] CRUSHR_PHASE16_04 — harden zstd benchmark strategy execution against host CLI differences: centralized zstd command construction, default-strategy no-flag behavior, and early capability diagnostics for unsupported non-default strategies.
- [x] CRUSHR_PHASE16_05 — deterministic ordering/locality benchmark experiments: centralized ordering strategy model, canonical harness integration, explicit ordering metadata in schema/results, and benchmark-only boundary documentation.
- [x] CRUSHR_PHASE16_06 — ordering input-list correctness hardening: deterministic tar-resolvable ordering file paths, preflight file-list validation diagnostics, and canonical tar `-T` compatibility for ordering experiments.
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

Milestone packets

- [x] CRUSHR_PRESERVATION_01 — baseline Linux-first filesystem metadata preservation (`regular`/`directory`/`symlink`, mode, mtime, empty directories, xattrs) in pack/extract with deterministic round-trip coverage.
- [x] CRUSHR_PRESERVATION_02 — preserve ownership (`uid`/`gid`) + hard-link semantics, restore best-effort ownership with explicit warnings, and expose metadata-class presence in `info`.
- [x] CRUSHR_PRESERVATION_03 — extend Linux tar-class preservation with sparse regular files, FIFO/device-node entry kinds, best-effort special-file restore warnings, and ownership-name enrichment (`uname`/`gname`) while keeping numeric uid/gid authoritative.
- [x] CRUSHR_PRESERVATION_04 — add explicit preservation/restore envelope for POSIX ACL metadata, SELinux label metadata, and Linux capability metadata with truthful warning-based degradation plus `info` metadata-presence visibility.
- [x] CRUSHR_PRESERVATION_05 — add explicit `--preservation <full|basic|payload-only>` profile contract, record profile in archive metadata, make strict/recover canonical semantics profile-aware, and show profile in `info`.
- [x] CRUSHR_PRESERVATION_FIX_06 — enforce extraction-time profile authority so omitted metadata classes are not restoration-attempted/warned in strict or recover paths, while full-profile behavior stays unchanged.

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

- Phase 15 hardening/cleanup closeout is complete (through CRUSHR_CLEANUP_10 + continuity reconciliation packet CRUSHR_CLEANUP_11).
- Next planner-issued packet should explicitly select the active Phase 16+ workstream rather than relying on historical “next” notes.
- Keep evidence/custody features on the long-range roadmap rather than inside the current 0.x core unless explicitly promoted by decision.
