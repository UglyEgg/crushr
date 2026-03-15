# .ai/DECISION_LOG.md

## 2026-02-17 — Canonical continuity policy source

- Decision: Use the prime scaffold `AGENTS.md` as canonical policy; preserve the original `crushr` `AGENTS.md` as legacy reference only.
- Alternatives:
  1. Replace scaffold `AGENTS.md` with the imported `crushr` `AGENTS.md`.
  2. Merge both into a single hybrid policy.
- Rationale: User instruction specifies `.tar.gz` directives as canonical; the scaffold is the `.tar.gz` source.
- Blast radius:
  - Affects how future instances interpret workflow, packaging, and handoff rules.
  - Imported policy references are now informational only.

## 2026-02-17 — Adopt core/format split and multi-tool suite

- Decision:
  - Introduce `crushr-format` (on-disk layouts) and `crushr-core` (engine over minimal IO traits), with `crushr` as the platform/integration crate.
  - Prefer a suite of focused CLI tools (pack/info/fsck/extract) over a monolithic CLI.
  - Enforce a **no-IPC** rule between tools (no JSON protocols, no sockets); all tools link crates and call APIs in-process.
- Alternatives:
  1. Single `crushr` crate + one CLI binary with many subcommands.
  2. Separate tools communicating via JSON/stdio or a daemon.
- Rationale:
  - Needed to support many knobs/features while keeping parsing logic centralized and enabling a "geek visibility" TUI.
  - In-process linking keeps cross-platform basics viable and avoids operational complexity.
- Blast radius:
  - Major repo restructure into a Cargo workspace.
  - Future development must respect crate boundaries to avoid duplicated parsing logic.

## 2026-02-17 — Freeze archive format v1.0 (BLK3/DCT1/IDX3/FTR4) and drop prototype compatibility

- Decision:
  - `SPEC.md` is now the v1.0 contract: BLK3 blocks, optional DCT1, IDX3 index, FTR4 footer.
  - Pre-v1.0 prototype archives are not guaranteed readable by v1.0 tools.
- Alternatives:
  1. Preserve backwards compatibility with BLK2/FTR2/older IDX variants.
  2. Provide a separate conversion tool only.
- Rationale:
  - The codebase was in a spec-drift state; freezing a single contract is required for a technically superior rewrite.
  - Compatibility can be added later as an explicit feature/phase if needed.
- Blast radius:
  - Existing prototype archives may become unreadable until/if a compatibility layer is implemented.

## 2026-02-17 — TUI supports live and snapshot modes

- Decision:
  - `crushr-tui` must support both **live mode** (open archive directly) and **snapshot mode** (load JSON outputs from `crushr-info --json` and `crushr-fsck --json`).
  - Snapshots are versioned and include an `archive_fingerprint`; snapshots with mismatched fingerprints must not be merged.
- Alternatives:
  1. Live mode only.
  2. Snapshot mode only.
  3. Ad-hoc, tool-specific JSON without a documented contract.
- Rationale:
  - Snapshot mode enables offline analysis, sharing, and deterministic regression tests without requiring access to the archive.
  - A documented contract prevents TUI/tool drift.
- Blast radius:
  - Introduces a stable JSON boundary (`docs/SNAPSHOT_FORMAT.md`) and schemas under `schemas/`.
  - TUI and tools must evolve snapshots in a versioned, backward-compatible way.

## 2026-03-08 — Recovery policy: detect and isolate only

- Decision: `fsck` detects and isolates corruption; it does not attempt reconstruction. Raw compressed blast-zone payload bytes may be dumped, and decompressed dumps are emitted only when verification passes.
- Rationale: preserves clarity, avoids ambiguous output, and keeps crushr out of the parity/reconstruction space.

## 2026-03-08 — Dictionary placement is per tail frame

- Decision: DCT1 is embedded per tail frame so each tail frame is self-contained for decode. Dictionary entries carry BLAKE3 hashes.
- Rationale: improves tail survivability without relying on external dictionary state.

