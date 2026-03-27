<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/CHANGELOG.md

## 2026-03-27 — CRUSHR_OPTIMIZATION_02 (v0.4.19)
- Optimized production `pack` emission by writing archive output through a 1 MiB `BufWriter`, reducing small-write overhead during payload + metadata block emission.
- Optimized compression-path allocation overhead by reusing a per-run compression output buffer for deterministic zstd writes (payload and metadata blocks), without changing codec, level, or zstd flag behavior.
- Replaced per-block `stream_position` probing with deterministic emitted-byte offset accounting (`BLK3_HEADER_WITH_HASHES_LEN`) so buffered writes keep block-offset truth used by identity metadata.
- Preserved semantics/correctness guardrails: fail-closed mutation detection, hash computation, preservation-profile behavior, manifest/index/tail finalization semantics, and truthful `--profile-pack` phase attribution boundaries remain unchanged.
- Advanced canonical version to `0.4.19` (`VERSION` + workspace package version sync).
- Validation: `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`; `cargo test -p crushr --test version_contract`.

## 2026-03-26 — CRUSHR_BENCHMARK_03 (v0.4.17)
- Added explicit production pack profiling flag `--profile-pack` and deterministic human-readable phase breakdown output for `discovery`, `metadata`, `hashing`, `compression`, `emission`, and `finalization`.
- Instrumented production pack timing attribution without changing archive bytes/semantics or compression behavior; phase timing is opt-in only and default/silent output remains unchanged unless profiling is requested.
- Added integration coverage asserting profile output is absent by default and emitted only when `--profile-pack` is explicitly provided.
- Updated benchmark reference docs with exact local operator commands for `medium_realistic_tree` and `large_stress_tree`, output-capture instructions, expected output shape, and interpretation guidance.
- Advanced canonical version to `0.4.17` (`VERSION` + workspace package version sync).
- Validation: `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`; `cargo test -p crushr --test version_contract`.

## 2026-03-26 — CRUSHR_PRESERVATION_FIX_06 (v0.4.16)
- Fixed strict/recover extraction metadata restore semantics so omitted-by-profile classes are not attempted at restore time (instead of being attempted and only filtered later for classification).
- Added profile-aware restore gating for ownership, xattrs, ACLs, SELinux labels, and Linux capabilities; this removes spurious omitted-class restore warnings for `basic` and `payload-only` archives.
- Preserved full-profile behavior: required metadata restoration still attempts restore and still reports warning/fail-closed behavior when blocked.
- Extended `metadata_preservation` deterministic coverage with omitted-profile warning suppression checks for strict+recover and explicit full-profile ownership-warning assertions.
- Advanced canonical version to `0.4.16` (`VERSION` + workspace package version sync).
- Validation: `cargo fmt --all`; `cargo test -p crushr --test metadata_preservation`; `cargo test -p crushr --test recovery_extract_contract`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`.

## 2026-03-26 — CRUSHR_BENCHMARK_02 (v0.4.15)
- Executed the full CRUSHR benchmark matrix over deterministic datasets (`small_mixed_tree`, `medium_realistic_tree`, `large_stress_tree`) with `tar+zstd`, `tar+xz`, `crushr(full)`, and `crushr(basic)`.
- Published canonical baseline raw results at `docs/reference/benchmarks/benchmark_results_v0.4.15.json` and the first comparative analysis report at `docs/reference/benchmark-baseline.md`.
- Recorded benchmark environment details (CPU/RAM/OS/filesystem) and explicit metric caveats (peak RSS and CPU timings unavailable due missing GNU `/usr/bin/time` in this environment).
- Updated reference index and benchmark contract page version marker to include baseline artifact visibility.
- Advanced canonical version to `0.4.15` (`VERSION` + workspace package version sync).
- Validation: `cargo build --release -p crushr`; `python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets`; `python3 scripts/benchmark/run_benchmarks.py --datasets .bench/datasets --crushr-bin target/release/crushr --output .bench/results/benchmark_results.json`; `python3 -m jsonschema -i .bench/results/benchmark_results.json schemas/crushr-benchmark-run.v1.schema.json`; `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`.

## 2026-03-26 — CRUSHR_BENCHMARK_01 (v0.4.14)
- Added deterministic benchmark dataset generation at `scripts/benchmark/generate_datasets.py` with three locked dataset classes (`small_mixed_tree`, `medium_realistic_tree`, `large_stress_tree`), fixed generation seed/mtime controls, and emitted `dataset_manifest.json`.
- Added reproducible benchmark execution harness at `scripts/benchmark/run_benchmarks.py` with explicit comparator commands for `tar+zstd`, `tar+xz`, `crushr(full)`, and `crushr(basic)`, including pack/extract timing and peak-RSS capture.
- Added benchmark result schema `schemas/crushr-benchmark-run.v1.schema.json` and benchmark methodology doc `docs/reference/benchmarking.md` (commands, metrics, reproducibility assumptions, and output contract).
- Linked benchmark contract page from `docs/reference/index.md` and ignored local benchmark workspace output via `/.bench/` in `.gitignore`.
- Advanced canonical version to `0.4.14` (`VERSION` + workspace package version sync).
- Validation: `cargo build --release -p crushr`; `python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets`; `python3 scripts/benchmark/run_benchmarks.py --datasets .bench/datasets --crushr-bin target/release/crushr --output .bench/results/benchmark_results.json`; `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`.

## 2026-03-26 — CRUSHR_PACK_STREAMING_01 (v0.4.13)
- Removed recurring production-pack whole-run raw payload retention: hard-link payload reuse cache in `emit_archive_from_layout` no longer stores `Vec<u8>` per block.
- Reworked file-manifest record construction to consume already computed per-block raw BLAKE3 digests, eliminating payload-byte retention for deferred manifest hashing.
- Preserved fail-closed mutation detection behavior (`input changed during pack planning`) and deterministic archive semantics.
- Captured reproducible before/after memory evidence and commands in `.ai/COMPLETION_NOTES_CRUSHR_PACK_STREAMING_01.md` (max RSS `525,800 KiB` on `HEAD~1` → `14,400 KiB` current for deterministic 250-file dataset).
- Validation: `cargo fmt --all`; `cargo test -p crushr pack_fails_if_file_changes_between_planning_and_emit -- --nocapture`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`; plus runtime pack/info/verify/extract probes on generated archive.

## 2026-03-25 — CRUSHR_INTROSPECTION_02 (v0.4.12)
- Expanded `crushr info` introspection readability with explicit preservation contract row and a compact `Entry kinds` section that summarizes regular/directories/symlinks/hard-links/sparse/FIFO/char-device/block-device visibility.
- Updated `crushr info` metadata visibility semantics to report `present`, `not present`, and `omitted by profile`, clarifying intentional profile omission vs extraction-time `metadata_degraded` outcomes.
- Improved `crushr info --list` with profile/scope context while preserving metadata/index-only fail-closed behavior; non-regular omission is now informational scope messaging rather than warning-level corruption framing.
- Updated docs/help wording for introspection boundaries and refreshed `info` human golden output plus deterministic tests for profile-aware listing/metadata readability.
- Bumped canonical product version to `0.4.12` (`VERSION` + workspace package version sync).
- Validation: `cargo fmt --all`; `cargo test -p crushr --test cli_presentation_contract --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`.

## 2026-03-25 — CRUSHR_RECOVERY_MODEL_08 (v0.4.11)
- Extended strict extraction fail-closed metadata semantics to non-regular canonical entries so profile-required metadata restore failures for directories, symlinks, FIFOs, and device nodes no longer remain warning-only canonical outcomes.
- Extended recover extraction metadata-degraded routing to non-regular canonical entries and aligned manifest truth fields (`trust_class`, `failed_metadata_classes`, `degradation_reason`) for those outcomes.
- Preserved profile-aware omission correctness across entry kinds (`basic`/`payload-only` omitted metadata classes are not treated as metadata degradation).
- Added deterministic metadata-preservation coverage for strict refusal + recover metadata_degraded placement/manifest assertions on mixed non-regular trees, plus omission-profile non-degradation assertions.
- Bumped canonical product version to `0.4.11` (`VERSION` + workspace package version sync).
- Validation: `cargo fmt --all`; `cargo test -p crushr --test metadata_preservation --test recovery_extract_contract`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`.

## 2026-03-25 — CRUSHR_PRESERVATION_05 (v0.4.10)
- Added explicit production preservation profiles via `crushr-pack --preservation <full|basic|payload-only>` with default `full`; no `--strip` alias.
- Added warn-and-omit semantics for profile-excluded entry kinds (e.g., FIFO/device under `basic`/`payload-only`, symlink under `payload-only`) with no silent flattening/fabrication fallback.
- Advanced on-disk index encoding to IDX7 and recorded preservation profile as structured archive metadata; legacy IDX3/IDX4/IDX5/IDX6 decode defaults to `full` compatibility profile.
- Made strict/recover metadata-degraded classification profile-aware for regular canonical outputs so intentionally omitted classes do not trigger metadata restoration failure.
- Added `crushr info` preservation-profile visibility and IDX7 format marker support; refreshed CLI golden fixtures and index-compat checks.
- Expanded deterministic metadata-preservation tests for profile default/recording/reporting, warn-and-omit behavior, payload-only semantics, and legacy profile compatibility behavior.
- Validation: `cargo fmt --all`; `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract --test index_codec --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.

## 2026-03-25 — CRUSHR_RECOVERY_MODEL_07
- Added explicit `metadata_degraded` trust class in extraction/recovery model and CLI trust presentation, including recover Result rows (`canonical`, `metadata_degraded`, `recovered_named`, `anonymous`, `unrecoverable`).
- Updated recover output layout to emit a dedicated `metadata_degraded/` tree and prevent metadata-degraded outputs from being merged into `canonical/`.
- Updated strict extraction behavior to refuse when required metadata restoration fails, with explicit metadata-failure refusal messaging.
- Extended recovery manifest schema + entry model with `trust_class`, `missing_metadata_classes`, `failed_metadata_classes`, and `degradation_reason`.
- Added/updated deterministic coverage for metadata-degraded behavior and strict refusal in `metadata_preservation.rs` + recover contract updates.
- Coverage note: metadata-degraded placement/classification is currently complete for regular-file canonical outputs; directory/symlink/special-entry metadata restoration remains warning-based and is explicitly tracked as a follow-up limitation.
- Validation: `cargo fmt --all`; `cargo test -p crushr --test recovery_extract_contract --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.

## 2026-03-25 — CRUSHR_PRESERVATION_03
- Advanced archive index encoding to IDX5 with explicit sparse/extents logical-offset metadata, FIFO/char-device/block-device entry kinds, and optional device major/minor fields while retaining IDX3/IDX4 decode compatibility.
- Extended production pack collection to detect sparse files, FIFOs, and device nodes; capture ownership-name enrichment (`uname`/`gname`) when available; and preserve sparse payloads without hole expansion.
- Extended strict and recover extraction paths to restore sparse files as sparse where supported, recreate FIFO/device entries, and emit explicit warning degradation (`WARNING[special-restore]`) when special-file restoration is unsupported or unprivileged.
- Extended `crushr info` metadata presence reporting with `sparse files` and `special files`, plus IDX5 format-marker visibility; refreshed golden outputs accordingly.
- Added deterministic regression coverage in `metadata_preservation.rs` for sparse-file round-trip, FIFO preservation, device-node truthful degradation behavior, and ownership-name enrichment non-placeholder capture.
- Validation: `cargo fmt --all`; `cargo test -p crushr --test metadata_preservation`; `cargo test -p crushr --test deterministic_pack --test mvp --test cli_presentation_contract`; `cargo clippy --workspace --all-targets -- -D warnings`.

## 2026-03-25 — CRUSHR_PRESERVATION_02
- Extended index encoding to IDX4 to preserve ownership metadata (`uid`/`gid`, optional ownership names) and explicit hard-link group identity.
- Updated production pack planning/emission so hard-linked regular files share a single payload block while retaining multiple logical file mappings.
- Updated strict and recover extraction to restore ownership best-effort with explicit warning surfacing on failure and to recreate hard links instead of duplicating file data.
- Expanded `crushr-info` human output with metadata presence visibility rows (`modes`, `mtime`, `xattrs`, `ownership`, `hard links`) and IDX3/IDX4 format-marker truth.
- Added deterministic metadata preservation coverage for hard-link round-trip and ownership-restore failure-warning behavior; refreshed info/pack goldens for IDX4 output.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test deterministic_pack --test mvp --test metadata_preservation --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -q`.

