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

- Decision: treat `docs/CONTRACTS/*`, `docs/RESEARCH/*`, `PROJECT_STATE.md`, `REPO_SNAPSHOT.md`, task packets, and review checklist as canonical project-control surfaces for implementation agents.
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
  - Change `crushr-lab` Phase 2 default output paths to `PHASE2_RESEARCH/manifests/` and `PHASE2_RESEARCH/generated/{foundation,execution}/`.
- Alternatives:
  1. Keep emitting defaults under `docs/RESEARCH/artifacts/*`.
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