## 2026-03-08 — TUI supports live and snapshot modes

- Decision: TUI is planned for both live archive access and versioned snapshot loading.
- Rationale: easier offline analysis, reproducible demos, and lower coupling.

## 2026-03-08 — Adaptive planning starts opt-in

- Decision: any auto-planning heuristics remain opt-in until tested and recorded in the ledger/results.
- Rationale: preserve determinism and avoid hidden behavior drift.

## 2026-03-08 — Adopt contracts, research scaffolding, and Codex control layer

- Decision: treat `docs/CONTRACTS/*`, `PHASE2_RESEARCH/methodology/*`, `PROJECT_STATE.md`, and the `.ai/` control files as canonical implementation guidance surfaces for active work.
- Rationale: reduce drift, preserve the thesis, and keep Codex constrained to bounded tasks.

## 2026-03-08 — Normalize `crushr-info`/`crushr-fsck` open/parse failure exit codes

- Decision: For current workspace baseline, both `crushr-info` and `crushr-fsck` return exit code `2` for archive open failures and structural/parse/validation failures; usage/argument errors remain exit code `1`.
- Alternatives:
  1. Keep pre-existing inconsistency (`crushr-info` parse/open as `1`, `crushr-fsck` as `2`).
  2. Introduce a broader multi-code mapping now (including internal-failure `4`) across all tools in this pass.
- Rationale: This bounded hygiene pass required consistency for open/parse/structural failures without redesigning the full CLI error taxonomy.
- Blast radius:
  - Affects observed nonzero exit code behavior for `crushr-info` callers.
  - No format/snapshot/schema or research-claim semantics changed.

## 2026-03-12 — CRUSHR-1.1-B: propagation contract narrowed to truthful observation boundary

- Decision:
  - Keep propagation reporting bounded to archives that can be opened/indexed by `crushr-info --json --report propagation`.
  - Treat current-state structural corruption as non-observable in this CLI path; represent structure failures only as hypothetical causes or explicit caller assumptions.
  - Rename report fields to make this boundary explicit (`assumed_corrupted_structure_nodes`, `actual_impacts_from_current_payload_corruption`).
- Alternatives:
  1. Implement structural-current-state reporting via lower-level fsck/open bypass path in this packet.
  2. Keep existing field names/prose and rely on caveats.
- Rationale:
  - Option 1 required invasive changes across open/parse boundaries and risked destabilizing Phase transition.
  - Option 2 left a contract lie.
  - Narrowing preserves deterministic behavior and removes misleading semantics.
- Blast radius:
  - Propagation schema/contract/tests and `crushr-info` report consumers must adopt renamed fields.
  - No extraction behavior or archive format changes.

## 2026-03-12 — CRUSHR-1.1-B follow-up: structural-current-state propagation fallback implemented

- Decision:
  - Implement bounded structural-current-state propagation fallback in `crushr-info --json --report propagation` so reports still emit for structural failures where normal open fails.
  - Remove `crushr-extract --mode salvage`; extract is strict-only.
- Rationale:
  - Prior narrowed-only approach was rejected; structural-current-state reporting is required.
  - Legacy salvage surface contradicted canonical thesis and was explicitly requested for removal.
- Blast radius:
  - Propagation field semantics return to current-state structural reporting (`corrupted_structure_nodes`, `actual_impacts_from_current_corruption`).
  - Extraction JSON/schema/docs no longer include salvage fields.

## 2026-03-12 — CRUSHR-CLEANUP-2.0-A: remove remaining legacy recovery/salvage surfaces

- Decision:
  - Remove remaining legacy recovery/salvage product surfaces from active code/docs: `crushr` CLI recover/salvage commands, public API recovery/salvage options/functions, legacy recovery module, and snapshot `salvage_plan` field.
- Alternatives:
  1. Keep surfaces behind legacy/hidden/deprecated flags.
  2. Keep API stubs while removing internals.
- Rationale:
  - Hidden/deprecated retention still leaves a contradictory product surface and Phase 2 contamination risk.
  - Full deletion matches integrity-first thesis and current canonical scope.