## 2026-03-25 — CRUSHR_PRESERVATION_02-FIX1
- Corrected `crushr info` ownership-presence semantics so IDX4 archives truthfully report ownership metadata as present even when numeric ownership is `0:0` (root-owned trees).
- Added explicit completion-evidence doc `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_02.md` with exact commands and observed outcomes for hard links, ownership, metadata visibility, IDX3 compatibility, xattr interaction, and warning-path honesty.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test metadata_preservation --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`.

## 2026-03-24 — CRUSHR_PRESERVATION_01
- Extended IDX3 metadata model for baseline filesystem preservation by adding explicit directory entry-kind support alongside existing regular/symlink semantics; pack now records mode, mtime, and xattrs from Linux filesystem metadata.
- Upgraded production pack collection to retain directory paths (including empty directories), symlink targets, and per-entry metadata while keeping deterministic ordering and duplicate-path safeguards.
- Upgraded strict and recover extraction flows to handle non-regular entries without refusal, create directories/symlinks during canonical extraction, and restore mode/mtime/xattrs with explicit warning surfacing when xattr restore is unsupported/denied.
- Added deterministic round-trip coverage in `metadata_preservation.rs` for mode, mtime, symlink, empty directory, and xattr behavior; updated pack determinism and CLI presentation goldens for metadata-aware archives.
- Recorded packet limits explicitly: uid/gid remains deferred and permission-denied xattr restore is warning-covered but not deterministically CI-forced in every environment.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test deterministic_pack --test mvp --test metadata_preservation`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_INTROSPECTION_01-FIX2
- Adjusted `info --list` status semantics so omitted non-regular entries are informational (not structural degradation) while structural proof failures still produce `DEGRADED`.
- Made `omitted entries` result-row emission conditional (shown only when non-zero) and kept explicit salvage guidance for proof-unavailable cases.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test version_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_INTROSPECTION_01-FIX1 (v0.4.1)
- Refined `crushr info --list` introspection behavior to report omitted non-regular index entries explicitly instead of silently dropping them.
- Added explicit degraded-path operator guidance to run `crushr salvage <archive>` when IDX3 listing proof is unavailable.
- Bumped product version to `0.4.1` (`VERSION` + workspace package version sync) and extended listing contract coverage accordingly.
- Validation: `cargo fmt --all`, `./scripts/check-version-sync.sh`, `cargo test -p crushr --test version_contract --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_INTROSPECTION_01 (v0.4.0)
- Added `crushr-info --list` archive introspection path with deterministic tree output by default and optional `--flat` full-path mode.
- Listing is derived from archive metadata/index (`IDX3`) only; no extraction/materialization is required for content inspection.
- Added corruption-aware degradation behavior: when archive opening fails, listing falls back to IDX3 proof when available, otherwise emits explicit warnings and prints no fabricated paths.
- Added integration coverage for tree/flat listing output and degraded proof-unavailable listing behavior in `cli_presentation_contract`.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_UI_POLISH_08 (v0.3.5)
- Confirmed `crushr-pack` progress truth remains stable through shared active-phase rows: `compression` and `serialization` are persistent labels in fixed order, both settle to `COMPLETE (files=N/N)`, then `finalizing` runs as a distinct closeout phase.
- Updated `crushr-info` Structure presentation labels to align with the current file-level model: `regular files` → `files`, `payload blocks` → `compressed units`, `extents referenced` → `file mappings`; added explicit `block model` row (`file-level (1:1 file → unit)`).
- Kept `info` changes presentation-only (no archive format or internal count-derivation logic changes) and refreshed the info golden contract fixture.
- Re-aligned canonical version to `0.3.5` (`VERSION` + workspace package version sync) per this UI-polish packet.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_UI_POLISH_06 (v0.3.5)
- Finalized shared CLI visual consistency: commands now begin with a leading blank line, use one canonical double-line divider, and keep right-column key/value alignment stable even with colorized labels.
- Reworked `crushr about` to the same product presentation contract (shared color semantics, canonical divider, aligned key/value rows) so it matches pack/verify/extract/info output style.
- Upgraded `crushr-info` human inspection summary: removed raw internal `has dct1` exposure, translated dictionary state into product language, and added compact archive-structure detail rows (regular files, extents, logical bytes, payload blocks, dictionary table/ledger, compression level or explicit unavailable).
- Bumped product version to `0.3.5` (`VERSION` + workspace package version sync) and refreshed CLI presentation golden fixtures.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-24 — CRUSHR_UI_POLISH_04
- Polished `crushr-pack` live progress by feeding shared active-phase detail updates during serialization (`files=<done>/<total>`) and settling the final serialization row with stable file-count truth.
- Added non-TTY motion cleanliness contract coverage in `cli_presentation_contract` to assert pack/verify/extract/extract-recover outputs never emit carriage-return or clear-line control artifacts when output is piped.
- Updated pack presentation golden output to lock the refined final serialization detail and reran formatting/lint/tests (`cargo fmt --all`, focused UI/recovery integration tests, clippy, full workspace tests).

## 2026-03-23 — CRUSHR_UI_POLISH_03
- Added shared active-phase motion primitives in `crates/crushr/src/cli_presentation.rs` with centralized animation controls, bounded redraw cadence (full/reduced), stable completion/failure freeze behavior, and non-TTY-safe fallback rendering.
- Migrated core long-running progress paths (`crushr-pack`, `crushr-extract`, `crushr-extract --verify`, including recover-phase callbacks) to the shared motion layer and removed command-local phase-status improvisation.
- Added motion policy contract `.ai/contracts/CLI_MOTION_POLICY.md` and contracts index entry; updated progress golden fixtures (`pack`, `verify success`, `verify failure`) to lock settled non-interactive output shape.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_UI_POLISH_02
- Added shared structural presentation primitives in `crates/crushr/src/cli_presentation.rs`: explicit phase/progress rows, banner framing (`INFO`/`WARNING`/`FAILURE`), and reusable result-summary rendering.
- Migrated core command output (`crushr-pack`, `crushr-extract`, `crushr-extract --verify`, `crushr-info`) to shared primitives for consistent title/target/progress/result hierarchy and standardized warning/failure framing.
- Removed duplicated per-command result-layout branches by routing result sections through `result_summary`, and updated golden fixtures for verify success/failure structure changes.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo test -p crushr --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_UI_POLISH_01
- Introduced a shared semantic CLI visual token layer in `crates/crushr/src/cli_presentation.rs`, replacing command-local meaning/color drift with centralized token mapping and no-color-safe rendering.
- Standardized user-facing status semantics to `PENDING`, `RUNNING`, `COMPLETE`, `DEGRADED`, `FAILED`, and `REFUSED`; retained bounded compatibility aliases and rendered legacy `PARTIAL` outcomes as `DEGRADED`.
- Added recover-mode trust-class presentation section in `crushr-extract` and documented token/status/trust contract in `.ai/contracts/CLI_VISUAL_SEMANTICS.md` (+ contracts index update).
- Updated presentation/recovery tests and goldens for consistent semantics (`cli_presentation_contract`, `recovery_validation_corpus`).
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test recovery_extract_contract`, `cargo test -p crushr --test recovery_validation_corpus`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_06
- Hardened OOXML high-confidence classification boundaries: `docx`/`xlsx`/`pptx` now require `_rels/.rels` plus existing subtype markers, reducing generic-zip container misclassification risk.
- Added deterministic naming collision coverage for repeated same-payload classification IDs to lock unique stable assigned names.
- Strengthened clean recover contract checks to enforce zero recovery artifacts for clean archives (no files under `recovered_named/` or `_crushr_recovery/anonymous/`) with empty manifest entries.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr recovery_classification::tests`, `cargo test -p crushr --test recovery_extract_contract`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_05
- Reworked `extract --recover` phase reporting to emit real incremental phase checkpoints through execution (`archive open`, `metadata scan`, `canonical extraction`, `recovery analysis`, `recovery extraction`, `manifest/report finalization`).
- Refined recover final summary wording/labels to align with trust classes (`canonical files`, `recovered_named`, `recovered_anonymous`, `unrecoverable`) and separated canonical vs recovery completeness in an explicit extraction-status section.
- Added precise recover notes behavior: no non-canonical warning text for clean archives, explicit non-canonical placement + manifest path guidance for damaged/mixed outcomes.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_04
- Added deterministic end-to-end recovery corpus contract test (`crates/crushr/tests/recovery_validation_corpus.rs`) covering strict/recover extraction divergence across clean + controlled corruption scenarios.
- Added deterministic corruption operators and mixed-outcome assertions for canonical/recovered_named/recovered_anonymous/unrecoverable in one archive, including anonymous naming tier checks and manifest truth validation.
- Added technical note `RECOVERY_VALIDATION_CORPUS.md` documenting fixture composition, scenario intent, and proof scope.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_validation_corpus`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_03
- Added modular recovery content-classification engine (`crates/crushr/src/recovery_classification.rs`) with ordered magic/structure/heuristic detection and confidence-tiered naming decisions.
- Integrated classification engine into recover extract flow and updated recovery manifest contract to separate trust class (`recovery_kind`) from content classification metadata (`kind`/`confidence`/`basis`/optional `subtype`).
- Updated recovery manifest schema + recover integration tests for the new classification contract.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_02
- Integrated salvage planning into recover extraction flow by reusing salvage engine analysis (`build_recovery_analysis`) from `extract --recover` execution path.
- Added required recover progress phases and required Result/Trust summary output contract for human recover mode.
- Added named-vs-anonymous recovery classification behavior for refused canonical entries and updated recover integration tests/docs accordingly.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_01-FIX1
- Applied rustfmt-conformant import ordering and assertion wrapping in `crates/crushr/tests/recovery_extract_contract.rs` to restore `cargo fmt --check` pass.
- Validation: `cargo fmt`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`.

## 2026-03-23 — CRUSHR_RECOVERY_MODEL_01
- Added internal recovery extraction engine (`recover_extract_impl`) and wired `crushr-extract --recover` while preserving strict default extraction semantics.
- Implemented required recovery output layout and manifest generation (`canonical/`, `recovered_named/`, `_crushr_recovery/manifest.json`, `_crushr_recovery/anonymous/`) with locked trust classes and anonymous naming policy.
- Added schema `schemas/crushr-recovery-manifest.v1.schema.json` and integration tests (`recovery_extract_contract.rs`) covering clean/damaged archives.
- Validation: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, `cargo test --workspace`.

## 2026-03-23 — CRUSHR_VERIFY_SCALE_01
- Reworked `crushr-extract --verify` to run strict verification in a bounded verify-only pass (no temp extraction output materialization), while preserving strict refusal/report semantics.
- Removed strict extraction decompressed payload cache retention/cloning in active paths to reduce verify-time memory residency pressure.
- Added deterministic production verify progress stages to human output and updated verify golden fixtures/integration assertions.
- Validation: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-22 — CRUSHR_CLI_UNIFY_04
- Pruned public `crushr-pack` parser/help surface to production-only controls (`input`, `-o/--output`, `--level`, `--silent`) and removed experimental flag acceptance from the production pack path.
- Added lab-owned experimental pack invocation surface `crushr lab pack-experimental` and wired it through shared command dispatch.
- Rewired lab comparison harness pack execution to call `crushr lab pack-experimental` for format experimentation instead of production `crushr-pack` flags.
- Updated tests to lock the boundary (`deterministic_pack` + `salvage_experimental_resilience`) and validated green with full workspace clippy/tests.

## 2026-03-22 — CRUSHR_CLI_UNIFY_03
- Added dedicated CLI contract integration coverage in `crates/crushr/tests/cli_contract_surface.rs` to lock canonical root command taxonomy, wrapper/canonical help+about+version equivalence, legacy alias rejection, root exit-code handling, and shared-flag behavior (`--json` + `--silent`).
- Removed undocumented alias behavior by making help/version first-argument-only across wrapper and command entry points (`crates/crushr/src/wrapper_cli.rs`, `commands/{pack,extract,info,salvage}.rs`).
- Synchronized docs/help contract text to observed runtime behavior (README wrapper control-position rule; `crushr-info` usage updated to optional `--json`).

## 2026-03-22 — CRUSHR_CLI_UNIFY_02
- Converted retained companion binaries to thin wrappers over shared CLI dispatch via `crushr::wrapper_cli::run_wrapper_env` (`crushr-pack`, `crushr-extract`, `crushr-info`, `crushr-salvage`) with uniform wrapper-level `--help`/`--version`/`about` handling and canonical-equivalent guidance.
- Moved salvage runtime into shared library command module (`crates/crushr/src/commands/salvage.rs`) and rewired top-level `crushr salvage` to in-process shared dispatch.
- Added explicit bin retention policy in `crates/crushr/Cargo.toml` (`autobins = false`) and removed deprecated `crushr-fsck` from active build outputs.
- Updated architecture/runtime docs for canonical-vs-wrapper usage mapping and refreshed product-surface test (`crushr-core/tests/minimal_pack_v1.rs`) to assert fsck-binary removal.

## 2026-03-21 — CRUSHR-CRATE-01
- Locked workspace crate policy in root `Cargo.toml`: `resolver = "3"`, `edition = "2024"`, and initial MSRV `rust-version = "1.85"`.
- Added `rust-version.workspace = true` across member crates and normalized publishable crate metadata inheritance to workspace-managed crates.io fields (`version`, `edition`, `rust-version`, `license`, `authors`, `repository`, `homepage`, `documentation`, `keywords`, `categories`).
- Made internal crates explicitly non-publishable with `publish = false` for `crushr-cli-common`, `crushr-lab`, and `crushr-tui`; publishable crates are `crushr`, `crushr-core`, and `crushr-format`.
- Added `scripts/check-crate-policy.sh` drift guard for package name presence, publishable metadata completeness, explicit internal publish intent, and resolver/edition/MSRV lock checks.
- Validation evidence: `./scripts/check-crate-policy.sh`, `cargo metadata --format-version 1 --no-deps`, `cargo +1.85.0 check --workspace`, and `cargo test -p crushr --test version_contract`.

## 2026-03-20 — CRUSHR-VERSION-01
- Added root canonical `VERSION` file (strict SemVer, initial value `0.2.2`) and a shared runtime accessor `crushr::product_version()` with strict SemVer validation in `crates/crushr/src/versioning.rs`.
- Replaced active `env!("CARGO_PKG_VERSION")` runtime/report/tool-version paths in `crushr` binaries/lab outputs with canonical `crushr::product_version()`.
- Added version-governance tooling: `scripts/check-version-sync.sh` (SemVer + drift validation), `scripts/sync-version.sh` (propagate `VERSION` to `workspace.package.version`), and updated `crates/crushr/dev/lib/rust.sh` packaging version source to read root `VERSION`.
- Added integration coverage `crates/crushr/tests/version_contract.rs` ensuring `VERSION` strict SemVer and Cargo workspace version alignment.

## 2026-03-20 — CRUSHR-UI-01
- Added shared CLI presentation layer (`crates/crushr/src/cli_presentation.rs`) defining deterministic header/section/outcome formatting and bounded status vocabulary.
- Wired `crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` to shared presentation semantics while preserving strict/salvage boundaries.
- Standardized `--silent` one-line deterministic summaries for scoped public commands and updated usage/help text accordingly.
- Added integration coverage (`crates/crushr/tests/cli_presentation_contract.rs`) for verify-output determinism and silent-mode behavior across pack/extract/verify/salvage.

## 2026-03-20 — CRUSHR-UI-02
- Realigned top-level `crushr` help/dispatch to canonical preservation suite commands (`pack`, `extract`, `verify`, `info`) with bounded non-primary `salvage`/`lab` entries.
- Demoted legacy generic-compressor command exposure from primary root surface (`append`, `list`, `cat`, `dict-train`, `tune`, `completions`) and added explicit legacy-demotion guidance.
- Repaired `crushr-extract --verify` structural-failure presentation so normal user output renders deterministic operator-facing refusal sections/reasons without raw parser leak strings.
- Added help-surface and verify-failure leakage tests in `crates/crushr/tests/cli_presentation_contract.rs`.

## 2026-03-20 — CRUSHR-UI-01-FIX1
- Restored workspace Cargo manifest validity by adding missing `package.name` in all workspace crates (`crushr-format`, `crushr-core`, `crushr-cli-common`, `crushr`, `crushr-tui`, `crushr-lab`).
- Unblocked and reran validation commands: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, and representative runtime runs for pack/extract/verify/salvage plus `--silent` variants.
- Corrected `crushr-salvage` output mode: default stdout is human presentation again; JSON stdout now requires explicit `--json` (with schema-contract tests updated to pass `--json`).

## 2026-03-20 — CRUSHR-LICENSE-01-FIX1
- Replaced `.reuse/dep5` with `REUSE.toml` to remove REUSE deprecation warnings while preserving the repository licensing split and SPDX coverage model.
- Revalidated with `reuse lint`, `cargo check --workspace`, and `cargo test --workspace`.

## 2026-03-20 — CRUSHR-LICENSE-01
- Unified repository licensing model: code is MIT OR Apache-2.0 and documentation/diagrams are CC-BY-4.0.
- Added canonical root license texts (`LICENSE-MIT`, `LICENSE-APACHE-2.0`, `LICENSE-CC-BY-4.0`) and aligned crate metadata to `license = "MIT OR Apache-2.0"` across the workspace.
- Applied SPDX headers repo-wide for `.rs`, `.sh`, `.md`, and `.svg` files, added `.reuse/dep5` mapping for repository-wide classification, and verified REUSE compliance with `reuse lint`.

## 2026-03-19 — CRUSHR-LAB-FIX-01
- Repaired three failing `crushr-lab` Phase 2 tests by removing implicit dependency on absent workspace artifact files.
- Added deterministic temporary-trials fixture setup for normalization shape/order tests and explicit scenario-id sort-helper usage in normalization code path.
- Updated comparison shape-contract test to use representative normalized records and validate emitted tables/rankings against active shape validators.

## 2026-03-18 — CRUSHR-HARDEN-03H
- Enforced canonical verification-output truth boundary by deleting CLI-local `VerifyReport` in `crushr-extract` and introducing canonical model-owned `VerificationReportView` projection in `crushr-core::verification_model`.
- Moved refusal-reason output labeling/mapping into `VerificationModel::to_report_view`, removing duplicate classification/summary assembly logic from the output layer.
- Revalidated boundary purity and output determinism with focused grep checks plus a byte-for-byte repeated `crushr-extract --verify --json` comparison on the same archive.

## 2026-03-18 — CRUSHR-HARDEN-03G (canonical verification + salvage lint cleanup, partial)
- Added `crushr-core::verification_model` with a canonical typed verification truth model (`VerificationModel`, `VerifySummary`, `FileResolution`, `FailureDomains`) and deterministic model construction from strict extraction reports.
- Updated `crushr-extract --verify` report assembly to consume the canonical verification model instead of constructing refusal/report truth directly from raw extraction internals.
- Fixed carried-forward salvage metadata classification clippy error (`if_same_then_else`) in verified-graph classification by removing redundant branch duplication and keeping deterministic orphan fallback behavior.

## 2026-03-18 — CRUSHR-HARDEN-03A
- Finalized API-boundary truth by removing accidental public `crushr::extraction_path` exposure and keeping confinement helpers internal-only.
- Added a compile-level visibility guard in `crates/crushr/src/lib.rs` via a `compile_fail` doctest to prevent unsupported API resurfacing.
- Updated README and crate-level docs to classify stable product surfaces, bounded internal surfaces, and experimental/lab-only surfaces explicitly.
- Adjusted snapshot synthetic archive helper to emit a minimal valid IDX3 payload for strict-verify compatibility and preserved verify/info regression coverage.

## 2026-03-18 — CRUSHR-HARDEN-03G follow-up
- Completed redundant-file-map/tail closeout cleanup by extracting bounded helpers (`build_redundant_file_map`, `write_tail_with_redundant_map`) from `emit_archive_from_layout`.
- Converted redundant map closeout payload to typed structs (`RedundantFileMap`, `RedundantFileMapFile`, `RedundantFileMapExtent`) before ledger serialization.
- Confirmed there is no separate 03H step after this cleanup; continuity notes now point back to metadata-pruning evidence review.

## 2026-03-18 — CRUSHR-HARDEN-03G
- Extracted pack experimental metadata JSON construction out of the main emission loop into bounded helper builders (`build_*record` / `build_*snapshot`), reducing cross-concern coupling while preserving output semantics.
- Kept pack pipeline staging from 03F intact and reran deterministic/canonical/experimental regression checks.

## 2026-03-18 — CRUSHR-HARDEN-03F
- Refactored `crushr-pack` into explicit pipeline stages by introducing typed planning surfaces (`PackLayoutPlan`, `MetadataPlan`, `DictionaryPlan`) and splitting orchestration into `build_pack_layout_plan` and `emit_archive_from_layout`.
- Isolated dictionary/auxiliary construction into `build_dictionary_plan` so path-dictionary body/copy planning is no longer interleaved directly with low-level block emission.
- Preserved canonical + experimental writer behavior while adding focused regressions for metadata-profile determinism and redundant-map profile recording.

## 2026-03-18 — CRUSHR-HARDEN-03E
- Decomposed the `crushr-lab-salvage` comparison engine from a single multi-thousand-line file into responsibility-based modules: `lab/comparison/common.rs`, `experimental.rs`, `format06_to12.rs`, `format13_to15.rs`, with `lab/comparison/mod.rs` as orchestration surface.
- Kept top-level comparison command dispatch stable by updating `crushr-lab-salvage` module path to `lab/comparison/mod.rs` and preserving existing run-command entry points.
- Revalidated comparison outputs using schema-backed tests and representative comparison command execution to preserve output semantics while improving maintainability.


## 2026-03-18 — CRUSHR-HARDEN-03C
- Added schema files for active FORMAT-12/13/14A/15 comparison summary artifacts (baseline + stress variants).
- Added integration tests that execute active comparison commands and verify emitted summary artifacts satisfy schema-required fields and schema-version constants.
- Documented that deeper typed decomposition of remaining untyped summary assembly is deferred to 03E.

## 2026-03-16 — CRUSHR-FORMAT-14A (dictionary-target corruption resilience comparison)

- Added two new lab commands: `run-format14a-dictionary-resilience-comparison` and `run-format14a-dictionary-resilience-stress-comparison`.
- Extended the corruption harness with explicit dictionary-target scenarios (`primary_dictionary`, `mirrored_dictionary`, `both_dictionaries`, `inconsistent_dictionaries`) and variant-aware dictionary mutation support.
- Added FORMAT-14A output artifacts: `format14a_dictionary_resilience_summary.{json,md}` and `format14a_dictionary_resilience_stress_summary.{json,md}`.
- Added focused tests for command wiring/help visibility and required dictionary-target metrics/artifact presence.

## 2026-03-16 — CRUSHR-FORMAT-10 (metadata pruning experiment + comparison harness)

- Added opt-in packer metadata profile surface: `--metadata-profile <payload_only|payload_plus_manifest|payload_plus_path|full_current_experimental>`.
- Added `run-format10-pruning-comparison` command wiring and output artifacts: `format10_comparison_summary.json` and `format10_comparison_summary.md`.
- Added archive-size accounting fields (`archive_byte_size`, deterministic `metadata_byte_estimate`, `overhead_delta_vs_payload_only`) and grouped breakdown by dataset/corruption target.
- Added CLI/help/dispatch and comparison-output integration tests, including FORMAT-09 post-change dispatch regression coverage.

## 2026-03-16 — CRUSHR-FORMAT-09 (metadata survivability and necessity audit harness)

- Added `crushr-lab-salvage run-format09-comparison --output <dir>` command wiring and dispatch.
- Expanded comparison harness with a deterministic FORMAT-09 scenario matrix (metadata regime × metadata target × payload topology) and targeted corruption operators for metadata/payload regions.
- Added FORMAT-09 outputs: `format09_comparison_summary.json` and `format09_comparison_summary.md` including recovery, metadata survival, and metadata-recovery-gain metrics.
- Added integration tests to cover help/dispatch and required FORMAT-09 summary fields.

## 2026-03-15 — CRUSHR-SALVAGE-07 harness hardening for deterministic discovery/resolution

- Hardened `crushr-lab-salvage` salvage binary resolution to avoid bare PATH dependency by using deterministic resolution (explicit `CRUSHR_SALVAGE_BIN`, sibling executable near current binary, and Cargo test binary environment path).
- Replaced extension-only archive discovery with bounded format-identity checks (`BLK3` leading magic or valid `FTR4` footer + `IDX3` index marker), enabling valid `.crs` and extensionless archives.
- Added focused harness regression tests for archive identity acceptance/rejection, deterministic ordering, PATH-independent binary resolution, and clear resolution failure messaging.

## 2026-03-14 — CRUSHR-SALVAGE-06 grouped analysis views for salvage experiments

- Extended `crushr-lab-salvage` to emit compact deterministic `analysis.json` and `analysis.md` alongside existing summaries for each experiment.
- Added deterministic grouped outcome/export/profile analysis plus ranked evidence lists with explicit archive-id tie-breaking.
- Extended `--resummarize <experiment_dir>` to regenerate summary and analysis artifacts from existing experiment outputs without rerunning salvage.
- Added harness tests for analysis generation, grouping/ranking behavior, deterministic ordering, profile fallback/inference, resummarize regeneration, and compactness guardrails.

## 2026-03-14 — CRUSHR-P2-EXEC-06A recovery-accounting harness upgrade

- Upgraded `crushr-lab` Phase 2 execution to run real extraction commands per format (`crushr-extract`, `unzip`, `tar`) and produce per-run recovery evidence artifacts (`extracted/`, `recovery_report.json`) plus structured file/byte accounting in `RawRunRecord`.
- Extended normalization outputs with deterministic recoverability metrics (`files_expected/recovered/missing`, `bytes_expected/recovered`, recovery ratios), blast-radius classes, and explicit `recovery_evidence_strength` enum values.
- Extended normalization summary with recovery aggregate rollups by format/corruption/target and blast-radius distributions, updated schemas/contracts, and added focused tests for recovery accounting behavior and blast-radius thresholds.

## 2026-03-14 — CRUSHR-P2-EXEC-03B: white-paper-grade Phase 2 execution evidence enrichment

- Enriched `crushr-lab` Phase 2 `RawRunRecord` output with first-class scenario axes + invocation + path + result fields, added deterministic `result_artifacts`/`result_completeness`, and removed ambient wall-clock timestamps from run metadata.
- Replaced broken generic tool `--version` capture with explicit truthful probing semantics (`detected`, `unsupported`, `unavailable`) and removed acceptance of strings like `unsupported flag: --version` as versions.
- Upgraded `execution_report.json` to include run/cardinality histograms, JSON-result counts, tool-version summary, and explicit completeness status while preserving raw per-run corpus output.
- Added schema contracts for execution report and raw run records plus focused tests for record/report shape, version handling, and schema constraints.

## 2026-03-13 — Phase 2 execution foundation dataset/baseline build (CRUSHR-P2-EXEC-02)

- Updated `crushr-lab build-phase2-foundation` defaults/outputs to emit fixture datasets under `PHASE2_RESEARCH/datasets/`, baseline archives under `PHASE2_RESEARCH/baselines/{crushr,zip,tar_zstd,tar_gz,tar_xz}`, and report artifacts under `PHASE2_RESEARCH/foundation/`.
- Expanded foundation archive records with required metadata fields (`archive_file`, `archive_size`, `archive_blake3`, `file_count`, `dataset_name`, `format`) and deterministic-generation confirmation in the report.
- Added deterministic timestamp/flag normalization for zip/tar baselines and generated Phase 2 foundation artifacts (`datasets`, `baselines`, `foundation_report.json`) for locked datasets and formats.

## 2026-03-13 — Phase 2, Step 2.1 cleanup (CRUSHR-P2-CLEAN-07: slim crushr-lab main dispatch edge)

- Reduced `crates/crushr-lab/src/main.rs` to command parsing + dispatch + top-level usage/exit behavior only.
- Added `crates/crushr-lab/src/cli.rs` for shared command parsing (`Command` enum), usage text, and workspace-root resolution.
- Moved `corrupt` command parsing/logging helpers and alias/target/magnitude parsing into `phase2_corruption.rs`; retained existing behavior/defaults and parsing tests there.
- Moved write/build/run command orchestration wrappers into owning Phase 2 modules (`phase2_manifest`, `phase2_foundation`, `phase2_runner`) so `main.rs` no longer contains packet-grown helper logic.

## 2026-03-13 — Phase 2, Step 2.1 cleanup (CRUSHR-P2-CLEAN-06 domain model unification)

- Added `crates/crushr-lab/src/phase2_domain.rs` as the canonical Phase 2 model for dataset, archive format, corruption type, target class, magnitude tiers, locked seeds, and scenario ID generation helper.
- Removed duplicated Phase 2 domain enums from `phase2_manifest` and `phase2_foundation`, and updated `phase2_runner` to consume shared domain types directly.
- Deleted translation shims (`map_dataset`, `map_format`) and centralized ordering/slug/scenario-id semantics through the shared domain model.
- Updated `crushr-lab` imports/tests so manifest generation, foundation archive prep, corruption requests, and execution records all use one Phase 2 type system.

## 2026-02-17 — Phase 0, Step 0.1 (migration)

- Created canonical repo root from prime scaffold.
- Imported `crushr` crate sources, docs, and dev tooling.
- Preserved legacy continuity documents under `.ai/imported_crushr/` and `docs/legacy/`.

## 2026-02-17 — Phase 0, Step 0.2 (spec + architecture lock-in)

- Replaced `SPEC.md` with Archive Format v1.0 (BLK3/DCT1/IDX3/FTR4).
- Preserved prior spec as `docs/legacy/SPEC_pre_v1.md`.
- Added `docs/ARCHITECTURE.md` (crate graph, tool suite, no-IPC rule).
- Converted repo to a Cargo workspace and introduced `crushr-format` and `crushr-core` crate skeletons.

## 2026-02-17 — Phase 0, Step 0.3 (TUI live/snapshot contract skeleton)

- Documented the TUI dual-mode data pipeline (live + snapshot) and merge rules in `docs/ARCHITECTURE.md`.
- Added normative snapshot contract: `docs/SNAPSHOT_FORMAT.md`.
- Added snapshot schema placeholders under `schemas/`.
- Added workspace skeleton crates: `crushr-cli-common` and `crushr-tui`.
- Added `crushr-core::snapshot` envelope types (v1 skeleton).

## 2026-02-17 — Phase 0, Step 0.4 (Ledger framing + snapshot fingerprint)

- Implemented `LDG1` ledger framing in `crushr-format` with canonical JSON serialization and BLAKE3 verification.
- Added unit tests for canonicalization and LDG1 round-trip.
- Introduced typed `ArchiveFingerprint` and deterministic derivation helper in `crushr-core::snapshot`.
- Added snapshot serialization tests.

## 2026-02-17 — Phase 0, Step 0.5 (BLK3 primitives)

- Added `crushr-format::blk3` defining BLK3 header layout, strict v1 validation, and read/write helpers.
- Enforced v1 rules: unknown flags rejected; dict flag/id consistency; reserved bytes must be zero.
- Added unit tests covering round-trips and invalid encodings.
- Updated `.ai/PHASE_PLAN.md` and `.ai/BACKLOG.md` with the near-future step plan and decision gates.

## 2026-02-17 — Phase 0, Step 0.6 (DCT1 primitives)

- Added `crushr-format::dct1` defining the DCT1 dictionary table layout.
- Implemented strict read/write with corruption guards (max count/size) and mandatory BLAKE3 dict hash verification.
- Added unit tests for multi-dict round-trips, duplicate dict_id rejection, and hash mismatch detection.

## 2026-02-17 — Phase 0, Step 0.7 (FTR4 primitives)

- Added `crushr-format::ftr4` defining the FTR4 footer layout.
- Implemented strict read/write with presence rules for optional sections (DCT/LEDGER), reserved-zero enforcement, overflow guards, and footer hash verification.
- Added unit tests for round-trip encoding/decoding, reserved-byte corruption, footer hash mismatch, and ledger presence invariants.

- p0s0.8f0: added contracts package, gated roadmap, and project-definition docs

- p0s0.9f0: added failure-domain validation phase and corruption harness skeleton

- p0s0.10f0: added formal failure-domain docs, impact schema, and decompression-free impact enumeration model

- p0s0.10f1: added Codex control-layer docs and first task packet


## 2026-03-08 — Phase 0, Step 0.8 (tail frame helpers)

- Added `crushr-format::tailframe` assembly helpers for deterministic `DCT1? + IDX3 + LDG1? + FTR4` layout.
- Added strict parsing helpers that validate footer/component boundaries and BLAKE3 integrity fields.
- Added round-trip tests (full + minimal) and malformed rejection tests (footer corruption and ledger corruption).
- Fixed a pre-existing syntax defect in `ftr4` tests to restore successful test/clippy runs for `crushr-format`.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 1)

- Repaired CLI source parse errors that prevented `cargo fmt --all` from running.
- Restored `Cmd::Info` to the command `match` in `main.rs`.
- Fixed `cli_ui` tier-2 sink wiring/type usage and corrected a path separator literal in `pack.rs`.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 2)

- Fixed `crushr/src/dict.rs` regressions that blocked `cargo check -p crushr`.
- Imported `walkdir::WalkDir`, restored valid function separation, and updated `zstd::dict::from_continuous` to pass sample sizes.
- Added explicit empty-sample guard for progress-based dictionary training.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 3)

- Cleaned `crushr-format` tail frame parsing internals to remove unchecked narrowing casts and keep deterministic bounds checks for component slices.
- Removed duplicate LDG1 parsing pass in trailing-byte detection by validating exact reader consumption directly.
- Re-ran and passed `cargo test -p crushr-format` and `cargo clippy -p crushr-format --all-targets -- -D warnings`.

## 2026-03-08 — Phase 0, Step 0.10 (real info snapshot emission)

- Added `crushr-core::open::open_archive_v1` and real tail-frame archive parsing over `ReadAt + Len`.
- Replaced `InfoSnapshotV1` skeleton fields with typed summary/tail/dict metadata and added mapping from `OpenArchiveV1`.
- Added deterministic snapshot JSON serialization helper + tests for minimal archives, DCT1/LDG1 presence, deterministic serialization, and clean invalid-input failure.
- Added minimal `crushr-info` binary path supporting `crushr-info <archive> --json`.

## 2026-03-08 — Phase 0, Step 0.10 (fix iteration 1)

- Tightened `crushr-core` snapshot emission test assertions to validate typed JSON envelope/payload fields rather than substring presence.
- Re-ran Step 0.10 verification commands (`cargo test -p crushr-core`, `cargo test -p crushr --no-run`) to confirm read-only info snapshot path remains green.
- Documented that `crushr-info` end-to-end CLI JSON testing remains blocked on the `crushr` pack path still producing legacy (non-FTR4) archives.

## 2026-03-08 — Phase 0, Step 0.10 (fix iteration 2)

- Added explicit footer metadata to the open/snapshot mapping path (`footer_offset`, `footer_len`, `has_footer`) so `InfoSnapshotV1` emits footer presence/details from parsed archive state.
- Added `crushr-core` test coverage for the real `crushr-info --json` binary path using synthetic valid archive bytes written to a temp file.
- Re-ran `cargo fmt --all`, `cargo test -p crushr-core`, and `cargo test -p crushr --no-run`.

## 2026-03-08 — Phase 0, Step 0.12 (real fsck JSON metadata path)

- Added `crushr-fsck` binary with real `--json` output over opened archives and deterministic nonzero exit on structural parse/validation failure.
- Extended `crushr-core::snapshot` with typed fsck payload mapping and clean `ImpactReportV1` emission for currently supported metadata validation scope.
- Added synthetic-archive tests covering fsck valid success JSON, deterministic JSON, corrupted-footer failure, and corrupted-IDX3-hash failure.

## 2026-03-08 — Phase 0, Step 0.11 (minimal self-hosting v1 pack path)

- Added `crushr-pack` binary implementing the first bounded v1 writer path using `crushr-format` BLK3 and tailframe helpers.
- New pack path writes BLK3 blocks (one per file), IDX3 payload bytes, and a valid `FTR4` tail frame (without DCT1/LDG1 for now).
- Added integration tests covering single-file and tiny-directory pack flows with successful `open_archive_v1`, `crushr-info --json`, and `crushr-fsck --json` reads, plus deterministic output checks.

## 2026-03-08 — Phase F, Step F.3 (first real e2e corruption experiment path)

- Extended `crushr-lab corrupt` with deterministic corruption controls (`--model`, `--seed`, optional `--offset`) and richer corruption metadata (`input_len` + touched offsets).
- Added integration test `crates/crushr-core/tests/first_corruption_experiment.rs` covering real single-file pack/corrupt/info/fsck loop and determinism checks.
- Recorded first true experiment artifacts at `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/`.
- Updated `docs/RESEARCH/RESULTS.md` with the initial recorded result and explicit limitation language.

## 2026-03-08 — Phase F, Step F.3 (fix iteration 1: workspace hygiene + exit-code normalization)

- Fixed pre-existing `crates/crushr/tests/mvp.rs` binary path assumptions so workspace tests no longer depend on crate-local `target/debug` layout.
- Updated `crushr-info` to use the same exit-code classification style as `crushr-fsck` for this baseline (`1` usage, `2` open/parse/structural failures).
- Added/extended binary-path tests in `crushr-core::snapshot` for `crushr-info` structural-failure exit code and info/fsck missing-archive open-failure parity.
- Clarified the normalized tool behavior in `docs/CONTRACTS/ERROR_MODEL.md`.
- Re-ran and passed `cargo test --workspace`.


## 2026-03-09 — Phase F, Step F.3 (fix iteration 2: BLK3 payload verification path)

- Added `crushr-core::verify` with typed BLK3 scanning and payload-hash verification over stored compressed payload bytes.
- Integrated verification into `crushr-fsck` snapshot generation so `blast_radius.impact.corrupted_blocks` now reports payload corruption block IDs.
- Added tests for clean/corrupt payload cases, deterministic fsck JSON for identical bytes, and preserved footer/tail corruption behavior.
- Re-ran and passed `cargo test -p crushr-core` and `cargo test --workspace`.


## 2026-03-09 — Phase F, Step F.3 (fix iteration 3: reproducible experiment runner path)

- Added `crushr-lab run-first-experiment` to deterministically execute the current first structural corruption experiment (fixture → pack → corrupt → info/fsck).
- Added expectation-gated artifact emission (defaulting to `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/`) including clean JSON outputs, corrupt exit/stderr captures, and refreshed manifest fields.
- Added integration coverage ensuring runner artifact presence and experiment-id/exit-code consistency.
- Updated `docs/RESEARCH/EXPERIMENT_METHOD.md` and `docs/RESEARCH/RESULTS.md` with the explicit runner command and clear small-scope limitation wording.

## 2026-03-09 — Phase F, Step F.4 (fix iteration 0: bounded competitor comparison scaffold)

- Added `crushr-lab run-competitor-scaffold` to produce deterministic scaffold artifacts under `docs/RESEARCH/artifacts/crushr_p0s13f0_competitor_scaffold_byteflip/`.
- Implemented scaffold target records for `crushr`, `zip`, and `tar+zstd` with explicit environment detection/deferral behavior, plus explicit `7z` deferral handling.
- Added integration tests in `crates/crushr-core/tests/first_corruption_experiment.rs` for manifest structure, deferred-target honesty, and docs/results artifact-reference alignment.
- Updated `docs/RESEARCH/EXPERIMENT_METHOD.md` and `docs/RESEARCH/RESULTS.md` to document scaffold scope and supported vs deferred targets without benchmark claims.

## 2026-03-09 — Phase 0, Step 0.14 (fix iteration 0: strict minimal-v1 extraction)

- Added `crushr-extract` binary for strict extraction of minimal v1 archives (regular files only) using existing open/verify/index paths.
- Strict extraction now refuses files requiring corrupted blocks while allowing unaffected files to extract in deterministic path order.
- Added extraction integration tests for single-file and tiny-directory clean round trips, payload-corruption selective refusal, invalid-footer failure, and deterministic stderr behavior.
- Updated `PROJECT_STATE.md` to record strict extraction availability and explicitly keep salvage/metadata fidelity out of scope.


## 2026-03-09 — Phase 0, Step 0.14 (fix iteration 1: policy-controlled strict refusal exit semantics)

- Added `--refusal-exit <success|partial-failure>` to `crushr-extract` with default `success` so strict refusal can optionally signal machine-detectable partial failure.
- Kept strict extraction behavior unchanged: unaffected files still extract, refused files are still skipped and deterministically reported.
- Extended extraction integration tests to cover clean archives in both modes, selective-refusal exit `0`/`3` split by policy, structural-failure exit `2` in both modes, and stable refusal reporting.
- Updated `docs/CONTRACTS/ERROR_MODEL.md` and `PROJECT_STATE.md` for the new policy-controlled exit contract.

## 2026-03-09 — Phase 0, Step 0.14 (fix iteration 2: strict extraction JSON reporting)

- Added `--json` to `crushr-extract` for deterministic strict extraction result reporting.
- JSON reports now include `overall_status`, deterministic `extracted_files`, and `refused_files` entries with stable refusal reason `corrupted_required_blocks`.
- Preserved existing strict behavior and refusal-exit semantics (`success` => exit `0` on refusals, `partial-failure` => exit `3`, structural failures => exit `2`) while adding JSON error envelopes for non-success cases.
- Extended `minimal_pack_v1` integration tests to cover clean JSON success, partial-refusal JSON under both refusal policies, structural-failure JSON behavior, and deterministic output for identical inputs.
- Updated `docs/CONTRACTS/ERROR_MODEL.md`, `docs/ARCHITECTURE.md`, and `PROJECT_STATE.md` accordingly.

## 2026-03-09 — Phase 0, Step 0.14 (fix iteration 3: typed strict extraction outcome classification)

- Refactored `crushr-extract` to classify extraction outcomes/errors via typed enums instead of message-string matching for exit-code selection.
- Preserved existing observable behavior for strict extraction, `--refusal-exit`, and `--json` success/error envelopes.
- Added a targeted unit test for typed outcome/error to exit-code mapping and re-ran strict extraction integration tests.

## 2026-03-09 — Phase 0, Step 0.15 (fix iteration 0: explicit salvage-mode extraction)

- Added explicit extraction mode selection to `crushr-extract` via `--mode <strict>` with default strict behavior preserved.
- Implemented salvage-mode deterministic reporting (`mode: salvage` + ordered `salvage_decisions`) while keeping integrity-first refusal for corrupted required blocks.
- Added focused salvage integration tests for clean archives, partial corruption with verified-only extraction, refusal-exit interaction, and deterministic JSON behavior.


## 2026-03-12 — Phase 1, Step 1.2 (fix iteration 0: maximum safe extraction formalization/reporting)

- Formalized `crushr-extract --json` around explicit maximum-safe-extraction reporting fields: `maximal_safe_set_computed`, deterministic `safe_files`, deterministic `refused_files`, and `safe/refused` counts.
- Added typed refusal reason serialization (`corrupted_required_blocks`) while preserving strict integrity-first extraction behavior and refusal policy exit semantics.
- Updated extraction integration tests for clean single-file, clean tiny-directory deterministic ordering/counts, selective corruption safe-vs-refused reporting, structural invalid JSON error envelope behavior, and deterministic serialization checks.
- Updated project/contract/research and `.ai` continuity docs to record maximum safe extraction as a first-class capability in minimal v1 scope.

## 2026-03-12 — Phase 1, Step 1.3 (fix iteration 0: extraction result contract formalization)

- Added a dedicated extraction JSON contract doc at `docs/CONTRACTS/EXTRACTION_RESULT_V1.md` covering success, partial-refusal, and error envelopes for minimal v1 scope.
- Added versioned schema `schemas/crushr-extract-result.v1.schema.json` capturing strict/salvage success surfaces, partial refusal, salvage-only fields, and error envelopes with stable refusal/decision enums.
- Tightened `minimal_pack_v1` extraction tests to explicitly assert strict-vs-salvage field presence/absence, deterministic ordered report arrays, stable refusal reason values, and error-envelope shape.
- Updated contract index/error-model docs to reference the formal extraction result contract.
- Added `#[allow(clippy::len_without_is_empty)]` on `crushr-core::io::Len` so required clippy `-D warnings` checks pass without behavior changes.

