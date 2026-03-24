<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/DECISION_LOG.md

## 2026-03-24 — CRUSHR_UI_POLISH_08 pack phase-row identity + info file-level terminology lock

- Decision:
  - Keep `pack` progress row identity stable by rendering `compression` and `serialization` as persistent shared active-phase rows (no alternating label multiplexing) and preserve explicit `finalizing` phase transition after both rows complete.
  - Update `info` human Structure labels to file-level model wording: `files`, `compressed units`, `file mappings`, and explicit `block model = file-level (1:1 file → unit)`.
  - Treat `info` terminology update as presentation-only; do not alter internal index/block counting or archive format behavior.
  - Align canonical product version to `0.3.5` for this packet.
- Alternatives considered:
  1. Keep alternating/multiplexed row identity for compression/serialization under one active row.
  2. Keep internal jargon labels (`regular files`, `payload blocks`, `extents referenced`) despite user confusion in file-level mode.
- Rationale:
  - Stable labels are required for operator trust in live phase tracking.
  - Current archive behavior is file-level 1:1, so user-facing terms must reflect that model directly to avoid misleading mental models.
  - Packet explicitly scopes changes to UI correctness and clarity without format/runtime model redesign.
- Blast radius:
  - Human `info` output shape changed; golden fixture updated.
  - Pack runtime behavior remains same execution path but row-identity stability remains explicit and locked through shared active-phase usage.
  - VERSION/workspace package versions changed to `0.3.5`.

## 2026-03-24 — CRUSHR_UI_POLISH_07 help/extension/progress/metrics/info compression truth lock

- Decision:
  - Route help output for core product commands (`crushr`, `crushr-pack`, `crushr-extract`, `crushr-info`) through shared `CliPresenter` sections/tokens so help colorization follows the same semantic palette as runtime command output.
  - Normalize pack output archive paths by appending `.crs` only when the user-supplied `-o/--output` has no extension; preserve explicit user extensions unchanged.
  - Split pack progress truth into explicit `compression` and `serialization` phases that both settle at `files=N/N`, then show a visible `finalizing` phase before result emission.
  - Expand pack final result rows with truthful runtime/compression metrics computed from real run values (input logical bytes, emitted archive bytes, measured elapsed duration).
  - Expand `info` human output with a dedicated `Compression` section (`method`, `level`) derived from parsed BLK3 headers; fall back to `unavailable` when data cannot be recovered.
- Alternatives considered:
  1. Keep static/plain help strings and colorize only command runtime output.
  2. Keep single `serialization` progress phase and rely on implicit tail closeout without explicit `finalizing`.
  3. Report only `files packed` in pack results and defer runtime/compression metrics to later packet work.
- Rationale:
  - Packet requires user-facing truth improvements, not cosmetic-only updates; hidden finalization and N-1/N end-state were trust regressions.
  - Shared help rendering avoids per-command style drift and keeps no-color/non-TTY behavior clean by reusing presenter gating.
  - `.crs` defaulting improves consistency without overriding intentional operator-specified extensions.
  - Compression metadata and metrics are now derived from real archive/runtime data, preventing fabricated values.
- Blast radius:
  - Human help output text layout changed for core commands; wrapper equivalence/behavior remains intact.
  - Human `pack` and `info` output shapes changed; updated CLI golden fixtures and harness expectations accordingly.
  - Lab harness identity-archive ordering expectation updated for extensionless output normalization (`c` -> `c.crs`).


## 2026-03-24 — CRUSHR_UI_POLISH_06 canonical divider/alignment lock + product-grade info summary

- Decision:
  - Standardize shared presenter title rows on one canonical style: leading blank line + double-line divider + shared color semantics, and route key/value alignment through padding-before-colorization so ANSI output does not shift the value column.
  - Rework `about` to use the same shared visual contract (title spacing/divider, token-based color semantics, and aligned key/value widths) instead of a bespoke formatter.
  - Promote `info` human mode from sparse/internal fields to product-facing archive inspection: surface regular file count, extent references, logical bytes, payload block count, dictionary table/ledger presence, and compression-level summary derived from block headers when available.
  - Remove raw internal label leakage (`has dct1`) from primary `info` output; translate to dictionary-table language.
- Alternatives considered:
  1. Keep `about` as a separate plain-text layout and only tweak wording.
  2. Leave colorized alignment drift unresolved and rely on no-color output for clean columns.
  3. Keep `info` minimal/internal and push richer inspection to a future verbose mode only.
- Rationale:
  - Packet requires suite-wide visual consistency and explicit removal of internal jargon from user-facing inspection.
  - Padding labels before token coloring prevents right-column drift in color-enabled terminals without changing no-color/non-TTY behavior.
  - Block-header scan enables truthful compression-level reporting; when unavailable, output explicitly says so rather than inventing values.