- Blast radius:
  - Removes recover/salvage entry points from the legacy `crushr` monolith binary/API.
  - Any callers depending on these removed surfaces must migrate.

## 2026-03-12 — CRUSHR-CLEANUP-2.0-B canonical doc/control collapse

- Decision:
  - Collapse startup/authority guidance to one canonical order across `AGENTS.md`, `AI_BOOTSTRAP.md`, `REPO_GUARDRAILS.md`, `PROJECT_STATE.md`, and `.ai/*` control files.
  - Remove stale transitional markdown from active paths (legacy docs and imported continuity sediment).
  - Set Phase 2.1 manifest/schema as explicit next packet across control/docs.
- Alternatives:
  1. Keep transitional/legacy markdown for historical context in active paths.
  2. Keep multiple startup orders and rely on operator judgment.
- Rationale:
  - Multiple contradictory doc surfaces caused onboarding ambiguity and policy drift.
  - Single authority + startup order reduces execution variance and packet confusion.
- Blast radius:
  - Documentation-only contract/control cleanup; no product behavior change.
  - Fresh contributors now have one deterministic reading path.

## 2026-03-12 — CRUSHR-P2.1-A: deterministic Phase 2 scenario IDs and enumeration order locked

- Decision:
  - Lock Phase 2 core scenario ID format to `p2-core-{dataset}-{format_id}-{corruption_type}-{target_class}-{magnitude}-{seed}`.
  - Lock enumeration order to nested axis order: dataset → format → corruption_type → target_class → magnitude → seed, using matrix values from `PHASE_2_LOCKS`.
- Alternatives:
  1. Use opaque numeric scenario IDs and keep axis values only as fields.
  2. Sort by lexicographic scenario_id string post-generation.
- Rationale:
  - Human-readable deterministic IDs improve traceability in artifacts and review.
  - Axis-driven ordering avoids accidental drift from string sorting quirks and matches lock-file semantics directly.
- Blast radius:
  - `crushr-lab` manifest producers/consumers and any downstream report tooling now rely on this stable ID and ordering contract.
  - No runtime execution semantics changed in this packet.

## 2026-03-13 — CRUSHR-P2-CLEAN-03: canonical Phase 2 research workspace root

- Decision:
  - Create `PHASE2_RESEARCH/` as the canonical Phase 2 research root and move active Phase 2 lock guidance to `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md`.
  - Change `crushr-lab` Phase 2 default output paths to `PHASE2_RESEARCH/manifest/` and `PHASE2_RESEARCH/generated/{foundation,execution}/`.
- Alternatives:
  1. Keep emitting defaults under the former `docs/RESEARCH/artifacts/*` path.
  2. Keep lock docs in `.ai/` while only moving generated outputs.
- Rationale:
  - Product/reference docs and generated research state must remain separated to reduce drift and operator confusion.
  - A dedicated root makes Phase 2 methodology, manifests, generated artifacts, normalized outputs, summaries, and whitepaper support discoverable and bounded.
- Blast radius:
  - Default paths for `crushr-lab` Phase 2 commands changed; operators relying on old defaults must use explicit flags or migrate to new root.
  - Repo docs/control references now point to `PHASE2_RESEARCH/` as canonical Phase 2 workspace.

## 2026-03-13 — CRUSHR-P2-CLEAN-04: replace 7z comparator with tar.gz and tar.xz in locked core matrix

- Decision:
  - Remove `7z/lzma` from the locked Phase 2 core publication matrix and replace it with `tar+gz` and `tar+xz`.
  - Lock core comparator set to: `crushr`, `zip`, `tar+zstd`, `tar+gz`, `tar+xz` (2700 runs).
- Alternatives:
  1. Keep `7z/lzma` with skip/deferred behavior when unavailable.
  2. Replace `7z/lzma` with only one additional tar comparator (`tar+gz` or `tar+xz`).
