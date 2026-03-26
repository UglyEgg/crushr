CRUSHR_BENCHMARK_01 complete (2026-03-26):
- Added deterministic benchmark dataset generator at `scripts/benchmark/generate_datasets.py` with three benchmark classes and emitted `dataset_manifest.json`.
- Added deterministic benchmark harness at `scripts/benchmark/run_benchmarks.py` capturing command provenance, archive size, pack/extract wall time, and peak RSS per run.
- Added benchmark contract/reference docs (`docs/reference/benchmarking.md`) plus locked output schema (`schemas/crushr-benchmark-run.v1.schema.json`).
- Benchmarks now run `tar+zstd`, `tar+xz`, `crushr --preservation full`, and `crushr --preservation basic` for each dataset and write structured output to `.bench/results/benchmark_results.json`.
- Canonical version advanced to `0.4.14` (`VERSION` + workspace package version sync).
- Validation in packet: `cargo build --release -p crushr`; `python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets`; `python3 scripts/benchmark/run_benchmarks.py --datasets .bench/datasets --crushr-bin target/release/crushr --output .bench/results/benchmark_results.json`; `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-version-sync.sh`.

CRUSHR_PACK_STREAMING_01 complete (2026-03-26):
- Production `pack` no longer retains per-block raw payload vectors for the entire run; hard-link reuse cache now stores only block metadata (lengths/hashes/offset).
- File-manifest digest writing now reuses the already computed `raw_hash` for each block, removing a hidden whole-run payload-retention path.
- Repro/evidence captured in `.ai/COMPLETION_NOTES_CRUSHR_PACK_STREAMING_01.md` with exact commands and before/after RSS (`HEAD~1` vs current) on a deterministic 250-file dataset.
- Canonical version advanced to `0.4.13` (`VERSION` + workspace package version sync).
- Validation in packet: `cargo fmt --all`; `cargo test -p crushr pack_fails_if_file_changes_between_planning_and_emit -- --nocapture`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`; plus runtime `pack/info/extract --verify/extract --all` probes.

CRUSHR_INTROSPECTION_02 complete (2026-03-25):
- `crushr info` now has explicit preservation contract labeling and entry-kind summary rows, plus metadata visibility states (`present`, `not present`, `omitted by profile`) to separate omission intent from degradation semantics.
- `crushr info --list` now prints profile/scope context and keeps non-regular omission messaging informational while preserving fail-closed proof-unavailable warning behavior.
- README introspection wording now explicitly states `info` is archive-contract truth; extraction/recovery outcomes like `metadata_degraded` are not implied by `info`.
- Canonical version advanced to `0.4.12` (`VERSION` + workspace package version sync).
- Validation in packet: `cargo fmt --all`; `cargo test -p crushr --test cli_presentation_contract --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`.

CRUSHR_RECOVERY_MODEL_08 complete (2026-03-25):
- Strict extraction fail-closed metadata restoration semantics now cover non-regular canonical outputs too (directories, symlinks, FIFOs, char devices, block devices) when metadata classes required by recorded preservation profile fail to restore.
- Recover extraction now routes non-regular canonical outputs into `metadata_degraded/` (with manifest entries) when profile-required metadata restoration fails; they no longer remain warning-only canonical.
- Recover manifest metadata degradation fields (`trust_class`, `failed_metadata_classes`, `degradation_reason`) are now populated consistently for non-regular metadata-degraded outcomes.
- Profile-aware omission behavior remains intact for non-regular outputs: omitted-by-profile metadata classes (e.g., ownership under `basic`) do not trigger false degradation.
- Added deterministic coverage in `metadata_preservation.rs` for strict refusal + recover metadata_degraded routing/manifest truth for directory/symlink/FIFO plus basic-profile omission non-degradation behavior.
- Version advanced to `0.4.11` (`VERSION` + workspace package version).
- Validation in packet: `cargo fmt --all`; `cargo test -p crushr --test metadata_preservation --test recovery_extract_contract`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo test -p crushr --test version_contract`.

CRUSHR_PRESERVATION_05 complete (2026-03-25):
- Production pack now exposes explicit preservation contract flag `--preservation <full|basic|payload-only>` (default `full`), with explicit warn-and-omit behavior for excluded entry kinds and no flattening fallback.
- Index encoding advanced to IDX7 with structured preservation-profile recording; legacy IDX3/IDX4/IDX5/IDX6 decode paths default to `full` profile.
- Strict/recover canonical metadata-degraded classification is profile-aware for regular canonical outputs; profile-omitted classes are no longer misclassified as metadata restoration failure.
- `crushr info` now displays `Preservation / profile` and recognizes IDX7 format markers.
- Added deterministic profile tests in `metadata_preservation.rs` and updated CLI/index/core compatibility fixtures.
- Validation in packet: `cargo fmt --all`; `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract --test index_codec --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.

CRUSHR_RECOVERY_MODEL_07 complete (2026-03-25):
- Recovery/extraction trust classes now include explicit `metadata_degraded`; canonical now requires successful required-metadata restoration (not only path/name/data proof).
- Recover output layout now includes `metadata_degraded/` and no longer permits silent metadata-degraded merge into `canonical/`.
- Strict extraction now refuses when metadata restoration fails and surfaces explicit metadata-failure cause text.
- Recovery manifest entries now include explicit degradation fields (`trust_class`, `missing_metadata_classes`, `failed_metadata_classes`, `degradation_reason`) and schema is updated accordingly.
- Recover summary/trust-class rows now include `metadata_degraded` and use `anonymous` result-count label.
- Coverage limitation: metadata-degraded placement/classification is currently complete for regular-file canonical outputs; directories, symlinks, and special entries still follow warning-based metadata restore behavior and are not yet fully classified/relocated as metadata-degraded.
- Validation in packet: `cargo fmt --all`; `cargo test -p crushr --test recovery_extract_contract --test metadata_preservation`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`.