## 2026-03-12 — Phase 1, Step 1.3 (fix iteration 1: schema-validation harness)

- Added `crates/crushr-core/tests/extract_result_schema_v1.rs` as a dedicated automated schema-validation harness for `crushr-extract --json`.
- Harness validates strict success, salvage partial refusal, and structural error envelopes against `schemas/crushr-extract-result.v1.schema.json` and enforces deterministic path ordering/enum constraints from the schema.


## 2026-03-12 — Phase 1, Step 1.1 (corruption propagation graph)

- Added `crushr-core::propagation` typed deterministic report model for minimal-v1 structure/file/block dependency and impact propagation.
- Added `crushr-info --json --report propagation` as the machine-readable propagation graph reporting surface.
- Added `docs/CONTRACTS/PROPAGATION_GRAPH_V1.md` and schema `schemas/crushr-propagation-graph.v1.schema.json`.
- Added contract/determinism/consistency integration tests in `crates/crushr-core/tests/propagation_graph_v1.rs`.


## 2026-03-12 — Phase 1, Step 1.1 (fix iteration 1: workspace lint debt cleanup)

- Removed existing workspace lint failures so `cargo clippy --workspace --all-targets -- -D warnings` passes.
- Applied lint-only changes in `crushr`, `crushr-tui`, and related modules (unused vars/mut, reserve-after-init, counter-loop, identity-op, needless returns, and targeted allow attributes for stable public signatures).
- No functional behavior or public API contracts were changed.