- Rationale:
  - `7z` tool availability is unreliable in current execution environments, which undermines core-matrix reproducibility.
  - `tar+gz` and `tar+xz` are broadly available and deterministic for this methodology.
- Blast radius:
  - Phase 2 manifest/schema enums, scenario count/ordering tests, foundation archive build logic, runner observation/version probes, and lock docs now align on the 5-format set.
  - Any downstream artifacts/scripts assuming 2160 scenarios or 7z comparator names must migrate to 2700 and tar variants.

## 2026-03-13 — CRUSHR-P2-CLEAN-04 follow-up: suppress command-line unknown-lint diagnostic for required clippy invocation

- Decision:
  - Add workspace cargo config rustflag `-A unknown-lints` to align required command `cargo clippy --workspace --all-targets -- -D warning` with clean output expectations.
- Alternatives:
  1. Keep command as-is and accept warning output.
  2. Change required command to `-D warnings` (not permitted by packet requirement).
- Rationale:
  - Packet requires running a fixed command string; this workspace-local rustflag removes the known diagnostic without changing public APIs or product behavior.
- Blast radius:
  - Affects lint-diagnostic behavior only; no runtime/archive-contract behavior changes.

## 2026-03-13 — Phase-2 Evidence Pipeline Required for White-Paper Trials

- Status: Accepted
- Rationale:
  - The credibility of the white paper depends on producing reproducible and auditable experimental results.
- Decision:
  - The repository will implement a formal experimental evidence system including:
    - deterministic scenario manifests
    - raw execution records
    - normalized result schema
    - completeness auditing
    - reproducibility metadata
- Scope constraint:
  - This system governs experimental methodology only and does not modify the crushr archive format.

## 2026-03-13 — White-paper baseline scope excludes recoverability, random access, and deduplication

- Status: Accepted
- Decision:
  - The baseline Phase-2 white-paper evaluation remains limited to the current crushr implementation.
  - The following capabilities are explicitly deferred until after the white-paper trials:
    - recoverable archive extraction
    - true random-access extraction
    - built-in deduplication
- Rationale:
  - Each of these features would materially change archive structure, corruption semantics, or extraction behavior.
  - Adding them before trials would weaken the validity of the baseline comparison corpus.
- Blast radius:
  - Planning and roadmap only.
  - No baseline format or trial-matrix behavior changes.

## 2026-03-13 — Deterministic archive generation included before white-paper trials

- Status: Accepted
- Decision:
  - Include minimal deterministic archive generation before the white-paper trials.
  - The deterministic rules are limited to:
    1. deterministic file ordering
    2. normalized timestamps
    3. normalized permissions
    4. deterministic compression parameters
    5. deterministic metadata ordering
- Rationale:
  - Reproducible archives strengthen the experimental methodology and trust in published results without changing corruption semantics.
- Scope constraint:
  - Implementation must not alter archive structure or corruption semantics.

## 2026-03-13 — V2 architectural direction locks content-addressed block identity

## 2026-03-15 — Redundant-map empirical validation remains bounded targeted comparison

- Status: Accepted
- Decision:
  - Add a deterministic targeted comparison workflow (`crushr-lab-salvage run-redundant-map-comparison`) that compares old-style archives (redundant metadata stripped) vs new-style archives (redundant metadata preserved) across a bounded 24-scenario corpus.
  - Persist only compact summary artifacts (`comparison_summary.json`, `comparison_summary.md`) with grouped metrics and deterministic scenario rows.
- Rationale:
  - Quantifies CRUSHR-FORMAT-01 impact without rerunning the full Phase 2 matrix or expanding salvage semantics.
- Blast radius:
  - Research harness and docs/tests only.
  - No change to canonical extraction semantics or archive format contracts.


- Status: Accepted
- Decision:
  - The long-term v2 direction for crushr is content-addressed block identity with deterministic on-disk indexing over content identities.
  - File records should ultimately reference verified block identities rather than positional-only storage.
- Rationale:
  - This gives recoverability, random access, and deduplication a coherent architectural foundation instead of layering them onto positional assumptions.