- Blast radius:
  - Human CLI output shape changed across core command goldens due canonical divider/newline policy.
  - `about` and `info` human outputs changed; `info --json` contract remains unchanged.
  - Version advanced to `0.3.5` for v0.3.x CLI/inspection product-surface milestone.

## 2026-03-24 — CRUSHR_UI_POLISH_04 pack live-detail polish + non-TTY artifact lock

- Decision:
  - Keep command-specific motion refinements routed through shared `cli_presentation::ActivePhase` primitives; for `pack`, expose real serialization progress detail via `set_detail(files=<done>/<total>)` and settle with a final stable count detail.
  - Add explicit integration coverage asserting non-TTY command output remains artifact-free (no `\r` redraw control or clear-line escape remnants) even with `CRUSHR_MOTION=full`.
- Alternatives considered:
  1. Leave pack serialization as a detail-free running phase to avoid minor output drift.
  2. Rely only on manual checks for non-TTY cleanliness.
- Rationale:
  - Packet requires practical refinement on real command UX while preserving shared motion ownership and copy/paste-safe final output.
  - Contract-level non-TTY checks prevent regressions where future motion changes leak terminal-control noise into logs/pipes.
- Blast radius:
  - Human pack progress output now includes stabilized serialization detail in final settled row.
  - `cli_presentation_contract` gained a non-TTY artifact guard for pack/verify/extract/recover flows.
  - No archive semantics, JSON contracts, or public CLI flag surface changes.

## 2026-03-23 — CRUSHR_UI_POLISH_03 restrained shared CLI motion policy + active-phase animation layer

- Decision:
  - Add one shared active-phase motion layer in `cli_presentation` (`begin_active_phase` / `ActivePhase`) with centralized motion policy, TTY gating, and stable phase settlement behavior.
  - Lock restrained animation to active progress rows only and keep final/settled sections static.
  - Introduce explicit motion controls (`CRUSHR_MOTION=full|reduced|off`, `CRUSHR_NO_MOTION=1`) and ensure non-TTY output never emits spinner carriage-control noise.
  - Apply the shared active-phase flow to `pack`, `extract`, and `verify` progress rendering; keep `info` static.
- Alternatives considered:
  1. Keep command-local phase animation logic.
  2. Add richer full-screen TUI redraw loops for progress.
- Rationale:
  - Packet requires semantic, calm motion centralized in shared presentation code with no fake progress and no command-by-command drift.
  - TTY-gated single-line active updates preserve readability and keep logs/pipes clean.
- Blast radius:
  - Human progress section rows in non-interactive output now settle as stable completion/failure rows instead of long-lived `RUNNING` placeholders.
  - Added new contract doc `.ai/contracts/CLI_MOTION_POLICY.md` and refreshed progress goldens.
  - No archive format, extraction semantics, or machine JSON contract changes.

## 2026-03-23 — CRUSHR_UI_POLISH_02 shared structural CLI presentation primitives

- Decision:
  - Extend `cli_presentation` with composable structural primitives (`title_block`, `phase`, `banner`, `result_summary`) and keep existing section/key-value/token behavior as the common base for all core commands.
  - Migrate `pack`, `extract`, `extract --recover`, `verify`, and `info` presentation paths to these primitives so title/target/progress/result hierarchy is stable and warnings/failures use explicit shared banner framing.
  - Keep progress tied to real execution boundaries (no redraw theater), and keep non-color output unchanged in readability.
- Alternatives considered:
  1. Keep command-local formatting helpers while only documenting preferred layout.
  2. Add richer TUI-style live rendering/redraw behavior in this packet.
- Rationale:
  - Packet requires reusable layout building blocks before any animation work and forbids one-off command presentation drift.
  - Shared primitives lower maintenance cost and reduce future contract/golden churn.
- Blast radius:
  - Human CLI output text/section naming changed in migrated commands (notably `Target` section usage and shared warning/failure banners).
  - Golden presentation fixtures were updated to lock the new structure.
  - No archive format, verification model truth, or machine JSON contracts changed.

## 2026-03-23 — CRUSHR_UI_POLISH_01 shared CLI visual semantics contract

- Decision:
  - Centralize user-facing CLI visual semantics in one shared token system (`VisualToken`) and one shared status vocabulary (`PENDING`, `RUNNING`, `COMPLETE`, `DEGRADED`, `FAILED`, `REFUSED`) in `cli_presentation`.
  - Treat prior human-output `PARTIAL` semantics as compatibility input only and render it as `DEGRADED` to avoid overloaded/ambiguous wording.
  - Render recovery trust classes explicitly in recover-mode output (`CANONICAL`, `RECOVERED_NAMED`, `RECOVERED_ANONYMOUS`, `UNRECOVERABLE`) with distinct visual tokens.