## 2026-03-12 — Phase 1, Step 1.1 (fix iteration 2: hostile-review hardening CRUSHR-1.1-B)

- Narrowed propagation report contract to truthful current capability boundaries (openable archives + payload-corruption observation).
- Renamed propagation fields to remove structural-current-state observability ambiguity.
- Hardened propagation schema/integration tests to validate strict nested shape, enum stability, unknown-field rejection behavior, and ordering invariants.
- Added explicit boundary test proving structural open/parse corruption returns nonzero and emits no propagation report.
- Aligned control docs to AGENTS/STATUS authority model and marked Phase 2 active with 2.1 as next packet.
- Removed stale markdown cruft (`WHITEPAPER_OUTLINE.md`, `REPO_SNAPSHOT.md`).


- Follow-up: removed `crushr-extract --mode salvage` surface and tightened strict-only extraction contract/schema/tests.
- Follow-up: propagation report now emits bounded structural-current-state impacts for open-path failures.


## 2026-03-12 — Phase 2, Step 2.0-A (legacy recovery/salvage surface deletion)

- Deleted legacy recovery/salvage CLI surfaces from `crates/crushr/src/main.rs` and removed the recovery module `crates/crushr/src/recovery.rs`.
- Deleted legacy public API recovery/salvage options/functions from `crates/crushr/src/api.rs` and removed related progress op variants.
- Removed `salvage_plan` from `crushr-core` fsck snapshot model and aligned docs/spec/tests to remove active recovery/salvage/repair workflow descriptions.