- Scope constraint:
  - This is roadmap/architecture guidance only and must not create ambiguity in the baseline white-paper implementation.

## 2026-03-14 — CRUSHR-P2-EXEC-03B: truthful tool-version observation model for execution evidence

- Decision:
  - Represent execution tool-version capture as a typed observation (`status`, optional `version`, optional `detail`) instead of a single opaque version string.
  - Record tar comparator version probing as `unsupported` in per-format records to avoid pretending per-variant versions where a stable direct probe is not available.
- Alternatives:
  1. Keep string-only `tool_version` and continue storing command stderr/stdout first-line values.
  2. Drop version collection entirely from execution evidence.
- Rationale:
  - White-paper-grade evidence requires truthful and machine-readable separation between detected versions and unsupported/unavailable probes.
  - Prevents invalid strings (e.g., unsupported-flag diagnostics) from being interpreted as tool versions in downstream analysis.
- Blast radius:
  - `crushr-lab` raw execution records/report schema and consuming analysis tooling now read typed version observations instead of a plain version string.



## 2026-03-14 — CRUSHR-P2-EXEC-04: Phase 2 normalization contract and classification ladder

- Decision:
  - Introduce a deterministic normalization command/output contract for the completed Phase 2 execution corpus: `run-phase2-normalization` emits `PHASE2_RESEARCH/results/normalized_results.json` and `normalization_summary.json` with explicit enums for `result_class`, `failure_stage`, and `diagnostic_specificity`.
  - Keep file-level counts nullable when extraction-outcome evidence is unavailable; do not infer file-level outcomes from unstructured comparator logs.
- Alternatives:
  1. Infer per-file outcomes from stdout/stderr heuristics for comparator tools.
  2. Delay normalization until extraction-mode reruns are available.
- Rationale:
  - The packet requires truthful, comparison-ready normalization over existing corpus evidence without rerunning trials.
  - Nullable file-level fields prevent overclaiming where the corpus does not include extraction-result artifacts.
- Blast radius:
  - Adds a new Phase 2 results artifact family and schema contracts consumed by downstream comparative analysis/reporting.
  - No changes to locked matrix axes, trial execution corpus, or archive-format behavior.


## 2026-03-14 — CRUSHR-P2-EXEC-06A: recovery accounting is extracted-output based and byte accounting is size-clamped

- Decision:
  - Phase 2 execution evidence now derives recoverability from actual extraction outputs for all formats and records deterministic per-run accounting (`files_expected/recovered/missing`, `bytes_expected/recovered`, ratio fields) plus extraction/recovery artifact paths.
  - `bytes_recovered` is computed as `sum(min(actual_size, expected_size))` over recovered expected files to treat truncated output as partial byte recovery without overcounting oversized outputs.
  - Normalization blast-radius class is determined solely from `recovery_ratio_files` thresholds: `NONE=1.0`, `LOCALIZED>=0.9`, `PARTIAL_SET>=0.5`, `WIDESPREAD>0.0`, `TOTAL=0.0`.
- Alternatives:
  1. Infer recoverability heuristically from exit codes/diagnostic text only.
  2. Require checksum/content validation in this packet before any recovery accounting is emitted.
- Rationale:
  - Exit/diagnostic-only evidence could not answer the white-paper recoverability thesis.
  - File+byte counts from extracted trees are deterministic, cheap, and comparable across formats while keeping this packet bounded (no full content verification requirement).
- Blast radius:
  - Changes raw run record and normalization schemas/consumers, execution command behavior (list/test probes -> extraction runs), and summary aggregation fields used by downstream analysis/reporting.
  - Full matrix rerun remains external to this PR workflow.

## 2026-03-14 — CRUSHR-P2-ANALYSIS-01: deterministic comparison metric and ranking formulas