- Alternatives considered:
  1. Keep command-local status wording and color decisions with only style-guide documentation.
  2. Preserve `PARTIAL` as the primary degraded user-facing term.
- Rationale:
  - Packet requires one reusable semantic visual language before deeper motion/polish work and forbids per-command improvisation.
  - `DEGRADED` communicates degraded-but-usable behavior more clearly than overloaded `PARTIAL` in recovery-aware contexts.
- Blast radius:
  - Human/silent CLI status strings changed where `PARTIAL` was previously presented.
  - Golden presentation fixtures and recovery validation assertions were updated.
  - No archive format, extraction safety policy, or machine JSON schema contracts changed.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_03 confidence-tiered content typing contract

- Decision:
  - Introduce a dedicated modular recovery content-classification engine for recovered payloads, using ordered detection (magic signature, secondary header/structure checks, confidence assignment).
  - Separate manifest trust class from content typing: add `recovery_kind` and redefine `classification` to represent detected content metadata (`kind`, `confidence`, `basis`, optional `subtype`).
  - Keep fail-closed policy: unknown/weak evidence downgrades to medium/low confidence and never upgrades optimistically to high.
- Alternatives considered:
  1. Keep prior minimal extension heuristics with `classification.kind` overloaded as trust class.
  2. Keep trust class only in manifest and skip structured content typing.
- Rationale:
  - Packet requires wide-format typing with explicit confidence boundaries and strict no-guessing behavior.
  - Separating trust class and content classification removes semantic overloading and makes manifest automation unambiguous.
- Blast radius:
  - Recovery manifest schema and recover integration tests updated for new field semantics.
  - Recover anonymous naming now derives from classification confidence tiers.
  - No change to strict extraction default behavior outside `--recover` outputs.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_01 recovery-aware extract contract

- Decision:
  - Keep `crushr-extract` default behavior strict, and add explicit `--recover` mode for recovery-aware extraction.
  - In recover mode, enforce segregated output directories (`canonical/`, `recovered_named/`, `_crushr_recovery/anonymous/`) and always emit `_crushr_recovery/manifest.json`.
  - Lock trust-class vocabulary and anonymous naming policy in code/schema: `canonical`, `recovered_named`, `recovered_anonymous`, `unrecoverable`; high/medium/low confidence naming patterns.
- Alternatives considered:
  1. Keep recovery as a separate primary `salvage` UX path.
  2. Mix recovered output into the canonical extraction directory.
- Rationale:
  - Packet explicitly requires recovery to be integrated into extraction while preventing silent canonical/recovered mixing.
  - Deterministic directory and manifest contracts make trust boundaries explicit for operators and automation.
- Blast radius:
  - `crushr-extract` CLI usage now accepts `--recover` (extract-only; rejected with `--verify`).
  - Recovery-mode extraction writes additional filesystem outputs and a recovery manifest schema contract.
  - Added integration tests for clean and damaged recovery-mode runs.

## 2026-03-23 — CRUSHR_VERIFY_SCALE_01 bounded verify execution + phase progress visibility

- Decision:
  - Replace verify-time strict extraction temp-output workflow with a bounded verify-only strict pass that validates extents/decompression without writing files.
  - Remove cross-run decompressed block payload caching from strict extraction so block payload bytes are not retained/cloned across the whole run.
  - Add explicit user-visible verify progress stages (`archive open / header read`, `metadata/index scan`, `payload verification`, `manifest validation`, `final result/report`) to the human verify surface.
- Alternatives considered:
  1. Keep temp-directory extraction in verify and only add progress text.
  2. Keep payload cache but cap size heuristically.
- Rationale:
  - Packet requires a production memory-scaling fix, not presentation-only changes.
  - Verify should surface real execution phases while preserving strict refusal semantics and deterministic reporting.
- Blast radius:
  - `crushr-extract --verify` runtime now executes strict validation without materializing output files.
  - CLI verify human output now includes a deterministic Progress section in success and structural-failure paths.
  - Golden presentation fixtures/tests were updated; JSON/silent contracts remain unchanged.

## 2026-03-22 — CRUSHR_CLI_UNIFY_04 production-vs-lab pack surface boundary

- Decision:
  - Restrict public `crushr-pack`/`crushr pack` CLI parser/help surface to production controls only (`<input>...`, `-o/--output`, `--level`, shared `--silent`), with no compatibility/deprecated/hidden acceptance for experimental format/layout/profile flags.
  - Relocate experimental writer controls to an explicit lab-owned surface `crushr lab pack-experimental ...` and route lab comparison harness pack invocations through that lab surface.
- Alternatives considered:
  1. Keep experimental flags on public `pack` and only reword help.
  2. Keep compatibility parsing for removed flags as hidden/deprecated aliases.