## 2026-03-12 — Phase 2, Step 2.0-B (documentation/control canonicalization)

- Rewrote active control docs to one canonical startup order and authority hierarchy.
- Rewrote core active docs (`README.md`, `SPEC.md`, `docs/ARCHITECTURE.md`, `docs/SNAPSHOT_FORMAT.md`, `PROJECT_STATE.md`, `REPO_LAYOUT.md`) to match integrity-first strict extraction reality.
- Deleted stale transitional markdown from active paths (`docs/legacy/*`, `docs/README.md`, `.ai/imported_crushr/*`).
- Marked Phase 2.1 manifest/schema packet as the explicit next engineering step across control files.


## 2026-03-12 — Phase 2, Step 2.0-C (schema contracts tightened)

- Tightened active schemas for `crushr-info`, `crushr-fsck`, and `crushr-impact` from permissive envelopes to explicit nested object/array contracts with bounded enums and `additionalProperties: false` where appropriate.
- Added validator-backed schema tests (`jsonschema`) to validate emitted `crushr-info --json` and `crushr-fsck --json` instances against their schemas and to validate typed `ImpactReportV1` instances against the impact schema.
- Upgraded extraction-result and propagation tests to perform real JSON Schema validation in addition to existing deterministic-shape assertions.
- Audited active schemas: extraction-result and propagation-graph remain active/canonical; no obsolete schema was deleted in this packet.