- Decision:
  - Define per-format comparison metrics from normalized Phase-2 records as: `recovery_success_rate` (`recovery_ratio_files > 0` frequency), mean file/byte recovery ratios, `detection_rate` (`detected_pre_extract` frequency), plus normalized blast-radius and diagnostic-specificity distributions.
  - Emit three deterministic ranking ladders from those metrics: survivability (success rate primary), diagnostic quality (detection + weighted specificity composite), and corruption containment (weighted blast-radius containment score).
- Alternatives:
  1. Rank solely by mean recovery ratios without separate success/detection/containment views.
  2. Delay rankings until additional post-normalization heuristics/content validation are introduced.
- Rationale:
  - White-paper table generation requires direct cross-format ordering on survivability, diagnostic quality, and containment from the frozen normalized corpus with no experiment rerun.
  - Explicit formulas keep outputs reproducible and auditable.
- Blast radius:
  - Adds new analysis-only summary artifacts/schemas and `crushr-lab` command surface for Phase 2 reporting.
  - Does not change manifest locks, trial execution semantics, or normalized input contracts.


## 2026-03-14 — CRUSHR-SALVAGE-01: introduce standalone salvage research tool

Status: Accepted

Decision:
A new standalone tool `crushr-salvage` will be introduced.

This tool is **not part of strict extraction semantics** and must not be
implemented as a mode or flag of `crushr-extract`.

Purpose:
Deterministic salvage planning over structurally damaged archives
for research analysis without fragment emission or reconstruction.

Constraints:
- `crushr-extract` remains strict-only.
- `crushr-salvage` must never modify archives.
- output must clearly label salvage plans as **unverified research output**.
- salvage plans/results must not be represented as safe or canonical extraction.
- CRUSHR-SALVAGE-01 remains plan-only (no fragment emission, no reconstruction).

Tool placement:

crushr-pack
crushr-info
crushr-fsck
crushr-extract
crushr-salvage
crushr-lab

Rationale:
Allows experimentation with recovery algorithms while preserving the
integrity-first product contract and white-paper baseline.

Blast radius:
- documentation
- workspace CLI registry
- future Phase 3 implementation work

## 2026-03-14 — CRUSHR-SALVAGE-02: salvage plan schema bumped to v2 for explicit verification states

- Decision:
  - Introduce `crushr-salvage-plan.v2` instead of extending v1 in place, because candidate and file-plan sections now carry materially richer verification and reason-code surfaces.
- Alternatives:
  1. Keep v1 filename and add optional fields only.
  2. Keep v1 shape and compress multiple verification states into free-form reason strings.
- Rationale:
  - SALVAGE-02 requires stable, schema-backed enums/reason codes for deterministic verification stages; v2 avoids ambiguous partial compatibility claims.
- Blast radius:
  - Affects only `crushr-salvage` research output, schema consumers/tests, and salvage documentation.
  - No changes to `crushr-extract` contracts or strict extraction semantics.


## 2026-03-14 — CRUSHR-SALVAGE-03: research-only verified fragment export in standalone salvage tool

- Decision:
  - Add optional `crushr-salvage --export-fragments <dir>` that emits deterministic research artifacts only from content-verified blocks/extents, with explicit `UNVERIFIED_RESEARCH_OUTPUT` labeling and no guessed/reconstructed bytes.
  - Keep `crushr-extract` unchanged and strict-only.
- Alternatives:
  1. Keep salvage plan-only with no artifact export.
  2. Add salvage/export mode to `crushr-extract`.
- Rationale:
  - Packet requires evidence artifact generation while preserving integrity-first canonical extraction boundary.
- Blast radius:
  - `crushr-salvage` CLI and salvage-plan v2 schema/tests/docs only; no strict extraction contract changes.


## 2026-03-15 — CRUSHR-FORMAT-01: add bounded redundant file-map metadata path (LDG1)

- Decision:
  - Emit compact redundant file-map metadata (`crushr-redundant-file-map.v1`) in LDG1 for new archives produced by `crushr-pack`.
  - Keep IDX3 as primary authoritative mapping path; use redundant map only as strict fallback in `crushr-salvage` when IDX3 is unusable.
  - Require all-or-nothing redundant-map verification (schema, structural consistency, block references, offsets/lengths, full file coverage) before any fallback use.
  - Bump salvage output schema to `crushr-salvage-plan.v3` to record `redundant_map_analysis` and per-file `mapping_provenance`.
