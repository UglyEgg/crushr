# .ai/CHANGELOG.md

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

- Added explicit extraction mode selection to `crushr-extract` via `--mode <strict|salvage>` with default strict behavior preserved.
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