- Rationale:
  - Packet locks require a hard production-vs-lab boundary and explicitly forbid hidden compatibility acceptance for removed production experimental flags.
  - Lab workflows still need deterministic access to experimental controls, so relocation keeps research capability without polluting production operator UX.
- Blast radius:
  - Public pack invocation contract is stricter; prior experimental pack flags now fail on production pack path.
  - Lab comparison harness pack resolution now targets `crushr` and invokes `lab pack-experimental`.
  - Integration tests and help-surface assertions were updated to enforce the new boundary.

## 2026-03-22 — CRUSHR_CLI_UNIFY_03 CLI contract enforcement + hidden-alias purge

- Decision:
  - Add explicit integration-level CLI contract tests (`crates/crushr/tests/cli_contract_surface.rs`) that fail closed on command-taxonomy drift, wrapper/canonical help-about-version divergence, legacy alias resurfacing, and shared-flag contract drift.
  - Remove remaining undocumented positional alias branches by recognizing wrapper/command `--help`/`--version` controls only as first arguments.
  - Keep JSON precedence over silent presentation for combined `--json --silent` usage and lock that behavior in tests.
- Alternatives considered:
  1. Keep existing presentation tests only and rely on manual review for contract drift.
  2. Retain positional `--help`/`--version` acceptance as permissive compatibility behavior.
- Rationale:
  - Packet requires enforceable product-surface contracts and explicit negative tests for legacy alias reintroduction.
  - Positional help/version handling was undocumented behavior and created hidden parser branches that could mask invalid invocations.
- Blast radius:
  - Wrapper/command argument parsing is stricter for misplaced help/version flags.
  - New contract tests will fail immediately on future surface drift in taxonomy/help/about/version/flag semantics.

## 2026-03-22 — CRUSHR_CLI_UNIFY_02 retained-wrapper unification and fsck binary removal

- Decision:
  - Make retained companion binaries (`crushr-pack`, `crushr-extract`, `crushr-info`, `crushr-salvage`) thin wrappers over one shared wrapper-entry helper (`crushr::wrapper_cli::run_wrapper_env`) rather than keeping wrapper-local help/version/about/presentation branches.
  - Move salvage runtime implementation to shared library command ownership (`crushr::commands::salvage`) so both `crushr salvage` and `crushr-salvage` execute the same in-process command path.
  - Remove deprecated `crushr-fsck` binary from active build outputs and treat it as non-retained product surface.
- Alternatives considered:
  1. Keep `crushr-salvage` as standalone binary logic with top-level process forwarding.
  2. Keep `crushr-fsck` as deprecated shim for compatibility.
- Rationale:
  - Packet requires wrapper binaries to be thin and to avoid duplicate parser/help/about/version implementations.
  - Packet explicitly calls for deleting fsck-era compatibility surfaces and undocumented legacy tool names.
- Blast radius:
  - Wrapper help/version/about output text changed to canonical wrapper mapping model.
  - `crates/crushr` bin-target declaration changed to explicit retention list.
  - Tests/docs expecting fsck shim were updated for removed-binary behavior.

## 2026-03-22 — CRUSHR_CLI_UNIFY_01 canonical shared-app CLI wiring

- Decision:
  - Make top-level `crushr` the canonical command host with shared parse/dispatch (`cli_app`) and in-process execution for canonical commands.
  - Extract `crushr-pack`, `crushr-extract`, and `crushr-info` runtime entrypoints into shared library modules (`crushr::commands::{pack,extract,info}`) and keep binary targets as thin wrappers only.
  - Promote `crushr-lab` to expose library dispatch (`crushr_lab::dispatch`) and wire `crushr lab` through crate dependency (`crushr-lab`).
  - Remove obsolete placeholder crate `crushr-cli-common` from workspace membership.
- Alternatives considered:
  1. Keep top-level process dispatch to sibling binaries.
  2. Add compatibility shims/aliases while retaining legacy dispatch paths.
- Rationale:
  - Packet requires hard removal of top-level external-process dispatch for canonical command ownership and a single authoritative CLI command model/help/about/version boundary.
- Blast radius:
  - Workspace dependency graph changes (`crushr` now depends on `crushr-lab`; `crushr-cli-common` removed).
  - Top-level command execution path and binary ownership boundaries changed.
  - No archive format or strict extraction semantics changes.

## 2026-03-21 — CRUSHR-CHECK-02-FIX1 follow-up review adjustments

- Decision:
  - Revert `.github/SECURITY.md` from the CRUSHR-CHECK-02 patch per review direction.
  - Keep unified `policy-gate` workflow unchanged and make style enforcement pass by running repository-wide `cargo fmt` cleanup.
- Alternatives considered:
  1. Keep `.github/SECURITY.md` despite review request.
  2. Keep formatting drift and tolerate failing style job.