CRUSHR_PRESERVATION_04 complete (2026-03-25):
- Production index format is now IDX6 (IDX3/IDX4/IDX5 decode compatibility retained) with explicit fields for POSIX ACL metadata, SELinux label metadata, and Linux capability metadata.
- Pack now captures ACL/SELinux/capability values as structured security metadata and keeps generic xattrs separate to avoid silent class omission.
- Strict/recover extraction now restore ACL/SELinux/capability metadata best-effort and emit explicit warning classes (`WARNING[acl-restore]`, `WARNING[selinux-restore]`, `WARNING[capability-restore]`) when restore is blocked.
- Security metadata restore ordering now applies ACL/SELinux/capability after ownership restore so capability xattrs are not cleared by subsequent ownership changes.
- `crushr info` metadata visibility now includes `ACLs`, `SELinux labels`, and `capabilities`; format marker now reports IDX6 when present.
- Exact operator/manual validation evidence is recorded at `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_04.md`.
- Validation run in this packet: `cargo test -p crushr --test index_codec --test metadata_preservation`, `cargo test -p crushr --test deterministic_pack --test mvp --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`.

CRUSHR_PRESERVATION_03 complete (2026-03-25):
- Production index format is now IDX5 (IDX3/IDX4 decode compatibility retained) with new entry-kind coverage for FIFO + character/block device files, sparse-file logical extent mapping, and optional device major/minor metadata.
- Pack now captures sparse extents (Linux hole/data probing), ownership names (`uname`/`gname`) as enrichment, FIFO/device node kinds, and preserves numeric uid/gid as authoritative ownership truth.
- Strict/recover extraction now restore sparse files hole-aware, recreate FIFO/device nodes when supported/permitted, and emit explicit `WARNING[special-restore]` degradation when special restoration is blocked.
- `crushr info` metadata visibility now includes `sparse files` and `special files`; format marker now reports IDX5 when present.
- Validation run in this packet: `cargo test -p crushr --test metadata_preservation`, `cargo test -p crushr --test deterministic_pack --test mvp --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`.

CRUSHR_PRESERVATION_02 complete (2026-03-25):
- Pack/index now preserve ownership + hard-link semantics in IDX4: entries include `uid/gid` (optional names) and regular-file hard-link group identity with shared payload blocks.
- Strict/recover canonical extraction now restores ownership best-effort with explicit `WARNING[ownership-restore]` on permission/platform failures; extraction continues without silent metadata drop.
- Hard links now round-trip as hard links (shared inode/dev) instead of duplicated payload materialization.
- `crushr info` now includes a concise `Metadata` section showing presence/absence for modes, mtime, xattrs, ownership, and hard links.
- Added deterministic metadata regression coverage for hard-link round-trip and ownership-restore warning path; updated golden outputs for pack/info IDX4 semantics.
- Exact reproducible validation commands + observed outputs are recorded in `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_02.md`.

CRUSHR_PRESERVATION_01 complete (2026-03-24):
- Baseline Linux-first metadata preservation is now wired in production pack/extract: directories and symlinks are retained in IDX3, and regular entries preserve mode/mtime/xattrs.
- Strict extraction now materializes directory/symlink entries and restores mode/mtime/xattrs for regular files/directories (with explicit xattr restore warnings on failure rather than silent drop).
- Recover extraction no longer fails on non-regular index entries; it preserves canonical directory/symlink structure while maintaining existing recovery behavior for refused regular payloads.
- New regression coverage: `metadata_preservation.rs` (mode/mtime/symlink/empty-dir/xattr round-trip) plus deterministic/golden updates for metadata-aware archives.
- Deferred/known limits: uid/gid is intentionally deferred; permission-denied xattr-restore warning paths are implemented but not deterministically CI-covered in all environments.

CRUSHR_INTROSPECTION_01-FIX2 complete (2026-03-24):
- `info --list` now keeps omission-only cases as informational (`COMPLETE`) and reserves `DEGRADED` for structural proof issues.
- `omitted entries` result row now appears only when count > 0.

CRUSHR_INTROSPECTION_01-FIX1 complete (2026-03-24):
- `crushr info --list` now surfaces an explicit omitted-entry count for non-regular IDX3 entries so future entry-kind growth is visible instead of silent.
- Degraded proof-unavailable list output now includes explicit operator guidance to run `crushr salvage <archive>` for recovery-oriented evidence.
- Canonical version is now `0.4.1`.