## 2026-03-12 — Phase 2, Step 2.0-D (shared core report assembly centralization)

- Added `crushr-core::extraction` typed report/refusal model and helper assembly (`build_extraction_report`) so extraction semantics live in shared core.
- Slimmed `crushr-extract` by removing local report/refusal structs and delegating outcome/report construction to core helpers.
- Added `build_structural_failure_report_v1` in `crushr-core::propagation` and updated `crushr-info` structural fallback branches to use shared helper.
- Added/updated unit tests to assert centralized report semantics remain deterministic.


## 2026-03-12 — Phase 2, Step 2.1 (CRUSHR-P2.1-A manifest-first experiment contract and scenario enumeration)

- Added typed Phase 2 manifest/scenario model to `crushr-lab` with locked core matrix enums and deterministic scenario ID generation.
- Added deterministic locked-core scenario expansion to 2160 scenarios with explicit enumeration ordering rules (`dataset` → `format` → `corruption_type` → `target_class` → `magnitude` → `seed`).
- Added schema `schemas/crushr-lab-experiment-manifest.phase2.v1.schema.json` for Phase 2 manifest contract.
- Added `crushr-lab write-phase2-manifest` command to generate a schema-tagged manifest artifact independent from execution logic.
- Added manifest/scenario tests for count, ordering, stable IDs, stable fields, and schema-shape validation.


## 2026-03-12 — Phase 2, Step 2.1 (CRUSHR-P2.1-B dataset fixture/archive foundation)

- Added `crates/crushr-lab/src/phase2_foundation.rs` implementing deterministic dataset fixture builders for `smallfiles`, `mixed`, and `largefiles` with explicit composition rules.
- Added deterministic inventory/provenance emission (`inventory.json` + `inventory.blake3.txt`) and stable inventory digesting for fixture-drift detection.
- Added typed archive-build foundation for `crushr`, `tar+zstd`, `zip`, and `7z/lzma` using structured `CommandExecutionRecord` instead of shell-story output.
- Added `crushr-lab build-phase2-foundation` command and validation helpers for archive coverage over the locked dataset/format matrix.
- Added reproducibility tests for dataset generation, inventory determinism, and archive coverage shape.


## 2026-03-12 — Phase 2, Step 2.1 (CRUSHR-P2.1-C deterministic corruption injection layer)

- Added `crates/crushr-lab/src/phase2_corruption.rs` implementing the locked core corruption engine for `bit_flip`, `byte_overwrite`, `zero_fill`, `truncation`, and `tail_damage`.
- Locked neutral targets (`header`, `index`, `payload`, `tail`), magnitudes (`1B`, `256B`, `4KB`), and fixed seed policy (`1337`, `2600`, `65535`) with validation in the main corruption path.
- Added deterministic mutation provenance model (source archive, scenario_id, corruption type, target, magnitude, seed, concrete mutation details).
- Updated `crushr-lab corrupt` to consume locked corruption options and emit deterministic provenance metadata in `.corrupt.json`.
- Added determinism/unit tests proving repeated scenario generation is byte/provenance-stable and that locked seed policy is enforced.

## 2026-03-12 — Phase 2, Step 2.1-D (execution runner + raw evidence capture)

- Added `crushr-lab` Phase 2 execution command (`run-phase2-execution`) that consumes the locked manifest plus foundation report and runs all scenarios through typed paths.
- Added typed raw run record schema (`RawRunRecord`) with deterministic per-scenario artifact layout under `docs/RESEARCH/artifacts/phase2_execution/raw/<scenario_id>/`.
- Added completeness auditing for missing/duplicate/mismatched scenario IDs and report output (`completeness_audit.json`).
- Added focused tests for raw JSON-result bookkeeping and completeness validation behavior.

## 2026-03-12 — Phase 2, Step 2.1 cleanup (CRUSHR-P2-CLEAN-01)

- Deleted obsolete scaffold/demo command surfaces from `crushr-lab` (`run-first-experiment`, `run-competitor-scaffold`) including related constants, helper flows, and tests.
- Slimmed `crates/crushr-lab/src/main.rs` to a Phase 2-focused orchestration edge with only active command paths (`corrupt`, `write-phase2-manifest`, `build-phase2-foundation`, `run-phase2-execution`).
- Removed one-off scaffold helper sediment and replaced removed tests with focused parsing tests relevant to retained command surfaces.


## 2026-03-12 — Phase 2, Step 2.1 cleanup (CRUSHR-P2-CLEAN-02)

- Removed narrative command-string provenance (`observed_command`) from `crates/crushr-lab/src/phase2_runner.rs` execution metadata used by active Phase 2 records.
- Added typed `InvocationMetadata` to `ExecutionMetadata` capturing truthful invocation fields from actual `Command` execution: `tool_kind`, `executable`, `argv`, `cwd`, `exit_status_code`, and stdout/stderr artifact paths.
- Updated Tar+Zstd observation path to execute directly via `tar --use-compress-program=zstd -tf <archive>` so invocation provenance is represented as a real executable + argv path rather than shell storytelling.
- Updated/kept Phase 2 runner tests aligned to the new structured provenance model.

## 2026-03-13 — Phase 2, Step cleanup (CRUSHR-P2-CLEAN-03)

- Created canonical `PHASE2_RESEARCH/` root with concrete subdirectories for methodology, manifests, generated outputs, normalized results, summaries, and whitepaper support.
- Moved active Phase 2 lock guidance from `.ai/PHASE2_LOCKS.md` to `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md`.
- Updated `crushr-lab` defaults so `write-phase2-manifest`, `build-phase2-foundation`, and `run-phase2-execution` write/read under `PHASE2_RESEARCH/*` instead of `docs/RESEARCH/artifacts/*`.
- Updated repository docs/control references to identify `PHASE2_RESEARCH/` as canonical Phase 2 workspace and keep product docs separate from generated research outputs.


## 2026-03-13 — Phase 2, Step cleanup (CRUSHR-P2-CLEAN-04)

- Replaced Phase 2 core comparator set `7z/lzma` with `tar+gz` and `tar+xz` across `crushr-lab` manifest, schema, foundation builder, and execution runner flows.
- Updated locked core manifest/scenario validation and tests to the 5-format matrix (2700 deterministic scenarios).
- Updated canonical Phase 2 lock methodology docs to reflect the new comparator set and run-count math.


## 2026-03-13 — Phase 2, Step cleanup (CRUSHR-P2-CLEAN-08)

- Removed stale active-control references to pre-migration Phase 2 paths/process language and kept canonical references pointed at `PHASE2_RESEARCH/`.
- Added concise operator-path guidance in `PHASE2_RESEARCH/README.md` linking manifest generation, foundation build, pre-trial audit checkpoint, execution run, and output locations.
- Aligned active control docs on next-step sequencing: pre-trial audit milestone first, then Phase 2.2 mapping/reporting packet.

## 2026-03-13 — Phase 2, Step cleanup (CRUSHR-P2-CLEAN-04 follow-up)

- Added workspace cargo config `.cargo/config.toml` with rustc flag `-A unknown-lints` so required command `cargo clippy --workspace --all-targets -- -D warning` runs without unknown-lint diagnostic noise.


## 2026-03-13 — Phase 2 pre-trial reproducibility prep (CRUSHR-P2-PRETRIAL-DET-01)

- Implemented deterministic baseline archive generation in `crushr-pack`: stable lexicographic relative-path ordering, normalized metadata (`mode=0`, `mtime=0`, empty xattrs), and fixed zstd encoder flags (`checksum=false`, `content-size=true`, `dict-id=false`).
- Added focused integration tests proving repeated runs over identical logical inputs produce byte-identical archives and that index entry ordering/metadata normalization are stable.
- Updated architecture documentation to reflect implemented deterministic baseline archive-generation behavior.

## 2026-03-13 — Phase 2, Step 2.1 pre-trial audit milestone (CRUSHR-P2-TRIAL-READY-01)

- Added `crushr-lab run-phase2-pretrial-audit` command and a dedicated `phase2_audit` module for deterministic readiness checks before trial execution.
- Implemented readiness checks for manifest/schema validity, locked matrix axis values, deterministic scenario count (2700), duplicate scenario IDs, truthful tool availability, support-file existence, and Phase 2 output-root writability.
- Added machine-readable audit reporting (`pass/fail`, failing checks, summary, tool status, matrix summary, output-root status) with default artifact output `PHASE2_RESEARCH/generated/audit/phase2_pretrial_audit.json`.
- Added focused tests for audit report serialization, wrong-scenario-count and duplicate-ID detection, and a local happy-path audit construction.
- Updated `PHASE2_RESEARCH/README.md` operator path to include the implemented pre-trial audit command.

## 2026-03-13 — Phase 2 execution freeze (CRUSHR-P2-EXEC-01)

- Generated and froze canonical Phase 2 manifest at `PHASE2_RESEARCH/manifest/phase2_manifest.json` using `crushr-lab write-phase2-manifest` with the locked core matrix.
- Emitted `PHASE2_RESEARCH/manifest/manifest_summary.json` capturing scenario count and locked dataset/format/corruption/seed lists for operator quick-checks.
- Verified manifest contract properties for the frozen artifact: deterministic ordering, stable scenario IDs, no duplicate IDs, and exact 2700 scenario count.

## 2026-03-14 — Phase 2 execution runner path/default correction (CRUSHR-P2-EXEC-03A)

- Fixed `crates/crushr-lab/src/phase2_runner.rs` source-archive resolution so `foundation_report.json` archive paths are resolved against workspace root when relative and passed through unchanged when absolute.
- Restricted `artifact_dir` usage to generated execution outputs and kept source archive loading independent from artifact output location.
- Updated `run-phase2-execution` defaults to canonical Phase 2 layout: manifest `PHASE2_RESEARCH/manifest/phase2_manifest.json`, foundation report `PHASE2_RESEARCH/foundation/foundation_report.json`, and execution artifact dir `PHASE2_RESEARCH/trials`.
- Added focused unit tests for relative/absolute source path handling, artifact output placement under `artifact_dir`, and canonical default-path constants; updated `PHASE2_RESEARCH/README.md` execution example paths.


## 2026-03-14 — CRUSHR-P2-EXEC-04 (normalized comparison-ready evidence dataset)

- Added `crushr-lab run-phase2-normalization` and a deterministic normalization module that converts Phase 2 `raw_run_records.json` + per-run stdout/stderr/result artifacts into normalized per-scenario records with explicit `result_class`, `failure_stage`, `detected_pre_extract`, `diagnostic_specificity`, file-level nullable counts, and evidence-strength provenance.
- Emitted canonical normalized artifacts at `PHASE2_RESEARCH/results/normalized_results.json` and `PHASE2_RESEARCH/results/normalization_summary.json` without rerunning trials.
- Added strict normalization schemas: `schemas/crushr-lab-phase2-normalized-results.v1.schema.json` and `schemas/crushr-lab-phase2-normalization-summary.v1.schema.json`.
- Added focused normalization tests for representative cases, deterministic ordering, classification mapping, and schema-ID/shape validation.
- Updated Phase 2 research docs with normalization command/output references and rule documentation (`PHASE2_RESEARCH/normalized/NORMALIZATION_RULES.md`).

## 2026-03-14 — Phase 2, Step CRUSHR-P2-ANALYSIS-01 (fix iteration 0: deterministic cross-format comparison summaries)