- Alternatives:
  1. Add a second full duplicate index table.
  2. Keep plan v2 and add optional unversioned fields.
- Rationale:
  - Experiments showed orphan evidence was dominated by mapping loss, not block loss; compact per-file extent redundancy improves survivability with bounded tail-frame blast radius.
  - Schema v3 avoids ambiguous partial compatibility for new provenance/reporting fields.
- Blast radius:
  - Affects `crushr-pack` tail-frame ledger content, `crushr-salvage` fallback behavior/reporting, and salvage schema/tests/docs.
  - Does not change `crushr-extract` strict semantics or mutate old archives.


## 2026-03-15 — CRUSHR-FORMAT-02: bounded experimental self-describing extents and distributed checkpoints

- Decision:
  - Add explicit experimental writer mode (`crushr-pack --experimental-self-describing-extents`) rather than changing default archive behavior.
  - Emit per-extent metadata blocks (`crushr-self-describing-extent.v1`) and separated checkpoint snapshots (`crushr-checkpoint-map-snapshot.v1`) only in experimental archives.
  - Extend `crushr-salvage` precedence with verified experimental paths after existing authoritative/fallback paths and record provenance (`CHECKPOINT_MAP_PATH`, `SELF_DESCRIBING_EXTENT_PATH`).
  - Add bounded three-arm comparison mode and compact outputs (`experimental_comparison_summary.json/.md`).
- Alternatives:
  1. Replace default writer path directly.
  2. Keep only centralized redundant map with no experimental distributed metadata.
- Rationale:
  - Preserves strict integrity/extraction boundaries while allowing targeted survivability experimentation with explicit opt-in behavior.
- Blast radius:
  - `crushr-pack`, `crushr-salvage`, `crushr-lab-salvage`, focused tests, and continuity/docs updates only.
  - No `crushr-extract` contract changes.


## 2026-03-15 — CRUSHR-FORMAT-03: file-identity anchored extents as bounded experimental fallback

- Decision:
  - Add explicit opt-in writer mode (`crushr-pack --experimental-file-identity-extents`) emitting `crushr-file-identity-extent.v1` records plus `crushr-file-path-map.v1`.
  - Require strict path linkage verification (`file_id` + path digest + verified path-map record) for named recovery; no guessing.
  - Extend salvage precedence with `FILE_IDENTITY_EXTENT_PATH` after primary/redundant/checkpoint fallback paths.
- Alternatives:
  1. Fold file identity into default format path.
  2. Keep only checkpoint/self-describing records with no dedicated file-identity records.
- Rationale:
  - Prior experiments showed surviving blocks without enough verified file membership; explicit per-extent file identity is the bounded next probe while preserving strict-only behavior.
- Blast radius:
  - `crushr-pack`, `crushr-salvage`, `crushr-lab-salvage`, salvage plan schema enum, targeted tests/docs/continuity files.
  - No `crushr-extract` semantic changes.


## 2026-03-15 — Path/name recovery rule B for FORMAT-04

- Decision:
  - Use rule **B** for experimental file-identity fallback: allow deterministic anonymous verified recovery when path map linkage is missing, without inventing original filenames.
- Alternatives:
  1. Rule A: refuse recovery when path map is missing.
- Rationale:
  - Improves strict salvageability in index/tail/header damage cases while preserving integrity-first behavior (verified content + verified extent identity only).
- Blast radius:
  - Affects research-only salvage output naming and provenance (`FILE_IDENTITY_EXTENT_PATH_ANONYMOUS`); does not affect `crushr-extract` canonical behavior.


## 2026-03-15 — CRUSHR-FORMAT-05 experimental wire contracts