CRUSHR_INTROSPECTION_01 complete (2026-03-24):
- Added `crushr info --list` for pre-extraction archive content inspection using metadata/index proof only.
- Default output is hierarchical tree; `--flat` emits deterministic full-path listing.
- Corruption handling is fail-closed: only IDX3-proven paths are shown, warnings are emitted when full structure or listing proof is degraded/unavailable.

CRUSHR_UI_POLISH_08 complete (2026-03-24):
- Pack progress rows are now explicitly stable for operators: persistent `compression` and `serialization` phases (no alternating labels) with explicit handoff into `finalizing` after both settle.
- Info structure terms now reflect the actual file-level 1:1 model: `files`, `compressed units`, `file mappings`, plus explicit `block model` line (`file-level (1:1 file → unit)`).
- This info change is presentation-only (no internal format/counting logic change).
- Canonical product version is aligned to `0.3.5` (`VERSION` + workspace package version synchronized).

CRUSHR_UI_POLISH_07 complete (2026-03-24):
- Help output now uses shared presentation/visual tokens; non-TTY remains ANSI-clean.
- Pack now defaults extensionless `-o` targets to `.crs`, reports truthful `compression`/`serialization` N/N + visible `finalizing`, and emits runtime/compression metrics.
- Info now includes `Compression` section with method + level from archive block headers.
- Version follow-up (superseded by UI_POLISH_08): canonical product version is now `0.3.5` (`VERSION` + workspace package version synchronized).

CRUSHR_UI_POLISH_06 completion update (2026-03-24):
- Shared CLI presentation now enforces one canonical title treatment: leading blank line before command output, UTF-8 double-line divider, and stable key/value column alignment even under ANSI color (padding applied before coloring).
- `crushr about` was rebuilt onto the same visual contract (colorized title/section/labels, canonical divider, shared key width), eliminating prior bespoke style drift.
- `crushr-info` human mode now reports product-grade inspection fields (regular file count, extents, logical bytes, payload block count, dictionary table/ledger, compression level when recoverable), and no longer leaks raw internal label `has dct1`.
- Version milestone advanced to `0.3.5` for this v0.3.x CLI/inspection polish pass; presentation goldens were refreshed and full fmt/clippy/workspace tests are green.

CRUSHR_UI_POLISH_04 completion update (2026-03-24):
- Refined shared-motion progress behavior for `crushr-pack` by wiring live serialization detail updates through `ActivePhase::set_detail` and settling the phase with stable final file-count truth.
- Added non-TTY cleanliness integration coverage for `pack`, `verify`, `extract`, and `extract --recover` with `CRUSHR_MOTION=full` to lock no residual carriage-control/spinner artifacts in piped output.
- Updated `crates/crushr/tests/golden/pack.txt` for the stabilized serialization detail row and validated with fmt + focused integration tests + workspace clippy/tests.

CRUSHR_UI_POLISH_03 completion update (2026-03-23):
- Added a shared active-phase motion layer in `crates/crushr/src/cli_presentation.rs` (`begin_active_phase` + `ActivePhase`) with centralized animation lifecycle, bounded cadence, and stable settle/freeze behavior.
- Added explicit motion controls (`CRUSHR_MOTION=full|reduced|off`, `CRUSHR_NO_MOTION=1`) plus strict TTY gating so non-interactive logs/pipes never receive spinner control sequences.
- Migrated `pack`, `extract`, and `verify` progress rendering to shared active-phase transitions (including recover callbacks) and updated progress goldens for settled non-interactive phase output.
- Added `.ai/contracts/CLI_MOTION_POLICY.md` and updated continuity docs (`STATUS`, `PHASE_PLAN`, `DECISION_LOG`, `CHANGELOG`) for step closeout.

CRUSHR_UI_POLISH_01 completion update (2026-03-23):
- Added a shared semantic visual token system in `crates/crushr/src/cli_presentation.rs` (title/section/label/muted/running/pending/success/degraded/failure/info + trust-class tokens).
- Standardized user-facing status vocabulary to `PENDING|RUNNING|COMPLETE|DEGRADED|FAILED|REFUSED` (with compatibility aliases retained where needed), and mapped prior human-output `PARTIAL` surfaces to `DEGRADED`.
- Added explicit recover-mode trust-class rendering (`CANONICAL`, `RECOVERED_NAMED`, `RECOVERED_ANONYMOUS`, `UNRECOVERABLE`) plus contract doc `.ai/contracts/CLI_VISUAL_SEMANTICS.md`; updated presentation goldens/tests accordingly.

CRUSHR_RECOVERY_MODEL_06 completion update (2026-03-23):
- Hardened zip-family high-confidence boundaries by requiring `_rels/.rels` alongside OOXML content markers before classifying `docx`/`xlsx`/`pptx`; generic zip-like payloads now stay medium-confidence `zip`.
- Added deterministic naming-collision regression coverage to lock unique assigned names for same payload across sequential recovery IDs.
- Extended clean recover-mode contract checks to assert zero recovered files under both `recovered_named/` and `_crushr_recovery/anonymous/`, while keeping empty manifest entries for clean archives.