- Rationale:
  - Follow-up packet instructions explicitly required undoing `SECURITY.md` and making policy-gate style checks green.
- Blast radius:
  - Documentation/policy files and formatting-only source changes.
  - No archive format or runtime semantic changes.

## 2026-03-21 — CRUSHR-CHECK-02 unified policy-gate baseline (secrets/audit/MSRV/style/version)

- Decision:
  - Replace separate `trufflehog` and `cargo-audit` workflows with a single `policy-gate` workflow that runs on pull requests and pushes to `main`.
  - Enforce one high-signal baseline: TruffleHog verified-only secret scanning, `cargo audit --deny warnings`, MSRV check on Rust 1.85.0, style checks (`check-crate-policy`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`), and root `VERSION` drift validation.
  - Add explicit audit exception policy in `.cargo/audit.toml` for `RUSTSEC-2025-0119` only (transitive `number_prefix` warning), keeping all other warnings/advisories fail-closed.
- Alternatives considered:
  1. Keep multiple scattered workflows for each check category.
  2. Keep `cargo audit` default warning behavior and allow unmaintained advisories to pass silently.
  3. Narrow fmt/clippy scope to avoid exposing existing drift.
- Rationale:
  - A unified policy gate gives one obvious maturity surface and avoids badge/workflow sprawl.
  - Explicit exception files are auditable; silent warning acceptance is not.
  - Style enforcement remains truthful even with known pre-existing rustfmt drift.
- Blast radius:
  - GitHub Actions governance surface and contributor PR expectations.
  - README badge row now reflects workflow-backed checks.
  - No archive format/runtime extraction behavior changes.

## 2026-03-21 — CRUSHR-CRATE-01 crate-governance lock (MSRV + metadata inheritance + publish intent)

- Decision:
  - Lock workspace crate policy to `resolver = "3"`, `edition = "2024"`, and initial MSRV `rust-version = "1.85"` in `[workspace.package]`.
  - Require publishable crates to inherit crates.io-facing metadata from workspace (`version`, `edition`, `rust-version`, `license`, `authors`, `repository`, `homepage`, `documentation`, `keywords`, `categories`) and carry crate-specific `description` + `readme`.
  - Treat `crushr-cli-common`, `crushr-lab`, and `crushr-tui` as internal crates with explicit `publish = false`.
  - Add fail-closed policy validation via `scripts/check-crate-policy.sh`.
- Alternatives considered:
  1. Keep MSRV at 1.86 to match current toolchain and skip pinned governance policy.
  2. Leave publish intent implicit based on historical use and omit explicit `publish = false` for internal crates.
  3. Duplicate full metadata in each crate manifest rather than enforcing workspace inheritance.
- Rationale:
  - Packet locks require an explicit initial MSRV and explicit publishability intent with no ambiguity.
  - Workspace inheritance reduces drift and simplifies future metadata governance.
  - A scripted drift check prevents silent manifest sediment and policy regression.
- Blast radius:
  - Cargo manifests and release metadata policy across all workspace crates.
  - Adds one policy-check script under `scripts/`; no runtime archive/extraction behavior changes.

## 2026-03-20 — CRUSHR-UI-02 public CLI surface realignment + verify structural-failure presentation lock

- Decision:
  - Convert top-level `crushr` into a focused dispatcher aligned to canonical commands (`pack`, `extract`, `verify`, `info`) and bounded non-primary commands (`salvage`, `lab`).
  - Remove legacy generic-compressor command exposure (`append`, `list`, `cat`, `dict-train`, `tune`, `completions`) from the primary help surface and return explicit demotion guidance when invoked.
  - Render `crushr-extract --verify` structural failures through deterministic operator-facing refusal presentation (with bounded failure-domain/reason wording) instead of printing raw parser internals in normal output mode.
- Alternatives considered:
  1. Keep the legacy monolithic `crushr` command map and only update wording.
  2. Keep raw parser errors in verify path for all output modes.
- Rationale:
  - Product surface must match the preservation-oriented suite and remain small/coherent.
  - Raw parse internals are unstable and not operator-grade as primary failure presentation.
- Blast radius:
  - Changes top-level `crushr --help` and command routing behavior.
  - Changes non-JSON verify structural failure presentation text in `crushr-extract --verify`.
  - No archive format, extraction semantics, or salvage schema contract changes.

## 2026-03-20 — CRUSHR-UI-01 unified CLI presentation + silent-mode contract

- Decision:
  - Introduce one shared CLI presentation helper (`crates/crushr/src/cli_presentation.rs`) for public runtime tools in scope.
  - Standardize a bounded user-facing status vocabulary: `VERIFIED`, `OK`, `COMPLETE`, `PARTIAL`, `REFUSED`, `FAILED`, `RUNNING`, `SCANNING`, `WRITING`, `FINALIZING`.
  - Standardize `--silent` across `crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` to emit deterministic one-line summaries for scripting.
- Alternatives considered:
  1. Keep command-local ad-hoc output and add only style guidance docs.
  2. Use independent per-binary formatting with no shared helper.
- Rationale:
  - Shared rendering primitives reduce wording/status drift and establish one product identity before benchmark-harness expansion.
  - A common `--silent` contract removes command-specific scripting surprises.
- Blast radius:
  - Affects CLI human-output/help behavior for pack/extract/verify/salvage.
  - Does not alter archive semantics, strict extraction verification boundaries, or salvage research contract fields.

## 2026-03-18 — CRUSHR-HARDEN-03G canonical verify truth boundary

- Decision:
  - Introduce `crushr-core::verification_model::VerificationModel` as the canonical typed verification truth model for strict verification reporting.
  - Require `crushr-extract --verify` output/report assembly to be derived from that model rather than from ad-hoc direct formatting of extraction internals.
- Alternatives considered:
  1. Keep direct `VerifyReport` assembly from `strict.report` in `crushr-extract`.
  2. Build a verification model only in the CLI layer.
- Rationale:
  - Centralizing verification truth in `crushr-core` reduces output drift risk and keeps format/internal changes from leaking into reporting semantics.
  - Core-level model construction allows deterministic tests on truth assembly independent of output rendering.
- Blast radius:
  - Affects strict verify report wiring in `crushr-extract`.
  - No strict extraction behavior or public archive format contract changes.

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

## 2026-03-16 — Unix metadata preservation is a product-completeness track, not a resilience detour

- Status: Accepted
- Decision:
  - Add a future explicit product-completeness workstream for Unix file-object metadata preservation.
  - The first bounded envelope should cover at least:
    - file type
    - mode
    - uid/gid
    - optional uname/gname policy
    - mtime policy
    - symlink target
    - xattrs
- Alternatives considered:
  1. Keep crushr focused on content bytes only and defer Unix metadata indefinitely.
  2. Attempt to implement every advanced Unix metadata surface at once.
- Rationale:
  - On Unix-like systems, tar earns trust because it preserves the surrounding file object, not just file bytes.
  - A bounded first envelope closes the most credible “tar does more” objection without dragging the project into an ACL/device-label abyss all at once.
- Blast radius:
  - Planning/roadmap/control docs now treat Unix metadata preservation as a real future product track.
  - No immediate canonical extraction or wire-format change by this decision alone.

## 2026-03-16 — Distributed dictionary work is a later optimization track gated on structural stability

- Status: Accepted
- Decision:
  - Reintroduce distributed dictionary experiments only after the current resilience architecture, metadata-layer pruning, and placement/grid evaluation stabilize.
  - Dictionary work must follow the same integrity-first rules as other format features:
    - explicit dictionary identity
    - verifiable block -> dictionary dependency
    - deterministic degradation when a required dictionary is missing
    - no silent decode fallbacks that change truth
- Alternatives considered:
  1. Return to dictionary optimization immediately.
  2. Treat dictionaries as a simple compression-only concern disconnected from recoverability.
- Rationale:
  - Compression tuning before structural stability would optimize an artifact whose supporting metadata story is still being validated.
  - In crushr, dictionaries are not just compression aids; they become part of the verifiable dependency graph and therefore require deliberate timing.
- Blast radius:
  - Backlog/roadmap/status documents now represent dictionaries as a post-stabilization optimization track.
  - No change to the current canonical v1 contract or experimental recovery packets.


## 2026-03-16 — CRUSHR-FORMAT-10 metadata pruning profile surface

- Decision:
  - Add explicit experimental packer profile surface `--metadata-profile <payload_only|payload_plus_manifest|payload_plus_path|full_current_experimental>`.
  - Keep default behavior unchanged unless profile is explicitly selected.
  - Add `run-format10-pruning-comparison` as the bounded four-arm recovery/size audit command.
- Alternatives:
  1. Reuse old boolean flags only and infer pruning variants in lab code.
  2. Add more than four profiles in the same packet.
- Rationale:
  - Explicit profile names make experiments reproducible and keep packet scope bounded to evidence-driven pruning.
  - Single switch avoids ambiguous flag combinations and supports deterministic reporting.
- Blast radius:
  - Experimental writer/lab interfaces only; canonical extraction semantics are unchanged.
  - Existing format09 and earlier commands continue to run without behavior changes.

## 2026-03-16 — CRUSHR-FORMAT-11 distributed extent identity profile surface

- Decision:
  - Add explicit experimental packer profile `--metadata-profile extent_identity_only`.
  - Encode distributed per-extent structural identity using `crushr-payload-block-identity.v1` records with local fields (`file_id`, `block_index`/extent index, `total_block_count`, `logical_length` + `payload_length`, and `content_identity` digests).
  - Do not include path/name in local extent identity records for this packet.
  - Add `run-format11-extent-identity-comparison` as the bounded four-arm command (`payload_only`, `payload_plus_manifest`, `full_current_experimental`, `extent_identity_only`).
- Alternatives:
  1. Keep manifest-first approach and defer distributed identity.
  2. Include names directly in local headers, conflating structure and path metadata.
- Rationale:
  - Tests structure-first anonymous recovery capability while minimizing global manifest dependency.
  - Preserves strict semantics by requiring verified local identity and avoiding speculative naming.
- Blast radius:
  - Experimental writer/salvage/lab surfaces only; canonical extraction behavior remains unchanged.

## 2026-03-16 — CRUSHR-FORMAT-12 inline naming remains experimental

- Decision: introduce `extent_identity_inline_path` as an opt-in metadata profile only; do not change default archive behavior or extraction semantics.
- Rationale: collect bounded evidence on named recovery gain vs duplication overhead before any keep/prune lock.
- Blast radius: `crushr-pack`, `crushr-salvage`, and `crushr-lab-salvage` experimental comparison/reporting only.

- Update (same packet): `extent_identity_distributed_names` is retained as a required FORMAT-12 comparison arm (distributed path checkpoints without inline per-extent path duplication) for direct evidence against inline naming and manifest-heavy controls.

## 2026-03-16 — FORMAT-13 dictionary identity fail-closed policy

- Decision: dictionary-based naming recovery requires a verified surviving dictionary copy; if multiple surviving copies disagree, salvage does not guess and falls back to anonymous recovery.
- Alternatives considered: pick first-seen copy; majority vote across copies.
- Rationale: preserve deterministic, strict, fail-closed semantics under corruption.
- Blast radius: affects only experimental FORMAT-13 metadata profiles and lab-comparison salvage planning.

## 2026-03-16 — FORMAT-14A dictionary placement recommendation under direct dictionary-target corruption

- Decision: keep `extent_identity_path_dict_header_tail` as the lead dictionary-placement candidate; treat `extent_identity_path_dict_single` as too fragile under direct primary-dictionary damage.
- Alternatives considered:
  1. Keep single-copy dictionary placement as co-lead despite direct-target fragility.
  2. Re-introduce quasi-uniform as lead for this packet.
- Rationale:
  - Direct dictionary-target scenarios now explicitly demonstrate the required fail-closed behavior and conflict handling.
  - Header+tail preserves named recovery when one copy is lost while still failing closed to anonymous recovery when both copies are unavailable or inconsistent.
- Blast radius:
  - Affects experimental FORMAT-14A recommendation and next-step policy lock only.
  - No change to canonical `crushr-extract` semantics or default archive behavior.

## 2026-03-17 — CRUSHR-TOOLING-VERIFY-01: retire public `crushr-fsck` surface and move strict verification to `crushr-extract --verify`

- Status: Accepted
- Decision:
  - `crushr-fsck` is no longer a public-facing tool surface.
  - Strict archive verification for canonical extraction moves to `crushr-extract --verify <archive>`.
  - `crushr-salvage` remains the recovery-oriented analysis surface and is not merged into extract verification.
  - Keep a temporary compatibility shim binary `crushr-fsck` that exits with a deterministic deprecation message and nonzero status.
- Alternatives considered:
  1. Keep `crushr-fsck` as a first-class public tool.
  2. Merge verification and salvage behavior under one extract mode.
- Rationale:
  - Removes overlapping public tool identity and aligns strict verification with canonical extraction semantics.
  - Preserves strict-vs-salvage boundary and avoids speculative recovery behavior in `crushr-extract`.
- Blast radius:
  - CLI invocation docs/help/tests must use `crushr-extract --verify` for strict verification flows.
  - Legacy `crushr-fsck` JSON schema/snapshot internals remain only as transitional/internal artifacts and are no longer part of the public workflow.


## 2026-03-18 — CRUSHR-HARDEN-03B salvage contract reconciliation direction

- Decision:
  - Choose **Option B** (schema is correct, implementation drifted) for salvage-plan contract repair.
  - Keep schema contract version at `crushr-salvage-plan.v3` and align implementation to existing v3 vocabulary instead of introducing a new version.
  - Enforce typed output-boundary enums for mapping provenance, recovery classification, and contract reason codes.
- Alternatives considered:
  1. Option A: keep implementation labels (`*_VERIFIED`, `ORPHAN_EVIDENCE_ONLY`) and rewrite schema/docs to match drift.
  2. Option C: create v4 solely to preserve both contradictory vocabularies.
- Rationale:
  - v3 already defines the active public salvage-plan vocabulary; restoring code to v3 avoids mixed-version ambiguity and repairs trust in emitted artifacts quickly.
  - Typed enum emission at the output boundary prevents silent string drift regressions.
- Blast radius:
  - `crushr-salvage` JSON output labels changed to v3 canonical enums where drift existed.
  - Tests/docs/lab expectations referencing legacy labels were updated where they consumed salvage-plan contract values.
  - No changes to canonical strict extraction semantics (`crushr-extract`).

## 2026-03-20 — CRUSHR-UI-03 section-based CLI presentation + info default mode

- Decision:
  - Adopt a minimalist section-based CLI rendering contract across public operator commands with canonical per-command section templates and required terminal `Result` section.
  - Make `crushr-info` human-readable by default and preserve machine-readable snapshot output under explicit `--json`.
  - Map verify structural failures to structured failure-domain fields (`component`, `reason`, `expected`, `received`) instead of exposing raw parser error text in normal user output.
- Alternatives considered:
  1. Keep prior mixed presenter grammar (`==`, `--`, bracketed status lines) and only adjust wording.
  2. Keep `crushr-info` JSON-only and require wrapper tooling for human readability.
- Rationale:
  - Unified section templates reduce command-to-command output drift and improve operator scanability.
  - Human-readable default `crushr-info` aligns command behavior with the rest of the product surface while keeping JSON automation intact.
  - Structured failure-domain output maintains deterministic operator semantics and avoids leaking unstable parser internals.
- Blast radius:
  - Human output text for `crushr-pack`, `crushr-extract --verify`, `crushr-info`, and `crushr-salvage` changed.
  - JSON output contracts remain unchanged for verify/info/salvage.
  - Added golden fixtures/tests locking the new output contract.

## 2026-03-20 — CRUSHR-VERSION-01 canonical product version source lock

- Decision:
  - Root `VERSION` is the single canonical human-edited product version source.
  - `VERSION` must contain strict SemVer only (no `v` prefix, no comments/prose).
  - Active runtime/report/tool metadata version paths use `crushr::product_version()` sourced from `VERSION`.
  - `workspace.package.version` remains aligned to `VERSION` via sync tooling and explicit drift validation.
- Alternatives considered:
  1. Keep Cargo workspace version as manual source and derive runtime from `env!("CARGO_PKG_VERSION")` only.
  2. Keep multiple manual version surfaces (Cargo/runtime/docs) with reviewer-enforced consistency.
- Rationale:
  - Single-touch human version edits reduce drift risk and unblock consistent future `crushr about`/report/release surfaces.
  - Explicit SemVer + drift checks fail closed on malformed/mismatched state.
- Blast radius:
  - `crushr` runtime version reporting paths and lab tool-version fields now consume canonical `VERSION` accessor.
  - Version governance tooling/docs (`scripts/check-version-sync.sh`, `scripts/sync-version.sh`, `VERSION`, README/continuity notes) now define the bump workflow.

## 2026-03-20 — CRUSHR-UI-04 locked `crushr about` surface + bounded build metadata fallback

- Decision:
  - Add top-level `crushr about` as a locked product-identity surface with fixed section ordering and present-state wording.
  - Inject build metadata at compile time (`commit`, `built`, `target`, `rustc`) and require explicit `unknown` fallback when unavailable.
  - Protect output contract with deterministic golden/fallback/help-surface tests to prevent wording/spacing drift.
- Alternatives considered:
  1. Keep `about` dynamic/freeform under shared presenter templates.
  2. Omit build metadata fields when unavailable.
- Rationale:
  - Product identity wording must stay stable and non-speculative.
  - Explicit fallback avoids panics/empty fields while keeping output deterministic.
- Blast radius:
  - Adds `about` to top-level help and command routing.
  - Introduces compile-time metadata injection for `crushr` binary.
  - No archive format, extraction semantics, or salvage contract changes.

## 2026-03-20 — CRUSHR-BUILD-01 musl release path + environment-first metadata injection

- Decision:
  - Add a repo-root Podman/Alpine musl release build path (`Containerfile.musl` + `scripts/build-musl-release-podman.sh`) that injects metadata through environment variables.
  - Treat `VERSION` as canonical release version source and pass it via `CRUSHR_VERSION` during release builds.
  - Keep `build.rs` environment-first with bounded shell fallbacks and final `unknown` values to prevent panics in minimal/dev environments.
- Alternatives considered:
  1. Shell-only metadata discovery in all environments.
  2. No containerized musl build path in-repo.
- Rationale:
  - Release reproducibility needs explicit metadata control and a stable musl build recipe.
  - Local developer workflows still need safe fallback behavior when metadata tooling is absent.
- Blast radius:
  - Adds build artifacts/tooling files (`Containerfile.musl`, `.cargo/config.toml`, build script helper).
  - Changes compile-time metadata key names consumed by `crushr about` build display fields.
  - No archive format or extraction/salvage behavior changes.