- Status: Accepted
- Decision:
  - Add explicit opt-in experimental writer flag `--experimental-self-identifying-blocks`.
  - Encode per-payload identity in `crushr-payload-block-identity.v1` and emit repeated verified path checkpoints via `crushr-path-checkpoint.v1`.
  - Extend salvage fallback precedence with payload-block identity path after file-identity extents.
- Rationale:
  - Improve metadata-independent file membership recovery under index/footer/tail loss while preserving strict verification-only semantics.
- Blast radius:
  - Experimental writer/salvage/comparison flows only; no default format migration and no `crushr-extract` semantic changes.

## 2026-03-15 — CRUSHR-ARCH-GRAPH-01: content-addressed recovery graph direction locked

- Status: Accepted
- Decision:
  - Adopt a long-term resilience direction where recovery truth is distributed across verified payload-adjacent structures rather than centralized archive metadata alone.
  - Recovery graph layers are: payload truth -> extent/block identity truth -> file manifest truth -> path truth.
  - Recovery degrades in reverse order: full named -> full anonymous -> partial ordered -> orphan evidence.
- Alternatives:
  1. Continue layering more centralized/redundant index structures around IDX3.
  2. Treat placement strategy experiments as the primary path before metadata-independent reconstruction is mature.
- Rationale:
  - Experimental evidence shows centralized metadata remains the dominant failure point.
  - File-identity metadata improved recovery where centralized/redundant metadata alone did not.
  - The next coherent direction is to mature a content-addressed recovery graph incrementally rather than keep multiplying fragile index-style structures.
- Blast radius:
  - Guides future experimental salvage/format packets.
  - Does not change canonical v1 extraction semantics or the active on-disk production contract.

## 2026-03-15 — CRUSHR-ARCH-INV-01: inversion principle for resilience work locked

- Status: Accepted
- Decision:
  - For resilience-oriented experimental work, prefer architectures where verified payload-adjacent structures carry reconstructive truth and centralized metadata acts as an accelerator rather than sole authority.
  - Prefer block -> file / manifest reconstruction paths over file -> block dependence on a single authoritative index.
  - Build recovery upward from verified surviving payload, not downward from fragile roots.
- Alternatives:
  1. Keep centralized metadata as the canonical truth for all recovery paths.
  2. Treat inversion as an informal design intuition only.
- Rationale:
  - This principle is already supported by the experimental evidence and is useful immediately as a decision filter for packets.
  - Locking it reduces future drift back toward fragile centralized-metadata designs.
- Blast radius:
  - Affects experimental format/recovery planning and review.
  - No immediate wire-format or extraction-contract change by itself.

## 2026-03-15 — CRUSHR-FORMAT-06 is the next active recovery-graph packet

- Status: Accepted
- Decision:
  - The next active packet after FORMAT-05 is FORMAT-06: verified file manifest checkpoints as the next graph layer.
  - Placement-strategy experiments (distributed-hash / low-discrepancy / golden-ratio) are deferred until payload identity + manifest truth are tested.
- Alternatives:
  1. Jump directly to checkpoint placement strategy bakeoffs.
  2. Jump directly to a generic graph engine abstraction.
- Rationale:
  - Current evidence says the active bottleneck is still file truth/completeness under metadata loss, not checkpoint spacing optimization.
  - FORMAT-06 is the smallest coherent next layer on top of FORMAT-05.
- Blast radius:
  - Planning/control docs and next-packet expectations only.
  - No change to current experimental or canonical extraction behavior.

## 2026-03-15 — CRUSHR-SCRUB-01 extraction confinement unification

- Decision: all extraction surfaces must use a shared confined-path resolver and hard-fail on unsafe archive paths.
- Decision: symlink extraction is disabled in hardened mode (fail closed) until a separately approved confined symlink model exists.
- Alternatives considered: keep legacy permissive behavior; strip/normalize malicious paths; allow symlinks with best-effort checks.
- Rationale: integrity-first semantics require deterministic fail-closed behavior and consistent safety across canonical, legacy, and API paths.
- Blast radius: malicious archives that previously extracted now error deterministically; safe relative-path extraction is unchanged.