CRUSHR_RECOVERY_MODEL_05 completion update (2026-03-23):
- Recover-mode progress output now streams real phase checkpoints from execution (`archive open`, `metadata scan`, `canonical extraction`, `recovery analysis`, `recovery extraction`, `manifest/report finalization`) instead of a deferred static pre-run dump.
- Recover final reporting now uses trust-class-aligned count labels (`recovered_named`, `recovered_anonymous`), keeps canonical/recovery completeness as separate status rows, and emits non-canonical notes only when recovery artifacts/unrecoverable loss are present.
- Clean recover runs now stay calm (no false recovery warning text); damaged recover runs explicitly surface the recovery-manifest path (`_crushr_recovery/manifest.json`) for structured follow-up.

CRUSHR_RECOVERY_MODEL_04 completion update (2026-03-23):
- Added deterministic recovery-validation integration test `crates/crushr/tests/recovery_validation_corpus.rs` covering clean baseline, tail truncation, index metadata damage, payload-hash mismatch (named recovery), and mixed-outcome recovery extraction in one archive.
- Added deterministic multi-block corpus generation to force anonymous recovery tiers (high/medium/low confidence naming) and explicit unrecoverable entries, with manifest-to-filesystem truth assertions.
- Added technical note `RECOVERY_VALIDATION_CORPUS.md` documenting corpus composition, corruption operations, and intended contract proof points.

CRUSHR_RECOVERY_MODEL_03 completion update (2026-03-23):
- Added `recovery_classification` module with data-driven signature table + structure validators and a strict confidence ladder (high/medium/low) used for recovered content typing.
- Recover manifest now records trust class separately (`recovery_kind`) and content classification fields (`classification.kind`, `confidence`, `basis`, optional `subtype`), aligned to anonymous naming tiers.
- Recovery contract tests were updated for the new manifest shape and classification metadata assertions; workspace fmt/clippy/tests are green.

CRUSHR_RECOVERY_MODEL_02 completion update (2026-03-23):
- `crushr-extract --recover` now runs salvage analysis as part of extract execution through shared salvage planning (`build_recovery_analysis`), avoiding duplicated planning logic in recover extract.
- Human recover output now shows required phase progress and required Result/Trust summary rows (`canonical files`, `named recovered`, `anonymous recovered`, `unrecoverable`; trust COMPLETE|PARTIAL).
- Recover extraction now writes full refused-entry recovery to `recovered_named/` with manifest class `recovered_named`, keeps partial fallback anonymous output + manifest class `recovered_anonymous`, and retains `unrecoverable` manifest rows for zero-byte recoveries.

CRUSHR_RECOVERY_MODEL_01-FIX1 completion update (2026-03-23):
- Fixed rustfmt drift in `crates/crushr/tests/recovery_extract_contract.rs` reported by `cargo fmt --check`; no functional/runtime behavior change.

CRUSHR_RECOVERY_MODEL_01 completion update (2026-03-23):
- Added `crushr-extract --recover` as the recovery-aware extraction mode while preserving strict extraction as the default behavior.
- Recover mode now writes segregated trust-boundary output structure: `canonical/`, `recovered_named/`, `_crushr_recovery/anonymous/`, plus required `_crushr_recovery/manifest.json`.
- Implemented manifest output contract (`crushr-recovery-manifest.v1`) with recovery classification, confidence/basis, original identity status, and recovery reason fields.
- Added integration tests for clean and damaged archives to assert output structure, manifest emission, and anonymous naming behavior.

CRUSHR-STYLE-FIX-01 completion update (2026-03-22):
- Completed a full workspace Clippy cleanup pass under enforced gate (`cargo clippy --workspace --all-targets -- -D warnings`) and removed all currently surfaced warnings without adding blanket lint suppressions.
- Primary cleanup class was `collapsible_if`; rewrites use Rust 1.88-compatible let-chains and preserve existing behavior.
- Required style commands now pass cleanly: `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Follow-up packets should preserve this clean style baseline and treat new Clippy warnings as regressions.

CRUSHR-CHECK-02-FIX1 completion update (2026-03-21):
- Reverted `.github/SECURITY.md` addition per follow-up review request.
- Applied repository formatting cleanup with `cargo fmt`; style gate commands now pass (`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`).

CRUSHR-CHECK-02 completion update (2026-03-21):
- Added unified GitHub Actions workflow `.github/workflows/policy-gate.yml` covering secrets (`trufflehog --only-verified` with full history), dependency audit (`cargo audit --deny warnings`), MSRV check (`cargo +1.85.0 check --workspace`), style (`check-crate-policy`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`), and version drift (`./scripts/check-version-sync.sh`).
- Added explicit cargo-audit exception policy in `.cargo/audit.toml` for `RUSTSEC-2025-0119` only; all other audit warnings/advisories fail the gate.
- Added `.github/SECURITY.md` private vulnerability reporting guidance and updated README badges to point to real enforced workflows.
- Important current constraint: `cargo fmt --check` fails today due pre-existing repository formatting drift; the new style gate remains intentionally truthful/fail-closed until that drift is resolved in a dedicated cleanup step.