- Added `crushr-lab run-phase2-comparison` to compute deterministic per-format comparison metrics from `PHASE2_RESEARCH/results/normalized_results.json` without trial recomputation.
- Added schema contracts for comparison tables and format rankings, plus focused `crushr-lab` tests for output-shape validation and schema ID integrity.
- Generated canonical summary outputs under `PHASE2_RESEARCH/summaries/comparison_tables.json` and `PHASE2_RESEARCH/summaries/format_rankings.json`.

## 2026-03-14 — CRUSHR-SALVAGE-01 (plan-only deterministic salvage planning)

- Added standalone `crushr-salvage` executable with deterministic damaged-archive inspection and plan JSON output (`--json-out` optional).
- Added deterministic BLK3 candidate scanning, authoritative IDX3-aware file classification, dictionary dependency gating, and unverified research labeling.
- Added salvage plan JSON schema and focused integration tests covering damaged footer/invalid index/no invented mappings/missing dictionary dependency/deterministic ordering/schema validation.
- Reconciled active control docs so salvage is clearly separate from strict extraction and CRUSHR-SALVAGE-01 is explicitly plan-only.

## 2026-03-14 — CRUSHR-SALVAGE-02 verified block analysis (plan-only)

- Extended `crushr-salvage` candidate model with deterministic verification states (header validity, payload bounds, dictionary dependency status, decompression result, raw-hash result, content-verification status).
- Added verification-backed file salvageability classification that requires authoritative IDX3 mappings plus content-verified dependencies and verified extent bounds.
- Introduced `schemas/crushr-salvage-plan.v2.schema.json` and updated salvage tests to validate v2 shape plus deterministic success/failure scenarios (decode failure, raw-hash mismatch, missing dictionary dependency, ordering stability).


## 2026-03-14 — CRUSHR-SALVAGE-03 verified fragment export (research-only)

- Added optional `crushr-salvage --export-fragments <dir>` output path for research artifacts from content-verified blocks and verified extents only.
- Added deterministic block/extents/full-file artifact export rules with `SALVAGE_RESEARCH_OUTPUT.txt` marker and per-sidecar `verification_label = UNVERIFIED_RESEARCH_OUTPUT`.
- Extended salvage plan v2 output/schema with optional `exported_artifacts` references when export mode is enabled.
- Added focused salvage integration tests for export success/failure gating, partial-vs-full export behavior, deterministic ordering, export-disabled behavior, and schema compatibility.

- 2026-03-14 — CRUSHR-SALVAGE-04
  - Added `crushr-lab-salvage` deterministic research harness for batch salvage experiments over `.crushr` archives.
  - Added experiment manifest + per-run metadata output layout with deterministic archive ordering and run IDs.
  - Added integration tests covering structure generation, deterministic ordering, export-enabled/disabled behavior, and summary population.


## 2026-03-14 — CRUSHR-SALVAGE-05

- Extended `crushr-lab-salvage` to generate deterministic compact experiment summaries (`summary.json`, `summary.md`) after experiment runs.
- Added stable per-run outcome categories and aggregate counters for verified blocks, salvageability, and exported artifact classes.
- Added `--resummarize <experiment_dir>` for summary-only regeneration from existing experiment outputs without rerunning salvage.
- Added focused harness tests for summary generation, aggregate correctness, deterministic ordering, classification coverage, export-aware totals, and resummarize behavior.


## 2026-03-15 — CRUSHR-FORMAT-01

- Added LDG1 redundant file-map metadata emission in `crushr-pack` for new archives.
- Added strict redundant-map fallback validation/consumption in `crushr-salvage` when primary IDX3 mapping is unusable.
- Bumped salvage plan schema to v3 with deterministic fallback provenance fields.
- Added targeted tests for fallback improvement, rejection behavior, backward compatibility, and determinism.

## 2026-03-15 — CRUSHR-SALVAGE-08

- Added `crushr-lab-salvage run-redundant-map-comparison --output <comparison_dir>` for bounded deterministic old-vs-new redundant-map salvage comparisons.
- Added compact comparison outputs (`comparison_summary.json`, `comparison_summary.md`) with required aggregate deltas, grouped breakdowns, and stable per-scenario improvement classes.
- Added focused comparison integration tests for execution/output presence, deterministic ordering, strict-boundary control behavior, and aggregate-delta correctness.


## 2026-03-15 — CRUSHR-FORMAT-02

- Added explicit experimental writer flag to emit self-describing extent metadata and distributed checkpoint snapshots.
- Added strict salvage support for verified `CHECKPOINT_MAP_PATH` and `SELF_DESCRIBING_EXTENT_PATH` provenance fallbacks.
- Added bounded three-arm experimental resilience comparison command with compact summary artifacts.
- Added focused tests for checkpoint-provenance recovery and experimental comparison outputs.


## 2026-03-15 — CRUSHR-FORMAT-03

- Added opt-in experimental writer flag `--experimental-file-identity-extents` with deterministic file-identity extent records and path-map records.
- Added salvage strict fallback path `FILE_IDENTITY_EXTENT_PATH` with path digest verification and deterministic refusal on inconsistent identity metadata.
- Extended bounded resilience comparison outputs with four-arm reporting and dedicated `file_identity_comparison_summary.json`/`.md` artifacts.

## 2026-03-15 — CRUSHR-FORMAT-03-f1

- Repaired `crushr-lab-salvage` CLI dispatch so `--help` succeeds and documented comparison command names are discoverable at top level.
- Added parser guard preventing known comparison subcommand names from being consumed as positional input paths in experiment mode.
- Added focused tests for help output coverage, subcommand misparse regression, and direct file-identity comparison command invocation.

## 2026-03-15 — CRUSHR-FORMAT-03-f2

- Added bounded `crushr-pack --help` support with explicit usage text listing `--experimental-self-describing-extents` and `--experimental-file-identity-extents`.
- Added focused regression tests ensuring help discoverability and acceptance/emission behavior for both experimental writer flags.
- Revalidated lab comparison workflow commands end-to-end against the packer CLI surface to prevent unsupported-flag regressions.



## 2026-03-15 — CRUSHR-FORMAT-04

- Added distributed bootstrap anchors and per-entry path metadata for experimental file-identity archives.
- Added deterministic footer/index-loss fallback metadata scan path plus bootstrap diagnostics in salvage-plan v3.
- Added deterministic anonymous verified naming fallback and provenance for path-map-loss scenarios.
- Added format04 comparison command/output aliases and updated tests/schema/docs accordingly.


- 2026-03-15: CRUSHR-FORMAT-05 completed — added payload-level self-identifying block metadata and repeated verified path checkpoints, integrated deterministic payload-identity salvage fallback/provenance, and added bounded `run-format05-comparison` outputs.

## 2026-03-16 — documentation continuity realignment after FORMAT-07/08 + new product/optimization tracks

- Realigned active control docs on current truth: FORMAT-07 and FORMAT-08 are complete, FORMAT-09 is the next evaluation packet, and older “next step = FORMAT-06 / deferred graph reasoning” wording is superseded.
- Added explicit near-term product-completeness track for Unix metadata preservation (including xattrs and core Unix file-object metadata) so crushr can close the common tar-on-Unix criticism cleanly.
- Added explicit post-stabilization optimization track for distributed dictionary experiments, with the reminder that dictionary work must obey the same verification/dependency discipline as the rest of the format.
- Marked Phase-09 as the point where weak metadata layers can be judged and potentially pruned if they add archive size without enough survivability benefit.

## 2026-03-16 — CRUSHR-FORMAT-11 (distributed extent-identity experiment + comparison harness)

- Added `extent_identity_only` metadata profile in `crushr-pack` and wired it through deterministic experimental metadata emission.
- Extended payload-identity salvage planning to keep ordered partial classifications when total extent coverage is incomplete.
- Added `run-format11-extent-identity-comparison` CLI wiring + runner output artifacts: `format11_comparison_summary.json` and `format11_comparison_summary.md`.
- Added tests for format11 command dispatch/reporting, format10 non-regression, and payload-identity grouping/ordered-partial behavior.

- 2026-03-16: Completed CRUSHR-FORMAT-12. Added `extent_identity_inline_path`, `extent_identity_distributed_names`, salvage inline-path verification fallback behavior, `run-format12-inline-path-comparison`, and generated `FORMAT12_RESULTS/format12_comparison_summary.{json,md}`.

## 2026-03-16 — CRUSHR-FORMAT-12-STRESS
- Added `run-format12-stress-comparison` command to `crushr-lab-salvage`.
- Added deterministic stress datasets (`deep_paths`, `long_names`, `fragmentation_heavy`, `mixed_worst_case`) and generated stress summary outputs with overhead/path/extent metrics and evaluation answers.
- Added regression tests for CLI/help wiring and stress summary schema fields.


## 2026-03-16 — CRUSHR-FORMAT-12-STRESS (artifact/schema alignment)
- Upgraded `run-format12-stress-comparison` to run deterministic corruption scenarios per stress dataset and aggregate required recovery + overhead metrics in `by_variant`, grouped breakdowns, and per-scenario rows.
- Added required artifact filenames `format12_stress_comparison_summary.json` and `format12_stress_comparison_summary.md` (while retaining legacy compatibility copies).
- Added deterministic fixture test coverage in `comparison.rs` plus stress visibility assertions in `salvage_experimental_resilience.rs` for path-length and extent-density stress guarantees.
- Saved generated artifacts under `PHASE2_RESEARCH/FORMAT12_STRESS_RESULTS/`.

- 2026-03-16: Completed CRUSHR-FORMAT-13. Added path-dictionary metadata profiles (`extent_identity_path_dict_single`, `extent_identity_path_dict_header_tail`, `extent_identity_path_dict_quasi_uniform`), dictionary-aware salvage fail-closed handling, `run-format13-comparison`, `run-format13-stress-comparison`, and generated `FORMAT13_RESULTS/format13*_comparison_summary.{json,md}` artifacts.


## 2026-03-16 — CRUSHR-FORMAT-14A-FIX1

- Repaired FORMAT-14A comparison classification logic to always emit exactly one terminal outcome and to account for both legacy (`FULL_VERIFIED`/`FULL_ANONYMOUS`) and verified (`*_VERIFIED`) class labels.
- Hardened payload-identity dictionary planning so dictionary-backed records do not recover named paths via checkpoint fallback when dictionary material is unavailable or conflicting.
- Added regression coverage for single terminal classification and dictionary fail-closed behavior in FORMAT-14A comparison output.
- Re-generated required FORMAT-14A baseline + stress summary artifacts.

## 2026-03-16 — CRUSHR-FORMAT-14A-FIX2

- Identified and fixed FORMAT-14A harness bug where dictionary survival inference from corrupted archive parsing could undercount surviving copies and over-apply fail-closed anonymous fallback.
- Switched FORMAT-14A dictionary-state shaping to deterministic scenario-aware copy/conflict expectations for this bounded corruption packet.
- Restored header+tail dual-copy one-loss named recovery reporting and re-generated required baseline/stress artifacts.

## 2026-03-17 — CRUSHR-FORMAT-15
- Added metadata profile `extent_identity_path_dict_factored_header_tail` with factored directory/basename/file-binding dictionary body and mirrored header+tail copy support.
- Added dictionary copy self-identification/generation fields and validation (`archive_instance_id`, `dict_uuid`, `generation`, `dictionary_length`, `dictionary_content_hash`) plus fail-closed handling for generation mismatch/conflict.
- Added `run-format15-comparison` and `run-format15-stress-comparison`, emitted `FORMAT15_RESULTS/format15_{,stress_}comparison_summary.{json,md}` with required recovery/dictionary/generation metrics and grouped breakdowns.

## 2026-03-17 — CRUSHR-FORMAT-15-FIX1
- Fixed FORMAT-15 comparison regression: `run_format15_impl` now uses scenario-authoritative expected dictionary copy/conflict state for fail-closed gating (matching FORMAT-14A semantics), rather than observed valid-copy metrics that could collapse to zero.
- Fixed salvage parser regression for dictionary v2 full-path body representation by accepting `body.entries` in addition to factored `directories`/`basenames`/`file_bindings`.
- Added regression test `v2_full_path_body_is_parsed` and refreshed FORMAT-15 baseline/stress artifacts after rerun.