CRUSHR-CRATE-01 completion update (2026-03-21):
- Locked workspace crate policy to `resolver = "3"`, `edition = "2024"`, and initial MSRV `rust-version = "1.85"` in root `Cargo.toml`.
- Added explicit `rust-version.workspace = true` inheritance to all member crates and normalized publishable crate metadata inheritance for crates.io-facing fields.
- Locked internal publish intent with explicit `publish = false` on `crushr-cli-common`, `crushr-lab`, and `crushr-tui`; publishable crates are now `crushr`, `crushr-core`, and `crushr-format`.
- Added `scripts/check-crate-policy.sh` to fail on missing `package.name`, publish-intent ambiguity for internal crates, missing publishable metadata inheritance, and workspace resolver/edition/MSRV drift.
- Verified policy with `./scripts/check-crate-policy.sh`, `cargo metadata --format-version 1 --no-deps`, and `cargo +1.85.0 check --workspace`.

CRUSHR-BUILD-01 completion update (2026-03-20):
- Added `Containerfile.musl` (Alpine + rustup musl target), `.cargo/config.toml` musl static rustflags, and `scripts/build-musl-release-podman.sh` (release build + checksum + runtime sample path).
- Build script now supports environment-first metadata injection for release builds: `CRUSHR_VERSION`, `CRUSHR_GIT_COMMIT`, `CRUSHR_BUILD_TIMESTAMP`, `CRUSHR_TARGET_TRIPLE`, `CRUSHR_RUSTC_VERSION`; bounded shell fallback remains for local environments.
- Final metadata fallback behavior is explicit `unknown`; no metadata path panics if tools/env are missing.
- `crushr about` now reads the new runtime constants while keeping locked UI wording/order unchanged.

CRUSHR-UI-04 completion update (2026-03-20):
- Added top-level `crushr about` command on the canonical `crushr` surface and exposed it in root help output.
- Implemented locked minimalist about renderer with fixed section order/wording and present-state-only behavior/data-model statements.
- Added deterministic compile-time build metadata injection (commit/built/target/rustc) with bounded `unknown` fallback when metadata is unavailable.
- Added about drift coverage: fixed-metadata golden rendering test, metadata-fallback stability test, and help-surface assertion that includes `about`.

CRUSHR-VERSION-01 completion update (2026-03-20):
- Root `VERSION` is now the canonical human-edited product version source (strict SemVer only, initial value `0.2.2`).
- Runtime/tool metadata version surfaces now route through `crushr::product_version()` (sourced from `VERSION`) instead of `env!("CARGO_PKG_VERSION")` strings in active output paths.
- Version drift/validation tooling is in place: `scripts/check-version-sync.sh` and integration test `crates/crushr/tests/version_contract.rs` enforce strict SemVer and `VERSION == workspace.package.version`; `scripts/sync-version.sh` updates workspace Cargo version from `VERSION` for single-touch bumps.

CRUSHR-UI-03 completion update (2026-03-20):
- Shared CLI presenter now renders the minimalist section contract: `<tool>  /  <action>`, fixed horizontal rule, title-based sections, aligned label/value rows, and explicit terminal `Result` section.
- Canonical section templates are now enforced in runtime tools: verify success (`Archive/Verification/Result`), verify refusal (`Archive/Failure domain/Result`), pack (`Target/Progress/Result`), info (`Archive/Structure/Result`), salvage (`Archive/Candidates/Evidence/Result`).
- `crushr-info` now defaults to human-readable sections; `--json` preserves existing snapshot JSON output contract.
- Verify structural failure rendering is now structured (`component/reason/expected/received`) with no raw parser error leakage in normal human output.
- Added deterministic golden output fixtures + contract test coverage for verify success, verify failure, pack, info human mode, and salvage (`tests/golden/*.txt`, `cli_presentation_contract.rs`).

<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Handoff

CRUSHR_UI_POLISH_06 completion update (2026-03-24):
- Shared CLI presentation now enforces one canonical title treatment: leading blank line before command output, UTF-8 double-line divider, and stable key/value column alignment even under ANSI color (padding applied before coloring).
- `crushr about` was rebuilt onto the same visual contract (colorized title/section/labels, canonical divider, shared key width), eliminating prior bespoke style drift.
- `crushr-info` human mode now reports product-grade inspection fields (regular file count, extents, logical bytes, payload block count, dictionary table/ledger, compression level when recoverable), and no longer leaks raw internal label `has dct1`.
- Version milestone advanced to `0.3.5` for this v0.3.x CLI/inspection polish pass; presentation goldens were refreshed and full fmt/clippy/workspace tests are green.

CRUSHR_UI_POLISH_03 completion update (2026-03-23):
- Added a shared active-phase motion layer in `crates/crushr/src/cli_presentation.rs` (`begin_active_phase` + `ActivePhase`) with centralized animation lifecycle, bounded cadence, and stable phase settlement/freeze behavior.
- Added explicit motion policy controls (`CRUSHR_MOTION=full|reduced|off`, `CRUSHR_NO_MOTION=1`) with strict TTY gating so pipes/logs stay free of spinner control characters.
- Migrated `pack`, `extract`, and `verify` progress rendering to shared phase transitions through the new primitives (including recover-mode incremental phases), and refreshed CLI presentation goldens for the settled phase-row model.
- Added `.ai/contracts/CLI_MOTION_POLICY.md` and contracts index entry to lock semantic/anti-noise motion behavior.

CRUSHR_UI_POLISH_02 completion update (2026-03-23):
- Added reusable shared CLI structural primitives in `crates/crushr/src/cli_presentation.rs`: `phase` (progress row with optional detail), `banner` (`INFO`/`WARNING`/`FAILURE`), and `result_summary` for standardized final summaries.
- Migrated core command presentation flows to those primitives: `pack`, `extract`, `extract --recover`, `verify`, and `info` now share stable title/target/progress/result layout and shared warning/failure framing.
- Updated verify golden fixtures to lock the structural changes (`Target` section naming + failure banner block), and revalidated presentation/recovery/workspace gates.

CRUSHR_RECOVERY_MODEL_02 completion update (2026-03-23):
- `crushr-extract --recover` now runs salvage analysis as part of extract execution through shared salvage planning (`build_recovery_analysis`), avoiding duplicated planning logic in recover extract.
- Human recover output now shows required phase progress and required Result/Trust summary rows (`canonical files`, `named recovered`, `anonymous recovered`, `unrecoverable`; trust COMPLETE|PARTIAL).
- Recover extraction now writes full refused-entry recovery to `recovered_named/` with manifest class `recovered_named`, keeps partial fallback anonymous output + manifest class `recovered_anonymous`, and retains `unrecoverable` manifest rows for zero-byte recoveries.

CRUSHR_VERIFY_SCALE_01 completion update (2026-03-23):
- `crushr-extract --verify` no longer routes through temp-directory strict extraction output; verify now executes strict validation in verify-only mode (no output materialization).
- Strict extraction no longer retains/clones a cross-run decompressed payload cache for block reads, reducing memory retention pressure in verify and extraction flows.
- Human verify output now includes deterministic real work phases (`archive open / header read`, `metadata/index scan`, `payload verification`, `manifest validation`, `final result/report`), with updated integration assertions and golden fixtures.
- Synthetic scale check: verify completed successfully on a generated 12,000-file archive without OOM termination in this environment.

CRUSHR_CLI_UNIFY_04 completion update (2026-03-22):
- Public `crushr-pack`/`crushr pack` now exposes only production-facing controls (`<input>...`, `-o/--output`, `--level`, `--silent`); all experimental format/layout/profile flags were removed from the production parser/help surface (no hidden compatibility acceptance).
- Added lab-owned experimental pack entrypoint `crushr lab pack-experimental ...` and switched lab comparison harness pack invocation paths to that lab command.
- Updated integration coverage (`deterministic_pack`, `salvage_experimental_resilience`, lab comparison runner wiring) to enforce production rejection + lab acceptance for experimental pack controls.

CRUSHR_CLI_UNIFY_03 completion update (2026-03-22):
- Added `crates/crushr/tests/cli_contract_surface.rs` to enforce canonical CLI contract invariants: approved root command taxonomy, wrapper/canonical help-about-version parity, negative legacy-alias rejection, root exit-code behavior, and combined shared-flag behavior (`--json` + `--silent`).
- Removed remaining undocumented alias drift by restricting help/version handling to first-argument controls only across shared wrapper and command dispatch paths (`wrapper_cli`, `pack`, `extract`, `info`, `salvage`).
- Synced CLI docs/help usage with runtime behavior (`README` wrapper-control position contract; `crushr-info` usage now shows optional `--json`).

CRUSHR_CLI_UNIFY_02 completion update (2026-03-22):
- Converted retained companion wrappers in `crates/crushr` (`crushr-pack`, `crushr-extract`, `crushr-info`, `crushr-salvage`) to use one shared wrapper entry helper (`crushr::wrapper_cli::run_wrapper_env`) so wrappers no longer own independent help/version/about logic.
- Migrated `crushr-salvage` runtime implementation into shared library command module (`crates/crushr/src/commands/salvage.rs`) and switched top-level `crushr salvage` to in-process shared dispatch.
- Enabled explicit bin target control via `autobins = false` and retained only approved/runtime-needed binaries in `crates/crushr/Cargo.toml` (`crushr`, `crushr-pack`, `crushr-extract`, `crushr-info`, `crushr-salvage`, plus internal test harness `crushr-lab-salvage`).
- Removed deprecated `crushr-fsck` binary from active build outputs; updated `crushr-core` minimal-pack test to assert removal from retained surface.