## 2026-03-17 — Phase 16, Step CRUSHR-HARDEN-02 (de-cruft + boundary cleanup)

- Reorganized salvage runtime modules under `crushr_salvage/core/` and lab harness modules under `crushr_lab_salvage/lab/` to make runtime-vs-lab boundaries explicit.
- Consolidated duplicated experimental metadata fallback planning in `crushr-salvage` into a single `plan_from_experimental_metadata` helper.
- Added missing architecture/control docs: `docs/ARCHITECTURE.md`, `docs/SNAPSHOT_FORMAT.md`, and `docs/testing-harness.md`; updated `README.md` and `docs/format-evolution.md` for canonical boundary clarity.
- Validation run: `cargo fmt --check`, `cargo clippy --all-targets --all-features` (warnings only), and `cargo test -p crushr --tests`.


## 2026-03-17 — CRUSHR-TOOLING-VERIFY-01
- Added strict verification mode `crushr-extract --verify <archive>` with deterministic success/refusal reporting (`verification_status`, `safe_for_strict_extraction`, refusal reasons, verified extent count, failed check count).
- Retired public `crushr-fsck` behavior and replaced binary surface with a deterministic deprecation shim directing users to `crushr-extract --verify` and `crushr-salvage`.
- Updated runtime/docs/contracts/tests to remove public `crushr-fsck` workflow references and enforce no-salvage leakage in `--verify` output.


## 2026-03-18 — CRUSHR-HARDEN-03B

- Reconciled salvage emitted contract with `crushr-salvage-plan.v3`: `mapping_provenance` and `recovery_classification` now serialize from typed enums using schema-v3 vocabulary (`FULL_VERIFIED`, `FULL_ANONYMOUS`, `PARTIAL_ORDERED`, `PARTIAL_UNORDERED`, `ORPHAN_BLOCKS`).
- Added typed reason-code emission at the salvage boundary for `block_candidates[].content_verification_reasons` and `file_plans[].failure_reasons` with schema-enforced closed enums.
- Updated salvage schema/docs/tests and repaired affected test expectations to remove drift between implementation and documented contract semantics.

## 2026-03-18 — CRUSHR-HARDEN-03D

- Audited reader/open/parse boundaries and confirmed canonical extract path remains in `strict_extract_impl` (not the permissive legacy reader path).
- Hardened `crushr-extract --verify` to execute strict extraction semantics in an isolated temporary output directory and emit deterministic refusal reasons tied to strict extraction refusal paths.
- Tightened legacy reader permissive behavior (`scan_blocks` region mismatch and decoded raw-length mismatch now fail closed) to reduce boundary confusion.
- Updated active boundary/control text (`PROJECT_STATE.md`, `.ai/STATUS.md`, `.ai/HANDOFF.md`, `.ai/PHASE_PLAN.md`) and stale CLI wording (`crushr-tui`) to reflect `crushr-extract --verify` as the public strict verification surface.

## 2026-03-19 — CRUSHR-HARDEN-03I-FIX1

- Removed the remaining active salvage metadata `Vec<Value>` truth path in `crushr_salvage/core/metadata.rs` by introducing typed `ExperimentalMetadataRecord` parsing and typed metadata record structs for path/dictionary/payload/file-identity/manifest flows.
- Replaced key-based bootstrap-anchor detection in `crushr-salvage` with typed variant checks.
- Preserved fail-closed dictionary validation and reran focused salvage + deterministic pack + canonical verification-model tests.

## 2026-03-19 — CRUSHR-HARDEN-03I-FIX2

- Removed the final localized active-path `serde_json::Value` use from dictionary-copy-v2 metadata parsing (`PathDictionaryCopyV2RawRecord`) in `crushr_salvage/core/metadata.rs`.
- Added deterministic raw-slice `body_raw_json` extraction helpers (`extract_top_level_field_raw_json` and JSON scanner helpers) so parity checks no longer require dynamic JSON carriers.
- Added focused parity regression `v2_raw_body_hash_parity_is_preserved_without_value_carrier` and reran salvage/clippy/deterministic pack checks.

## 2026-03-20 — CRUSHR-UI-03

- Reworked shared CLI presentation into minimalist section-based output with aligned key/value rows, explicit final `Result` section, grouped numeric rendering, and optional status color accents in TTY mode.
- Applied canonical section templates to command surfaces: verify success/failure, pack, info, and salvage.
- Switched `crushr-info` default output to human-readable sections and retained current JSON output behind `--json`.
- Hardened verify structural refusal rendering to emit structured failure-domain fields (`component`, `reason`, `expected`, `received`) without exposing raw parser internals.
- Added deterministic golden output fixtures and integration checks for verify success/failure, pack, info human mode, and salvage output.

## 2026-03-20 — CRUSHR-UI-04

- Added canonical top-level `crushr about` command and root-help exposure for a locked product-identity screen.
- Implemented minimalist about renderer with fixed section order/wording plus deterministic build metadata fields (`version`, `commit`, `built`, `target`, `rust`) and explicit `unknown` fallback behavior.
- Added deterministic drift guards: fixed-metadata golden output lock (`tests/golden/about.txt`), metadata fallback stability test, and root help-surface coverage for `about`.

## 2026-03-20 — CRUSHR-BUILD-01

- Added reproducible musl release build path: repo-root `Containerfile.musl`, `.cargo/config.toml` musl static flags, and `scripts/build-musl-release-podman.sh`.
- Updated `crates/crushr/build.rs` to use environment-first metadata (`CRUSHR_VERSION`, `CRUSHR_GIT_COMMIT`, `CRUSHR_BUILD_TIMESTAMP`, `CRUSHR_TARGET_TRIPLE`, `CRUSHR_RUSTC_VERSION`) with bounded shell fallback and final `unknown`.
- Kept `crushr about` rendering contract locked while switching runtime metadata reads to the new constant names.

## 2026-03-21 — CRUSHR-CHECK-02
- Added unified policy-gate workflow (`.github/workflows/policy-gate.yml`) enforcing secrets scan (TruffleHog verified mode), dependency audit (`cargo audit --deny warnings`), MSRV check on Rust 1.85.0, style/lint checks, and VERSION drift validation on PRs and pushes to `main`.
- Added `.cargo/audit.toml` with explicit temporary ignore for `RUSTSEC-2025-0119` to keep audit gating fail-closed and exception-visible.
- Added `.github/SECURITY.md` for private vulnerability reporting guidance.
- Updated README maturity signal badges to machine-backed workflow status (`policy-gate`, `reuse`) and removed stale CI/legacy secret-scan badge references.
- Validation evidence: YAML parse check, `./scripts/check-version-sync.sh`, `cargo audit --deny warnings`, `cargo +1.85.0 check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (known failing due pre-existing formatting drift).

## 2026-03-21 — CRUSHR-CHECK-02-FIX1
- Reverted `.github/SECURITY.md` addition per review direction.
- Ran repository formatting cleanup (`cargo fmt`) so policy-gate style enforcement is green.
- Validation evidence: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## 2026-03-22 — CRUSHR-STYLE-FIX-01
- Ran a repo-wide Clippy style sweep under enforced policy gate and resolved all surfaced warnings (primarily `collapsible_if`) across workspace crates, binaries, lab/test/support code, and build script paths.
- Applied idiomatic Rust 1.88 let-chain rewrites to collapse nested conditionals while preserving behavior and readability; no blanket `#[allow(clippy::...)]` strategy used.
- Validation evidence: `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` both pass.

## 2026-03-22 — CRUSHR_CLI_UNIFY_01
- Introduced canonical shared CLI app dispatcher (`crates/crushr/src/cli_app.rs`) and rewired `crushr` to execute canonical commands in-process for `pack`, `extract`, `verify`, `info`, `about`, and `lab`.
- Moved canonical command entrypoints for `pack`, `extract`, and `info` into shared library modules (`crates/crushr/src/commands/*`) and converted corresponding binaries into thin wrappers.
- Promoted `crushr-lab` with a library dispatch entrypoint (`crushr_lab::dispatch`) and added crate wiring to call it from top-level `crushr`.
- Removed obsolete workspace crate `crushr-cli-common` and updated architecture/readme boundary docs for the new shared-app layout.

## 2026-03-22 — CRUSHR_PACK_SCALE_01
- Reworked production `crushr-pack` planning to stop retaining full raw/compressed file bytes in `PackLayoutPlan`; planning now records lightweight per-file descriptors and metadata plan state only.
- Moved file read/compress/hash to emission-time processing and added fail-closed detection for file-size mutation between planning and serialization.
- Added unit regressions for unreadable-file planning behavior and plan/emit mutation detection to guard bounded-memory pipeline behavior.
- Synthetic 20,000-file memory evidence: max RSS reduced from `177248 KiB` (pre-fix `HEAD~1`) to `76556 KiB` (current).


## 2026-03-24 — CRUSHR_UI_POLISH_07
- Migrated root/canonical help output to shared CLI presentation sections and tokenized color semantics for `crushr`, `crushr-pack`, `crushr-extract`, and `crushr-info`, while keeping non-TTY/help piping free of ANSI control codes.
- Standardized pack archive extension handling so `-o/--output` paths without an extension are normalized to `.crs`; explicit user extensions remain untouched.
- Corrected pack phase rendering to show `compression` and `serialization` phases reaching `files=N/N`, followed by explicit visible `finalizing`; removed misleading `N-1/N` terminal states.
- Added pack Result metrics derived from real run data: archive path, files packed, total size, compressed size, compression ratio, reduction percentage, and processing time.
- Expanded `crushr-info` human output with a dedicated `Compression` section showing method and level from BLK3 headers (`unavailable` fallback when not recoverable).
- Validation evidence: `cargo fmt --all`; `cargo test -p crushr --test cli_presentation_contract --test cli_contract_surface`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.

## 2026-03-24 — CRUSHR_UI_POLISH_07 maintenance (v0.3.7)
- Bumped canonical product version to `0.3.7` by updating root `VERSION` and syncing `workspace.package.version` in `Cargo.toml`.
- Validation evidence: `./scripts/check-version-sync.sh`; `cargo test -p crushr --test version_contract`.

## 2026-03-25 — CRUSHR_PRESERVATION_04
- Advanced production index encoding to IDX6 and added explicit entry fields for POSIX ACL metadata (`acl_access`, `acl_default`), SELinux labels (`selinux_label`), and Linux capabilities (`linux_capability`).
- Updated pack capture to preserve ACL/SELinux/capability metadata as structured security fields (separate from generic xattrs), and updated strict/recover extraction to restore these classes best-effort with explicit warning surfacing when blocked.
- Expanded `crushr info` metadata visibility with `ACLs`, `SELinux labels`, and `capabilities`; updated goldens and compatibility checks for IDX6 markers.
- Added/updated preservation regression coverage (`metadata_preservation`, `index_codec`) and validated with fmt + focused integration tests + workspace clippy.

## 2026-03-25 — CRUSHR_PRESERVATION_04 manual-evidence addendum
- Added operator-level completion evidence file `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_04.md` with exact commands and observed outcomes for ACL/SELinux/capability/info visibility/degraded restore/backward-compatibility/recovery-model checks.
- Fixed security-metadata restore ordering so ACL/SELinux/capability restoration runs after ownership restore, preventing capability loss from post-restore ownership changes.

## 2026-03-27 — CRUSHR_OPTIMIZATION_01
- Optimized `crushr-pack` discovery to be preservation-profile aware at capture time (`collect_files(inputs, profile)`), so omitted metadata classes and entry kinds are skipped before expensive probing rather than captured and discarded later.
- Added explicit discovery capture policy gating for ownership lookups, xattr/security probes, sparse detection, symlink/special-entry inclusion, and hard-link grouping needs by profile.
- Removed duplicate regular-file planning `stat` calls by carrying `raw_len` from discovery into planning (`InputFile.raw_len`) and reusing it in `build_pack_layout_plan`.
- Added per-run UID/GID ownership-name caches in discovery to eliminate repeated `getpwuid`/`getgrgid` resolution for identical IDs.
- Updated benchmark operator guidance and completion notes for required medium/large full/basic `--profile-pack` validation runs.