CRUSHR_CLI_UNIFY_01 completion update (2026-03-22):
- Canonical `crushr` command host now dispatches through shared command model/entrypoint code in `crates/crushr/src/cli_app.rs` (pack/extract/verify/info/about/lab surfaces).
- Canonical command implementations for `pack`, `extract`, and `info` were moved into shared library modules under `crates/crushr/src/commands/`; binary targets are now thin wrappers to those shared entrypoints.
- `crushr-lab` now exports a library dispatch entrypoint (`crushr_lab::dispatch`) so `crushr lab` runs in-process.
- Obsolete workspace shared-CLI placeholder crate `crushr-cli-common` was removed.

CRUSHR-UI-02 completion update (2026-03-20):
- Replaced legacy top-level monolithic `crushr` command surface with a bounded dispatcher identity aligned to the canonical suite: `pack`, `extract`, `verify`, `info`; bounded non-primary surfaces remain `salvage` and `lab`.
- Removed legacy generic-compressor commands from primary help exposure (`append`, `list`, `cat`, `dict-train`, `tune`, `completions`), and now return explicit demotion guidance if invoked.
- Rewrote root help/branding/examples so `crushr --help` describes an integrity-first preservation suite instead of generic solid-block compressor language.
- Repaired strict verify structural-failure presentation in `crushr-extract --verify`: normal output now emits deterministic operator-facing refusal sections (`failure_domains`, bounded refusal reason) and no longer surfaces raw parser internals such as `parse FTR4: bad footer magic ...`.
- Added coverage in `crates/crushr/tests/cli_presentation_contract.rs` for root-help command-surface assertions and invalid-archive verify leakage prevention checks.

CRUSHR-UI-01 completion update (2026-03-20):
- Added shared CLI presentation helper (`crates/crushr/src/cli_presentation.rs`) used by `crushr-pack`, `crushr-extract`, and `crushr-salvage`.
- Adopted bounded status vocabulary for human output (`VERIFIED`, `OK`, `COMPLETE`, `PARTIAL`, `REFUSED`, `FAILED`, `RUNNING`, `SCANNING`, `WRITING`, `FINALIZING`) and a deterministic section/header/outcome grammar.
- Standardized `--silent` one-line summaries across `crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` for scripting.
- Added integration coverage in `crates/crushr/tests/cli_presentation_contract.rs` for output determinism (`--verify`) and silent-mode one-line behavior across all scoped commands.
- FIX1 follow-up: restored workspace Cargo manifest validity by re-adding missing `package.name` entries across all workspace crates; blocked `cargo fmt --all` and UI/runtime validation commands now execute successfully.
- Output-mode correction from validation review: `crushr-salvage` now defaults to human presentation output; machine-readable JSON on stdout requires `--json` (while `--json-out` remains available).


License compliance update (2026-03-20):
- License metadata follow-up (2026-03-20): replaced `.reuse/dep5` with `REUSE.toml` to eliminate REUSE deprecation warnings; compliance remains green.
- CRUSHR-LICENSE-01 is complete.
- Repository licensing is now unified: code is MIT OR Apache-2.0, docs/diagrams are CC-BY-4.0.
- SPDX headers were applied repo-wide for source/docs classes and `REUSE.toml` metadata now covers full repository classification for REUSE auditing.
- Root license texts now exist as `LICENSE-MIT`, `LICENSE-APACHE-2.0`, and `LICENSE-CC-BY-4.0`; crate `Cargo.toml` metadata is aligned on `MIT OR Apache-2.0`.

Current boundary update (2026-03-18):
- Public strict verification flow is now `crushr-extract --verify <archive>`.
- `crushr-fsck` is retained only as a temporary compatibility shim that exits with deprecation guidance.
- `crushr-salvage` remains recovery-oriented and separate from canonical extraction verification.
- Reader-boundary hardening tightened legacy permissive behavior: block-region mismatch and decoded raw-length mismatch in `read.rs` now fail closed.
- `crushr-extract --verify` now runs strict extraction semantics in an isolated temp output path to ensure strict-verify alignment.
- `crushr-core` now exposes a canonical typed verification model (`VerificationModel`) and `crushr-extract --verify` output is derived from that model.
- CRUSHR-HARDEN-03H removed remaining verify-output duplicate truth path: CLI-local `VerifyReport` was deleted and verify rendering now consumes `VerificationReportView` projected by the canonical model layer (`VerificationModel::to_report_view`).
- Carried-forward salvage metadata classification clippy failure (`if_same_then_else` in verified-graph classification) has been removed.

Next focus:
- Complete CRUSHR-HARDEN-03G typed metadata conversion in remaining pack/salvage builder paths (remove dynamic `Value` as intermediate truth in active core metadata builders).
- Continue metadata-pruning evidence review using active FORMAT-10/11/12/13/14A outputs once hardening packet is fully closed.

CRUSHR-HARDEN-03I completion update (2026-03-19):
- `crushr-pack` active experimental metadata builders are typed (`Serialize` structs/enums) across self-describing/checkpoint/file-identity/payload-identity/path-checkpoint/manifest/path-dictionary flows.
- `crushr-salvage` active metadata path in `crushr_salvage/core/metadata.rs` now uses typed `ExperimentalMetadataRecord` variants (no active `Vec<Value>` metadata truth path).
- salvage metadata classification/parser helpers for dictionary/path/payload/file-identity/manifest now consume typed records and preserve deterministic fail-closed behavior.
- bootstrap-anchor availability checks were moved from key-based JSON lookups to typed metadata variant checks.
- FIX2 follow-up: removed localized dictionary-copy-v2 `body: serde_json::Value` raw carrier; parser now captures `body_raw_json` via deterministic raw-slice extraction from the metadata block JSON bytes and preserves hash/length parity semantics.

CRUSHR-LAB-FIX-01 completion update (2026-03-19):
- `crushr-lab` Phase 2 contract tests no longer assume checked-in `PHASE2_RESEARCH` artifacts exist at workspace root.
- Comparison shape-contract coverage now uses representative in-test normalized records to validate emitted comparison tables/rankings.
- Normalization shape + ordering tests now create deterministic temporary trials fixtures and validate both schema-shape conformance and canonical `scenario_id` ordering from emitted normalized records.


## CRUSHR-HARDEN-03A handoff
- Removed accidental `crushr::extraction_path` library exposure; path-confinement helpers remain internal implementation detail.
- Added compile-level visibility guard in `crates/crushr/src/lib.rs` (`compile_fail` doctest) to prevent re-expansion of that internal API surface.
- Updated boundary docs (`README`, crate-level docs in `crushr`, `crushr-core`, `crushr-format`) to explicitly classify stable product surfaces vs bounded internal crates/modules vs experimental/lab workflows.
- Verified representative product workflow (`crushr-pack` + `crushr-extract --verify`) and full workspace test suite after boundary tightening.


## CRUSHR-HARDEN-03G handoff
- Added helper builders in `crushr-pack` for self-describing extent, file-identity, payload-identity, checkpoint, and manifest record/snapshot JSON generation.
- `emit_archive_from_layout` now consumes those helpers rather than constructing most experimental metadata records inline.
- Added `build_redundant_file_map` and `write_tail_with_redundant_map` helpers so redundant-map JSON construction and tail assembly are no longer inline in emitter closeout.
- Converted redundant-map closeout to typed structs (`RedundantFileMap*`) before ledger serialization to reduce untyped tail-closeout assembly surface.
- Existing deterministic + experimental writer tests and representative archive-creation commands were rerun and passed.

## CRUSHR-HARDEN-03F handoff
- `crushr-pack` now runs as: input discovery/normalization (`collect_files` + duplicate rejection) → canonical file model construction + metadata profile planning (`build_pack_layout_plan`) → dictionary planning (`build_dictionary_plan`) → final serialization (`emit_archive_from_layout`).
- Experimental profile and dictionary toggles are computed in typed `MetadataPlan`/`DictionaryPlan` data and consumed by the emitter, improving canonical-vs-experimental boundary readability.
- Added pack-stage regressions in `deterministic_pack.rs` for metadata-profile determinism and redundant-map profile recording.
- Remaining coupled area: experimental metadata record JSON assembly is still built inline during emission; move to typed helper builders in next step.

## CRUSHR-HARDEN-03E handoff
- Active comparison summaries now have dedicated schema files under `schemas/` for FORMAT-12/13/14A/15 baseline + stress outputs.
- Integration test `comparison_output_schemas.rs` runs active comparison commands and checks emitted artifacts against required schema fields/version constants.
- Comparison engine is now split into `lab/comparison/mod.rs`, `common.rs`, `experimental.rs`, `format06_to12.rs`, and `format13_to15.rs`.
- Command dispatch in `crushr-lab-salvage` is unchanged; import path now points to `comparison/mod.rs`.
- Remaining concern: format09/10 helper internals still use permissive helper visibility and some untyped `Value` helper flow that should be tightened in follow-up 03F.


## Current near-term priority note

- Repository licensing is now unified (MIT OR Apache-2.0 for code; CC-BY-4.0 for docs) and REUSE-compliant.
- `zensical.toml` is now the canonical docs-site configuration; `mkdocs.yml` should be treated as transitional compatibility only.
- Before benchmark-harness expansion, the intended next product-facing step is a unified CLI presentation contract (`CRUSHR-UI-01`) so pack/extract/verify/salvage share one operator-facing identity and consistent `--silent` behavior.

## CRUSHR_PACK_SCALE_01 handoff
- Production `crushr-pack` planning no longer loads/compresses every input file during `build_pack_layout_plan`; the layout now stores only file identity/path/size + metadata-plan toggles.
- Payload bytes and compression artifacts are now produced per-file inside `emit_archive_from_layout`, reducing retained memory from O(sum(raw+compressed for all files)) toward bounded per-file working-set behavior.
- A new deterministic safety check aborts with `input changed during pack planning` if file length changes between planning and emission (protects index/tail correctness after deferring reads).
- Added unit regressions in `commands::pack` covering unreadable-file planning and change-between-plan/emit detection.
- Measured synthetic scale evidence (20k files): max RSS pre-fix `177248 KiB` (`HEAD~1`) vs post-fix `76556 KiB` (current).
