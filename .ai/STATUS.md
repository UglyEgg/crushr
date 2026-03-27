<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR_CLEANUP_02 complete** (pack preservation-profile authority collapsed into one planning decision layer)


Latest maintenance fix (2026-03-27):
- **CRUSHR_CLEANUP_02 complete**: collapsed pack preservation-profile policy to one canonical planning authority (`plan_pack_profile`) that returns explicit included/omitted outcomes and omission-reason classification.
- **CRUSHR_CLEANUP_02 complete**: discovery is now policy-free for preservation profile handling (`collect_files` captures raw candidates only; no profile-driven omission/warnings).
- **CRUSHR_CLEANUP_02 complete**: emission/finalization now consume authoritative plan outcomes only; profile warnings are emitted once through centralized `emit_profile_warnings` using omission decisions from the canonical plan.
- **CRUSHR_CLEANUP_02 complete**: removed split ownership path (`collect_files(..., profile)` + `apply_preservation_profile`) and replaced it with explicit `PackProfilePlan { included, omitted }` carried by `PackLayoutPlan`.
- **CRUSHR_CLEANUP_02 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-27):
- **CRUSHR_HOSTILE_REVIEW_01 complete**: completed a hostile, enterprise-focused structural review covering pack pipeline, strict/recover extraction split, introspection contract surfaces, preservation-profile semantics, benchmark tooling, and embedded comments/docs.
- **CRUSHR_HOSTILE_REVIEW_01 complete**: published prioritized findings for duplication, layered patching, hidden coupling, stale/misleading contract messaging, and test-structure illusions in `.ai/COMPLETION_NOTES_CRUSHR_HOSTILE_REVIEW_01.md`.
- **CRUSHR_HOSTILE_REVIEW_01 complete**: produced actionable cleanup packet roadmap (`CRUSHR_CLEANUP_01`..`CRUSHR_CLEANUP_06`) with recommended execution order before additional feature expansion.
- **CRUSHR_HOSTILE_REVIEW_01 addendum (2026-03-27)**: refreshed `.ai/COMPLETION_NOTES_CRUSHR_HOSTILE_REVIEW_01.md` with stricter file/function-specific findings and explicit answers to all locked review questions (duplication/layering/source-of-truth/dead-path/test-illusion axes).
- **CRUSHR_HOSTILE_REVIEW_01 complete**: no product runtime behavior, archive semantics, schema contracts, or benchmark outputs were changed in this packet.

Latest maintenance fix (2026-03-27):
- **CRUSHR_OPTIMIZATION_03 complete**: production `pack` now reuses a single `zstd::bulk::Compressor` context for payload and metadata block compression across the run, eliminating per-unit encoder construction/teardown in the hot path.
- **CRUSHR_OPTIMIZATION_03 complete**: deterministic zstd frame flags remain explicitly locked (`checksum=false`, `contentsize=true`, `dictid=false`), compression method remains `zstd`, and level behavior remains unchanged.
- **CRUSHR_OPTIMIZATION_03 complete**: compression output buffer lifecycle is now owned by the reusable compressor (`compress_to_buffer`), reducing compression-path allocation churn while preserving existing hashing/emission/finalization boundaries.
- **CRUSHR_OPTIMIZATION_03 complete**: fail-closed mutation detection, preservation profile semantics, archive validity/extractability expectations, and `--profile-pack` phase truth are unchanged.
- **CRUSHR_OPTIMIZATION_03 complete**: canonical version advanced to `0.4.20` (`VERSION` + workspace package version sync).

Latest maintenance fix (2026-03-27):
- **CRUSHR_OPTIMIZATION_02 complete**: production `pack` now writes through a 1 MiB `BufWriter`, reducing small-write syscall pressure in payload and metadata emission without changing archive layout semantics.
- **CRUSHR_OPTIMIZATION_02 complete**: payload/metadata compression now reuses a per-run compression output buffer, reducing repeated allocation overhead while preserving codec, level, and deterministic zstd flags.
- **CRUSHR_OPTIMIZATION_02 complete**: block offsets used by experimental identity records are now tracked via deterministic emitted-byte accounting (`BLK3_HEADER_WITH_HASHES_LEN`) so profiling remains truthful while buffered writes are enabled.
- **CRUSHR_OPTIMIZATION_02 complete**: fail-closed mutation detection (`input changed during pack planning`), preservation profile behavior, hash work, and tail/index finalization semantics are unchanged.
- **CRUSHR_OPTIMIZATION_02 complete**: canonical version advanced to `0.4.19` (`VERSION` + workspace package version sync).

Latest maintenance fix (2026-03-27):
- **CRUSHR_OPTIMIZATION_01 complete**: production `pack` discovery now gates metadata capture by selected preservation profile, avoiding eager probes for omitted classes in `basic`/`payload-only`.
- **CRUSHR_OPTIMIZATION_01 complete**: removed duplicate per-regular-file planning `stat` overhead by reusing discovery-captured `raw_len` in layout planning.
- **CRUSHR_OPTIMIZATION_01 complete**: discovery now caches ownership-name lookups per UID/GID and skips xattr/security/sparse probes when profile semantics do not require them.
- **CRUSHR_OPTIMIZATION_01 complete**: updated benchmark operator commands to require medium+large full/basic `--profile-pack` runs for direct discovery-phase validation.
- **CRUSHR_OPTIMIZATION_01 complete**: canonical version advanced to `0.4.18` (`VERSION` + workspace package version sync).


Latest maintenance fix (2026-03-26):
- **CRUSHR_BENCHMARK_03 complete**: added production `crushr pack --profile-pack` opt-in profiling output for deterministic phase attribution (`discovery`, `metadata`, `hashing`, `compression`, `emission`, `finalization`).
- **CRUSHR_BENCHMARK_03 complete**: instrumented production pack pipeline phase timing in-process without changing archive bytes, preservation semantics, or default pack CLI noise.
- **CRUSHR_BENCHMARK_03 complete**: added CLI integration coverage ensuring phase output is absent by default and appears only when `--profile-pack` is explicitly requested.
- **CRUSHR_BENCHMARK_03 complete**: updated benchmark methodology docs with exact local attribution commands for `medium_realistic_tree` and `large_stress_tree`, expected output shape, capture guidance, and interpretation hints.
- **CRUSHR_BENCHMARK_03 complete**: canonical version advanced to `0.4.17` (`VERSION` + workspace package version sync).
- **CRUSHR_BENCHMARK_03 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `./scripts/check-version-sync.sh`, and `cargo test -p crushr --test version_contract` are green.


Latest maintenance fix (2026-03-26):
- **CRUSHR_PRESERVATION_FIX_06 complete**: strict/recover extraction metadata restoration now gates restore attempts on the archive-recorded preservation profile, so omitted classes are skipped rather than attempted-and-filtered after warning emission.
- **CRUSHR_PRESERVATION_FIX_06 complete**: omitted-by-profile classes (`ownership`, `xattrs`, `ACLs`, `SELinux labels`, `capabilities`) no longer emit spurious restore warnings in `basic`/`payload-only` extraction paths.
- **CRUSHR_PRESERVATION_FIX_06 complete**: full-profile behavior remains unchanged; required metadata restoration still attempts restore and still reports warning/fail-closed behavior when blocked.
- **CRUSHR_PRESERVATION_FIX_06 complete**: deterministic coverage added for omitted-profile warning suppression in strict+recover, and explicit full-profile ownership warning assertions.
- **CRUSHR_PRESERVATION_FIX_06 complete**: canonical version advanced to `0.4.16` (`VERSION` + workspace package version sync).
- **CRUSHR_PRESERVATION_FIX_06 validation**: `cargo fmt --all`, `cargo test -p crushr --test metadata_preservation`, `cargo test -p crushr --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `./scripts/check-version-sync.sh` are green.


Latest maintenance fix (2026-03-26):
- **CRUSHR_BENCHMARK_02 complete**: executed the full locked benchmark matrix over `small_mixed_tree`, `medium_realistic_tree`, and `large_stress_tree` for `tar+zstd`, `tar+xz`, `crushr --preservation full`, and `crushr --preservation basic`.
- **CRUSHR_BENCHMARK_02 complete**: published canonical raw baseline results at `docs/reference/benchmarks/benchmark_results_v0.4.15.json` and human analysis report at `docs/reference/benchmark-baseline.md`.
- **CRUSHR_BENCHMARK_02 complete**: documented environment context (CPU/RAM/OS/filesystem) and explicit caveats (`peak_rss_kb` unavailable in this environment due missing GNU `/usr/bin/time`).
- **CRUSHR_BENCHMARK_02 complete**: canonical version advanced to `0.4.15` (`VERSION` + workspace package version sync).
- **CRUSHR_BENCHMARK_02 validation**: benchmark harness full run, JSON schema validation (`jsonschema`), `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `./scripts/check-version-sync.sh` are green.

Latest maintenance fix (2026-03-26):
- **CRUSHR_BENCHMARK_01 complete**: added deterministic benchmark dataset generator (`scripts/benchmark/generate_datasets.py`) with three reproducible dataset classes (`small_mixed_tree`, `medium_realistic_tree`, `large_stress_tree`) plus emitted `dataset_manifest.json`.
- **CRUSHR_BENCHMARK_01 complete**: added benchmark harness runner (`scripts/benchmark/run_benchmarks.py`) with explicit command execution for `tar+zstd`, `tar+xz`, `crushr --preservation full`, and `crushr --preservation basic`, plus structured JSON result output.
- **CRUSHR_BENCHMARK_01 complete**: added benchmark contract documentation (`docs/reference/benchmarking.md`) and locked result schema (`schemas/crushr-benchmark-run.v1.schema.json`) for reproducible, attributable measurements.
- **CRUSHR_BENCHMARK_01 complete**: canonical version advanced to `0.4.14` (`VERSION` + workspace package version sync).
- **CRUSHR_BENCHMARK_01 validation**: `cargo build --release -p crushr`, dataset generation + full benchmark suite run over all three datasets, `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `./scripts/check-version-sync.sh` are green.

Latest maintenance fix (2026-03-26):
- **CRUSHR_PACK_STREAMING_01 complete**: removed recurring whole-run payload retention from production `pack` by dropping in-memory raw-byte caching in hard-link payload reuse state.
- **CRUSHR_PACK_STREAMING_01 complete**: file-manifest digest emission now uses already-computed per-block raw BLAKE3 digests instead of retaining payload bytes for later re-hash.
- **CRUSHR_PACK_STREAMING_01 complete**: fail-closed mutation guard remains unchanged (`input changed during pack planning`) and still runs at serialization-time metadata checks.
- **CRUSHR_PACK_STREAMING_01 evidence**: synthetic 250-file (2 MiB each) dataset max RSS dropped from **525,800 KiB** (`HEAD~1`) to **14,400 KiB** (current), with equivalent successful archive emission.
- **CRUSHR_PACK_STREAMING_01 validation**: `cargo fmt --all`, `cargo test -p crushr pack_fails_if_file_changes_between_planning_and_emit -- --nocapture`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo test -p crushr --test version_contract`, and pack/info/verify/extract runtime probes are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_INTROSPECTION_02 complete**: `crushr info` now surfaces preservation profile contract classification (`full-fidelity Linux-first` / `basic Linux metadata` / `content-oriented payload`) and adds concise entry-kind visibility summary (`regular`, `directory`, `symlink`, `hard link`, `sparse`, `FIFO`, `char/block device`).
- **CRUSHR_INTROSPECTION_02 complete**: metadata visibility rows now distinguish `present`, `not present`, and `omitted by profile` so omission intent is not framed as corruption/degradation.
- **CRUSHR_INTROSPECTION_02 complete**: `crushr info --list` now includes profile/scope context while preserving fail-closed metadata/index-only proof behavior; non-regular omission is rendered as informational scope notes instead of warning-level corruption semantics.
- **CRUSHR_INTROSPECTION_02 complete**: updated docs wording to clarify `info` reports archive contract truth while `metadata_degraded` remains an extraction/recovery outcome.
- **CRUSHR_INTROSPECTION_02 complete**: canonical version advanced to `0.4.12` (`VERSION` + workspace package version sync).
- **CRUSHR_INTROSPECTION_02 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test metadata_preservation`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo test -p crushr --test version_contract` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_RECOVERY_MODEL_08 complete**: strict extraction now fail-closes on profile-required metadata restoration failures for non-regular canonical entries (directories, symlinks, FIFOs, char devices, block devices) instead of leaving warning-only canonical gaps.
- **CRUSHR_RECOVERY_MODEL_08 complete**: recover extraction now applies profile-aware metadata-degraded routing/manifest truth consistently across non-regular canonical outputs; non-regular metadata failures no longer remain implicitly canonical.
- **CRUSHR_RECOVERY_MODEL_08 complete**: profile omission semantics remain honest (`basic`/`payload-only` omitted metadata classes are not misclassified as degradation), including non-regular canonical outputs.
- **CRUSHR_RECOVERY_MODEL_08 complete**: deterministic coverage now includes strict refusal + recover metadata_degraded placement/manifest assertions for directory/symlink/FIFO paths and basic-profile omission non-degradation behavior.
- **CRUSHR_RECOVERY_MODEL_08 complete**: canonical version advanced to `0.4.11` (`VERSION` + workspace package version sync).
- **CRUSHR_RECOVERY_MODEL_08 validation**: `cargo fmt --all`, `cargo test -p crushr --test metadata_preservation --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo test -p crushr --test version_contract` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_PRESERVATION_05 complete**: production pack now supports explicit `--preservation <full|basic|payload-only>` (default `full`) with deterministic warn-and-omit behavior for excluded entry kinds; no `--strip` alias was added.
- **CRUSHR_PRESERVATION_05 complete**: production index encoding advanced to IDX7 with structured on-disk preservation-profile recording; legacy IDX3/IDX4/IDX5/IDX6 decode paths default to `full` compatibility semantics.
- **CRUSHR_PRESERVATION_05 complete**: strict/recover metadata-degraded classification is now profile-aware for regular-file canonical outcomes (`basic`/`payload-only` omitted classes no longer misclassify as metadata restoration failure).
- **CRUSHR_PRESERVATION_05 complete**: `crushr info` now renders `Preservation` profile visibility and format-marker truth includes IDX7.
- **CRUSHR_PRESERVATION_05 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract --test index_codec --test metadata_preservation`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_RECOVERY_MODEL_07 complete**: extraction/recovery trust model now includes `metadata_degraded`, and recover output layout now creates `metadata_degraded/` separately from `canonical/` (no silent merge).
- **CRUSHR_RECOVERY_MODEL_07 complete**: strict extract now refuses when required metadata restoration fails and surfaces explicit metadata-failure cause messaging instead of treating those outcomes as canonical success.
- **CRUSHR_RECOVERY_MODEL_07 complete**: recovery manifest contract now includes `trust_class`, `missing_metadata_classes`, `failed_metadata_classes`, and `degradation_reason`, with `metadata_degraded` explicitly represented in schema + manifest output.
- **CRUSHR_RECOVERY_MODEL_07 complete**: recover CLI summary/trust-class presentation now includes `metadata_degraded`, and result rows align to `canonical / metadata_degraded / recovered_named / anonymous / unrecoverable`.
- **CRUSHR_RECOVERY_MODEL_07 known coverage limit**: metadata-degraded routing/classification is currently complete for regular-file canonical outputs; directories, symlinks, and special entries still use warning-based metadata restore behavior and are not yet fully routed through metadata-degraded placement/classification.
- **CRUSHR_RECOVERY_MODEL_07 validation**: `cargo fmt --all`, `cargo test -p crushr --test recovery_extract_contract --test metadata_preservation`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_PRESERVATION_04 complete**: production index encoding advanced to IDX6 with explicit fields for POSIX ACL access/default metadata, SELinux label metadata, and Linux capability metadata.
- **CRUSHR_PRESERVATION_04 complete**: pack now captures ACL/SELinux/capability xattrs as structured security metadata (without silently dropping supported Linux metadata classes), and strict/recover extraction restores them best-effort with explicit `WARNING[acl-restore]`, `WARNING[selinux-restore]`, and `WARNING[capability-restore]` when blocked.
- **CRUSHR_PRESERVATION_04 complete**: `crushr info` metadata visibility now includes `ACLs`, `SELinux labels`, and `capabilities`, and format marker reflects IDX3/IDX4/IDX5/IDX6 truth.
- **CRUSHR_PRESERVATION_04 manual validation**: exact operator commands + observed outcomes for ACL/SELinux/capability/info/degraded/backward-compat/recovery-model checks are recorded at `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_04.md`.
- **CRUSHR_PRESERVATION_04 validation**: `cargo fmt --all`, `cargo test -p crushr --test index_codec --test metadata_preservation`, `cargo test -p crushr --test deterministic_pack --test mvp --test cli_presentation_contract`, and `cargo clippy --workspace --all-targets -- -D warnings` are green.


Latest maintenance fix (2026-03-25):
- **CRUSHR_PRESERVATION_03 complete**: production index encoding advanced to IDX5 with sparse regular-file mapping (`logical_offset` extents), FIFO/character/block device entry kinds, optional device major/minor metadata, and ownership-name enrichment capture where available.
- **CRUSHR_PRESERVATION_03 complete**: strict + recover extraction now restore sparse files hole-aware, recreate FIFOs/device nodes where permitted, and emit explicit `WARNING[special-restore]` degradation when platform/privilege blocks special-file restoration.
- **CRUSHR_PRESERVATION_03 complete**: `crushr info` metadata presence now includes `sparse files` and `special files`, and format marker reflects IDX3/IDX4/IDX5 truth.
- **CRUSHR_PRESERVATION_03 complete**: operator-level manual validation evidence with exact commands/results is recorded at `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_03.md`.
- **CRUSHR_PRESERVATION_03 validation**: `cargo fmt --all`, `cargo test -p crushr --test metadata_preservation`, `cargo test -p crushr --test deterministic_pack --test mvp --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_PRESERVATION_02 complete**: production pack/index now preserves uid/gid ownership metadata and hard-link group identity in IDX4 while keeping Linux-first mode/mtime/xattr behavior.
- **CRUSHR_PRESERVATION_02 complete**: strict + recover extraction paths now restore ownership best-effort with explicit warnings on permission/platform failures and recreate hard links from preserved link groups.
- **CRUSHR_PRESERVATION_02 complete**: `crushr info` now reports metadata presence classes (`modes`, `mtime`, `xattrs`, `ownership`, `hard links`) and format marker reflects IDX3/IDX4 truth.
- **CRUSHR_PRESERVATION_02 validation**: `cargo fmt --all`, `cargo test -p crushr --test deterministic_pack --test mvp --test metadata_preservation --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -q` are green.

Latest maintenance fix (2026-03-25):
- **CRUSHR_PRESERVATION_02-FIX1 complete**: corrected `info` ownership presence detection for IDX4 archives so root-owned (`uid=0,gid=0`) metadata is not falsely reported absent.
- **CRUSHR_PRESERVATION_02-FIX1 complete**: added explicit packet completion evidence at `.ai/COMPLETION_NOTES_CRUSHR_PRESERVATION_02.md` with exact commands + observed outputs for all addendum validation cases.


Latest maintenance fix (2026-03-24):
- **CRUSHR_PRESERVATION_01 complete**: production `pack` now captures baseline Linux-first metadata (`directory`/`symlink` entries, mode, mtime, and xattrs) and stores it in IDX3 using explicit entry kinds.
- **CRUSHR_PRESERVATION_01 complete**: strict and recover extraction paths now materialize directories/symlinks and restore mode/mtime/xattrs with explicit warning surfacing when xattrs cannot be restored.
- **CRUSHR_PRESERVATION_01 validation**: `cargo fmt --all`, `cargo test -p crushr --test deterministic_pack --test mvp --test metadata_preservation`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.




Latest maintenance fix (2026-03-24):
- **CRUSHR_INTROSPECTION_01-FIX2 complete**: omission-only list cases now remain `COMPLETE` and are surfaced as informational notes, not structural degradation.
- **CRUSHR_INTROSPECTION_01-FIX2 complete**: `omitted entries` result row is shown only when non-zero; degraded structural proof failures still show salvage guidance.

Latest maintenance fix (2026-03-24):
- **CRUSHR_INTROSPECTION_01-FIX1 complete**: `info --list` now reports omitted non-regular index entries explicitly and keeps regular-file listing semantics transparent.
- **CRUSHR_INTROSPECTION_01-FIX1 complete**: degraded listing-proof warnings now include explicit `crushr salvage <archive>` guidance while preserving fail-closed no-guess behavior.
- **CRUSHR_INTROSPECTION_01-FIX1 complete**: canonical version advanced to `0.4.1` (`VERSION` + workspace package version sync).

Latest maintenance fix (2026-03-24):
- **CRUSHR_INTROSPECTION_01 complete**: added `crushr info --list` archive introspection path (tree default + `--flat`) driven strictly from metadata/index (`IDX3`) without extraction.
- **CRUSHR_INTROSPECTION_01 complete**: added corruption-aware listing fallback that shows only provable index-backed paths and degrades with explicit warnings when listing proof is unavailable.
- **CRUSHR_INTROSPECTION_01 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-24):
- **CRUSHR_UI_POLISH_08 complete**: confirmed `pack` progress retains stable, separate `compression` + `serialization` rows with shared active-phase rendering and explicit `finalizing` phase transition after both rows settle.
- **CRUSHR_UI_POLISH_08 complete**: updated `info` Structure terminology to user-truthful file-level labels (`files`, `compressed units`, `file mappings`) and added explicit `block model` line (`file-level (1:1 file → unit)`) without changing index/block calculations.
- **CRUSHR_UI_POLISH_08 complete**: synchronized v0.3.x version target back to `0.3.5` (`VERSION` + workspace package version sync) and refreshed info presentation golden output.
- **CRUSHR_UI_POLISH_08 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.


Latest maintenance fix (2026-03-24):
- **CRUSHR_UI_POLISH_07 maintenance**: temporary version bump to `0.3.7` was superseded by CRUSHR_UI_POLISH_08 packet alignment back to `0.3.5` via canonical version workflow (`VERSION` + workspace package version sync).
- **CRUSHR_UI_POLISH_07 maintenance validation**: `./scripts/check-version-sync.sh` and `cargo test -p crushr --test version_contract` are green.

Latest maintenance fix (2026-03-24):
- **CRUSHR_UI_POLISH_07 complete**: colorized help surfaces (`crushr`, `crushr-pack`, `crushr-extract`, `crushr-info`) now route through shared CLI presentation tokens/sections instead of ad hoc plain help text.
- **CRUSHR_UI_POLISH_07 complete**: standardized pack output extension behavior to append `.crs` when `-o/--output` has no extension while preserving explicit user-provided extensions.
- **CRUSHR_UI_POLISH_07 complete**: corrected pack progress truth to show explicit `compression` and `serialization` phases reaching `N/N`, followed by visible `finalizing`; added result metrics (`archive`, size totals, compression ratio, reduction, processing time).
- **CRUSHR_UI_POLISH_07 complete**: expanded `info` human output with a dedicated `Compression` section exposing method + level from parsed BLK3 headers (with unavailable fallback when data is missing).
- **CRUSHR_UI_POLISH_07 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test cli_contract_surface`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-24):
- **CRUSHR_UI_POLISH_06 complete**: standardized shared CLI title rendering with a leading blank line, canonical double-line divider, and color-safe key/value alignment that remains consistent with ANSI color enabled.
- **CRUSHR_UI_POLISH_06 complete**: rebuilt `about` around shared presentation semantics (colorized title/sections/labels, canonical divider, aligned columns) so it matches the rest of the product CLI surface.
- **CRUSHR_UI_POLISH_06 complete**: upgraded `info` human output from sparse/internal fields to product-facing structural inspection rows (file/extents/logical bytes/payload blocks/dictionary presence/compression level), replacing raw `has dct1` jargon with dictionary language and marking compression as unavailable when not recoverable.
- **CRUSHR_UI_POLISH_06 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.


Latest maintenance fix (2026-03-24):
- **CRUSHR_UI_POLISH_04 complete**: refined shared motion application on `pack` by reporting live serialization file progress via shared active-phase details and settling to stable final file-count output.
- **CRUSHR_UI_POLISH_04 complete**: added explicit non-TTY cleanliness contract checks ensuring `pack`, `verify`, `extract`, and `extract --recover` outputs contain no spinner carriage-control artifacts even when motion mode is set to `full`.
- **CRUSHR_UI_POLISH_04 complete**: refreshed pack progress golden output to lock the stabilized serialization row and reran fmt/focused integration/workspace validation.


Latest maintenance fix (2026-03-23):
- **CRUSHR_UI_POLISH_03 complete**: added shared active-phase motion primitives in `cli_presentation` (`begin_active_phase` / `ActivePhase`) with centralized animation state, bounded redraw cadence, and stable phase freeze behavior on completion/failure.
- **CRUSHR_UI_POLISH_03 complete**: applied shared motion/state transitions to core long-running progress surfaces (`pack`, `extract`, `extract --recover`, `verify`) and kept `info` static; progress rows now settle as stable phase outcomes in non-interactive output.
- **CRUSHR_UI_POLISH_03 complete**: formalized motion policy contract in `.ai/contracts/CLI_MOTION_POLICY.md` including anti-goals, redraw rates, no-motion controls, and non-TTY guarantees; refreshed progress goldens accordingly.
- **CRUSHR_UI_POLISH_03 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_UI_POLISH_01 complete**: replaced ad hoc CLI status/color handling with a shared semantic token layer (`VisualToken`) in `cli_presentation`, including required title/section/label/muted/running/pending/success/degraded/failure/info and trust-class tokens.
- **CRUSHR_UI_POLISH_01 complete**: standardized user-facing status semantics around `PENDING`, `RUNNING`, `COMPLETE`, `DEGRADED`, `FAILED`, and `REFUSED` (with bounded compatibility/status aliases retained for existing call sites), and mapped prior `PARTIAL` presentation to `DEGRADED`.
- **CRUSHR_UI_POLISH_01 complete**: updated recover-mode output to surface an explicit `Trust classes` section (`CANONICAL`, `RECOVERED_NAMED`, `RECOVERED_ANONYMOUS`, `UNRECOVERABLE`) and added a formal contract doc at `.ai/contracts/CLI_VISUAL_SEMANTICS.md`.
- **CRUSHR_UI_POLISH_01 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract --test recovery_extract_contract`, `cargo test -p crushr --test recovery_validation_corpus`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_06 complete**: zip-family high-confidence promotion now requires root OOXML relationship marker (`_rels/.rels`) in addition to prior content markers, preventing docx/xlsx/pptx over-promotion from generic zip-like payloads.
- **CRUSHR_RECOVERY_MODEL_06 complete**: added deterministic naming-collision guard test for repeated same-payload classification IDs to lock unique anonymous naming progression.
- **CRUSHR_RECOVERY_MODEL_06 complete**: strengthened clean recover-mode contract assertions so clean archives produce zero recovered artifacts (no files under `recovered_named/` or `_crushr_recovery/anonymous/`) and still emit empty manifest entries.
- **CRUSHR_RECOVERY_MODEL_06 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr recovery_classification::tests`, `cargo test -p crushr --test recovery_extract_contract`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_05 complete**: moved recover-mode progress emission onto real execution phases (`archive open`, `metadata scan`, `canonical extraction`, `recovery analysis`, `recovery extraction`, `manifest/report finalization`) so phase updates are emitted incrementally during `extract --recover`.
- **CRUSHR_RECOVERY_MODEL_05 complete**: refined recover final summary to use trust-class-aligned count labels (`recovered_named`, `recovered_anonymous`) plus separate `Extraction status` rows for canonical vs recovery completeness.
- **CRUSHR_RECOVERY_MODEL_05 complete**: added precise conditional notes: clean archives do not emit non-canonical warnings, while damaged/mixed recoveries explicitly report non-canonical placement and surface `_crushr_recovery/manifest.json`.
- **CRUSHR_RECOVERY_MODEL_05 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_04 complete**: added deterministic end-to-end recovery corpus test (`recovery_validation_corpus`) that generates mixed fixture trees (structured text, binary signatures, office-container markers, nested/repeated paths, empty directory) and validates strict/recover behavior under clean and damaged archives.
- **CRUSHR_RECOVERY_MODEL_04 complete**: added deterministic corruption operations (tail truncation, index metadata mutation, block payload-hash bit flip, compressed-payload clobbering) and scenario assertions covering canonical, recovered_named, recovered_anonymous (high/medium/low naming tiers), and unrecoverable outcomes in one archive.
- **CRUSHR_RECOVERY_MODEL_04 complete**: added corpus technical note (`RECOVERY_VALIDATION_CORPUS.md`) and validated manifest truth against emitted outputs (assigned names, trust classes, classification fields, identity status, recoverable/unrecoverable semantics).
- **CRUSHR_RECOVERY_MODEL_04 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_validation_corpus`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_03 complete**: added a modular, data-driven recovery classification engine (`recovery_classification`) with ordered detection pipeline (magic -> secondary header/structure checks -> confidence assignment) and broad coverage across document/archive/media/binary/system signatures.
- **CRUSHR_RECOVERY_MODEL_03 complete**: recover manifest entries now separate trust class (`recovery_kind`) from content typing (`classification.kind/confidence/basis/subtype`) and anonymous naming policy now strictly follows high/medium/low tiered naming.
- **CRUSHR_RECOVERY_MODEL_03 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, and `cargo test --workspace` are green.
Immediate Next Step: execute CRUSHR_CLEANUP_03 to deduplicate recover metadata-degraded routing after pack profile-authority cleanup completion.


Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_02 complete**: wired `extract --recover` to run salvage-plan analysis during extract execution and reused salvage engine planning (`build_recovery_analysis`) rather than adding a second recovery-planning implementation path.
- **CRUSHR_RECOVERY_MODEL_02 complete**: recover output now reports required phased progress (`archive open`, `metadata scan`, `canonical extraction`, `recovery analysis`, `recovery extraction`, `finalization`) and emits the required Result/Trust summary rows (`canonical files`, `named recovered`, `anonymous recovered`, `unrecoverable`; canonical/recovery trust COMPLETE|PARTIAL).
- **CRUSHR_RECOVERY_MODEL_02 complete**: recover extraction now emits named recovered files when full bytes are recoverable under untrusted identity, keeps anonymous fallback for partial recovery, and keeps recovery manifest classifications aligned (`recovered_named`, `recovered_anonymous`, `unrecoverable`).
- **CRUSHR_RECOVERY_MODEL_02 validation**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p crushr --test recovery_extract_contract`, and `cargo test --workspace` are green.

Latest maintenance fix (2026-03-23):
- **CRUSHR_RECOVERY_MODEL_01 complete**: added `crushr-extract --recover` mode that keeps strict default extraction unchanged while emitting recovery-aware outputs under `canonical/`, `recovered_named/`, and `_crushr_recovery/{manifest.json,anonymous/}`.
- **CRUSHR_RECOVERY_MODEL_01 complete**: integrated trust classifications (`canonical`, `recovered_named`, `recovered_anonymous`, `unrecoverable`) and locked anonymous naming policy (`file_<id>.<ext>`, `file_<id>.probable-<type>.bin`, `file_<id>.bin`).
- **CRUSHR_RECOVERY_MODEL_01 complete**: introduced `crushr-recovery-manifest.v1` schema + generator with classification/original-identity/reason fields and added integration coverage for clean + damaged recover runs.
- **CRUSHR_RECOVERY_MODEL_01-FIX1 complete**: resolved formatting drift in `recovery_extract_contract.rs` so `cargo fmt --check` is green again without behavioral changes.

Latest maintenance fix (2026-03-23):
- **CRUSHR_VERIFY_SCALE_01 complete**: removed verify-path temp-directory extraction/materialization by adding a verify-only strict pass that validates extents/decompression without writing extracted files.
- **CRUSHR_VERIFY_SCALE_01 complete**: eliminated strict-extract decompressed block payload cache retention/cloning from active extraction paths, reducing whole-run payload residency pressure.
- **CRUSHR_VERIFY_SCALE_01 complete**: added production verify progress stages (`archive open / header read`, `metadata/index scan`, `payload verification`, `manifest validation`, `final result/report`) with deterministic output coverage in `cli_presentation_contract` and golden fixtures.
- **CRUSHR_VERIFY_SCALE_01 evidence**: large synthetic verify run (`12,000` files archive) now completes successfully with visible phase output and no OOM termination in this environment.

Latest maintenance fix (2026-03-22):
- **CRUSHR_PACK_SCALE_01 complete**: removed whole-run payload pre-materialization in `crushr-pack` planning by replacing `PackLayoutPlan` file payload storage with lightweight file descriptors (`abs_path`, logical path, planned size/id), so planning no longer retains raw+compressed bytes for every file simultaneously.
- **CRUSHR_PACK_SCALE_01 complete**: moved file read/compress/hash work to the serialization loop with a deterministic guard that fails closed if an input file changes between planning and emit.
- **CRUSHR_PACK_SCALE_01 complete**: added pack-stage regressions for planning-on-unreadable-files and mutation-between-planning-and-emit detection; workspace fmt/clippy/tests remain green.
- **CRUSHR_PACK_SCALE_01 evidence**: synthetic 20,000-file dataset max RSS dropped from **177,248 KiB** (pre-fix `HEAD~1`) to **76,556 KiB** (current), with identical pack success semantics.

Latest maintenance fix (2026-03-22):
- **CRUSHR_CLI_UNIFY_04 complete**: pruned production `crushr-pack` parser/help surface to production flags only (`inputs`, `-o/--output`, `--level`, shared `--silent`), and removed acceptance of experimental format/layout/profile flags from the public pack path.
- **CRUSHR_CLI_UNIFY_04 complete**: added lab-owned experimental pack entrypoint (`crushr lab pack-experimental`) and rewired lab comparison harness pack invocations to run through lab-owned experimental surface instead of production `crushr-pack` flags.
- **CRUSHR_CLI_UNIFY_04 complete**: updated deterministic/resilience CLI tests to assert production pack flag rejection and lab experimental-flag acceptance; full workspace clippy/tests are green after boundary pruning.

Latest maintenance fix (2026-03-22):
- **CRUSHR_CLI_UNIFY_03 complete**: added dedicated CLI contract integration coverage (`crates/crushr/tests/cli_contract_surface.rs`) to lock canonical command taxonomy, wrapper/canonical help+about+version equivalence, legacy alias rejection, root exit-code behavior, and `--json`+`--silent` shared-flag consistency.
- **CRUSHR_CLI_UNIFY_03 complete**: removed remaining undocumented argument-position aliases by making help/version controls first-argument-only across wrapper and command dispatch paths (`wrapper_cli`, `pack`, `extract`, `info`, `salvage`).
- **CRUSHR_CLI_UNIFY_03 complete**: synchronized public CLI docs with actual behavior (wrapper control-position contract + `crushr-info` usage text reflects optional `--json`).

Latest maintenance fix (2026-03-22):
- **CRUSHR_CLI_UNIFY_02 complete**: moved `crushr-salvage` runtime into shared command module (`crushr::commands::salvage`) and rewired `crushr` top-level `salvage` command to run in-process through shared dispatch.
- **CRUSHR_CLI_UNIFY_02 complete**: converted retained wrappers (`crushr-pack`, `crushr-extract`, `crushr-info`, `crushr-salvage`) to a common wrapper entry helper with uniform wrapper mechanics (`--help`, `--version`, `about`) and explicit canonical-equivalent guidance.
- **CRUSHR_CLI_UNIFY_02 complete**: restricted `crates/crushr` explicit bin targets to retained surface plus required internal research harness binary (`crushr-lab-salvage`) and removed deprecated `crushr-fsck` compatibility binary target from build outputs.

Latest maintenance fix (2026-03-22):
- **CRUSHR_CLI_UNIFY_01 complete**: extracted canonical command entrypoints into shared library modules (`crushr::commands::{pack,extract,info}`), rewired `crushr` root CLI to a canonical command model/dispatcher (`cli_app`) and removed top-level companion-binary process dispatch for canonical product commands.
- **CRUSHR_CLI_UNIFY_01 complete**: promoted `crushr-lab` to expose a library dispatch entrypoint and wired top-level `crushr lab` to run in-process through crate dependency (`crushr-lab`).
- **CRUSHR_CLI_UNIFY_01 complete**: removed obsolete `crushr-cli-common` crate from the workspace and updated architecture docs to reflect the new shared command-host boundary.



Latest maintenance fix (2026-03-22):
- **CRUSHR-STYLE-FIX-01 complete**: performed a repo-wide Clippy style sweep and resolved all surfaced warning classes (primarily `collapsible_if` let-chain rewrites) across workspace crates, binaries, tests, and build/support code.
- **CRUSHR-STYLE-FIX-01 complete**: reran required style gates; `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` both pass, restoring a clean enforced baseline.


Latest maintenance fix (2026-03-21):
- **CRUSHR-CHECK-02-FIX1 complete**: reverted `.github/SECURITY.md` addition per review direction.
- **CRUSHR-CHECK-02-FIX1 complete**: applied repository formatting cleanup (`cargo fmt`) so policy-gate style checks are green (`cargo fmt --check`, clippy, and full workspace tests pass).

Latest maintenance fix (2026-03-21):
- **CRUSHR-CHECK-02 complete**: replaced scattered security/governance checks with a unified GitHub Actions policy gate workflow (`policy-gate`) covering secrets (TruffleHog verified findings), dependency advisories (`cargo audit --deny warnings`), MSRV check on Rust 1.85.0, crate-policy drift guard, clippy (`-D warnings`), fmt check, and VERSION drift validation.
- **CRUSHR-CHECK-02 complete**: added `.cargo/audit.toml` with explicit temporary ignore for `RUSTSEC-2025-0119` (`number_prefix` transitively via `indicatif`) so audit remains fail-closed for other warnings/advisories while keeping the exception visible.
- **CRUSHR-CHECK-02 complete**: aligned README badge row to machine-backed workflows (`policy-gate`, `reuse`).
- **CRUSHR-CHECK-02 note**: style enforcement remains fail-closed in CI and now passes after formatting cleanup.

Latest maintenance fix (2026-03-21):
- **CRUSHR-CRATE-01 complete**: locked workspace MSRV policy to `rust-version = "1.85"` while retaining `edition = "2024"` and `resolver = "3"` at the workspace root.
- **CRUSHR-CRATE-01 complete**: normalized crate metadata policy by inheriting crates.io-facing fields from `[workspace.package]`, requiring `rust-version.workspace = true` on members, and setting explicit non-publish intent (`publish = false`) for internal crates (`crushr-cli-common`, `crushr-lab`, `crushr-tui`).
- **CRUSHR-CRATE-01 complete**: added `scripts/check-crate-policy.sh` to fail closed on missing `package.name`, metadata inheritance drift, publish-intent ambiguity, and workspace policy drift (`resolver`/`edition`/`rust-version`).

Latest maintenance fix (2026-03-20):
- **CRUSHR-BUILD-01 complete**: added repo-root `Containerfile.musl`, `.cargo/config.toml` musl static flags, and `scripts/build-musl-release-podman.sh` for reproducible Podman/Alpine release builds with checksum emission (`SHA256SUMS`, optional `B3SUMS`).
- **CRUSHR-BUILD-01 complete**: updated `crates/crushr/build.rs` to prefer environment-injected release metadata (`CRUSHR_VERSION`, `CRUSHR_GIT_COMMIT`, `CRUSHR_BUILD_TIMESTAMP`, `CRUSHR_TARGET_TRIPLE`, `CRUSHR_RUSTC_VERSION`) and only shell out as bounded fallback before final `unknown`.
- **CRUSHR-BUILD-01 complete**: `crushr about` build metadata now consumes the new runtime constants (`CRUSHR_*`) with existing locked wording/layout unchanged.

Latest maintenance fix (2026-03-20):
- **CRUSHR-UI-04 complete**: added top-level `crushr about` command with locked minimalist section order/wording and UTF-8 divider contract.
- **CRUSHR-UI-04 complete**: wired build metadata injection (`version`, `commit`, `built`, `target`, `rust`) via compile-time build script env values with explicit `unknown` fallback behavior.
- **CRUSHR-UI-04 complete**: added deterministic output guards (golden about output + fallback + help-surface assertions) to prevent presentation drift and preserve present-state-only product wording.

Latest maintenance fix (2026-03-20):
- **CRUSHR-VERSION-01 complete**: added root canonical `VERSION` (`0.2.2`) as strict SemVer source of truth and replaced runtime/tool metadata version reads with `crushr::product_version()` so active `--version`/JSON/report tool-version surfaces no longer depend on hardcoded strings.
- **CRUSHR-VERSION-01 complete**: added deterministic SemVer+drift guardrails via `scripts/check-version-sync.sh` and integration test `crates/crushr/tests/version_contract.rs` asserting `VERSION` == `workspace.package.version` and strict SemVer validity.
- **CRUSHR-VERSION-01 complete**: added `scripts/sync-version.sh` for single-touch version bumps (humans edit `VERSION`, tooling propagates Cargo workspace version), and aligned dev packaging helper `cargo_version()` to read root `VERSION`.

Latest maintenance fix (2026-03-20):
- **CRUSHR-UI-03 complete**: replaced shared CLI presenter output with minimalist section-based layout (`<tool>  /  <action>`, horizontal rule, aligned label/value rows) and explicit per-command canonical section flow ending in `Result`.
- **CRUSHR-UI-03 complete**: switched `crushr-info` to human-readable section output by default while preserving existing JSON envelope output behind `--json`.
- **CRUSHR-UI-03 complete**: strict verify refusal/error paths now emit structured `Failure domain` fields (`component/reason/expected/received`) and never print raw parser internals in normal operator output.
- **CRUSHR-UI-03 complete**: added deterministic golden-output coverage for verify success/failure, pack, info human mode, and salvage in `cli_presentation_contract`.

Latest maintenance fix (2026-03-20):
- **CRUSHR-UI-02 complete**: rewired top-level `crushr` into a focused dispatcher aligned to `pack/extract/verify/info` plus bounded `salvage/lab`, removed legacy command exposure (`append/list/cat/dict-train/tune/completions`) from primary help/surface, and added help-surface + verify failure-path tests to lock identity drift.
- **CRUSHR-UI-02 complete**: strict verify structural failures in `crushr-extract --verify` now render deterministic operator-facing refusal output (with failure-domain section and bounded refusal reason) instead of leaking raw parser internals to normal users.
- **CRUSHR-UI-01-FIX1 complete**: repaired workspace manifest validity by restoring missing `package.name` across all workspace crate manifests, unblocked `cargo fmt --all`, reran targeted UI contract tests, executed representative pack/extract/verify/salvage + `--silent` runtime validation commands, and finalized salvage output mode policy as default human with explicit `--json` for machine output.
- **CRUSHR-UI-01 complete**: added shared CLI presentation helper (`cli_presentation`) with bounded status vocabulary and deterministic section/header/outcome grammar; wired `crushr-pack`, `crushr-extract`, `crushr-extract --verify`, and `crushr-salvage` to the shared surface; standardized `--silent` one-line scriptable summaries across those commands; added integration tests for determinism/status vocabulary/silent behavior.
- **CRUSHR-LICENSE-01-FIX1 complete**: replaced deprecated `.reuse/dep5` with `REUSE.toml` to remove REUSE tooling deprecation warnings while preserving the same license mapping model and passing `reuse lint`.
- **CRUSHR-LICENSE-01 complete**: unified repository licensing to MIT OR Apache-2.0 for code and CC-BY-4.0 for docs/diagrams; aligned workspace crate metadata, added root license texts, applied SPDX headers repo-wide, and verified REUSE compliance via `reuse lint`.

Latest maintenance fix (2026-03-19):
- **CRUSHR-LAB-FIX-01 complete**: repaired Phase 2 lab comparison/normalization contract tests so they no longer depend on missing workspace artifacts and instead generate representative deterministic fixtures in-test.
- Normalization ordering contract is now explicitly enforced through a dedicated scenario-id sort helper used by `normalize_from_trials`.

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` default mode remains strict for canonical extraction behavior, with explicit recovery-aware extraction via `--recover`; strict verification remains `crushr-extract --verify`.
- Current experimental evidence says payload-adjacent file identity is the first major recovery direction that materially improved outcomes.
- The architectural direction remains locked toward a **content-addressed recovery graph**.
- The inversion principle remains active for resilience work: prefer verified payload-adjacent truth over centralized metadata authority.
- FORMAT-06 and FORMAT-07 stabilized classification/confidence without changing headline recovery counts in the current bounded corpus.
- FORMAT-08 now allows bounded comparison of metadata placement strategies (`fixed_spread`, `hash_spread`, `golden_spread`) for graph-supporting metadata checkpoints.
- FORMAT-09 added an expanded corruption matrix (metadata regime × metadata target × payload topology) and emitted `format09_comparison_summary.{json,md}` with survivability/gain metrics.
- FORMAT-10 now adds explicit metadata-pruning variants and emits `format10_comparison_summary.{json,md}` including recovery outcomes, classification counts, and archive-size overhead deltas versus `payload_only`.
- FORMAT-11 adds `extent_identity_only` (distributed per-extent identity via payload-block identity records; no local path/name fields) and emits `format11_comparison_summary.{json,md}` with recovery/size deltas vs `payload_plus_manifest`.
- FORMAT-12 adds `extent_identity_inline_path` (inline verified `name`/`path`/`path_digest` embedded in each payload identity record) and `extent_identity_distributed_names` (distributed checkpoint naming), and emits `format12_comparison_summary.{json,md}` for naming-gain vs size-cost evidence.
- FORMAT-12 stress packet (`CRUSHR-FORMAT-12-STRESS`) adds `run-format12-stress-comparison` and emits `format12_stress_comparison_summary.{json,md}` over deterministic `deep_paths`, `long_names`, `fragmentation_heavy`, and `mixed_worst_case` datasets, including overhead/path/extent metrics and explicit evaluation answers.
- FORMAT-13 adds `extent_identity_path_dict_single`, `extent_identity_path_dict_header_tail`, and `extent_identity_path_dict_quasi_uniform`, plus `run-format13-comparison` and `run-format13-stress-comparison` with artifacts `format13_comparison_summary.{json,md}` and `format13_stress_comparison_summary.{json,md}`.
- FORMAT-14A adds direct dictionary-target corruption scenarios (`primary_dictionary`, `mirrored_dictionary`, `both_dictionaries`, `inconsistent_dictionaries`) and new commands `run-format14a-dictionary-resilience-comparison` / `run-format14a-dictionary-resilience-stress-comparison` with artifacts in `FORMAT14A_RESULTS/`.

- CRUSHR-HARDEN-03B reconciled salvage-plan v3 output semantics: `mapping_provenance` + `recovery_classification` now emit schema-v3 enums, and reason-code arrays (`content_verification_reasons`, `failure_reasons`) are closed + schema-enforced.
- CRUSHR-HARDEN-03D completed strict reader-boundary hardening:
  - canonical verification now executes strict extraction semantics in a temporary sink via `crushr-extract --verify`, preventing permissive read-path leakage
  - legacy reader best-effort behavior was tightened (`scan_blocks` footer-boundary mismatch and block raw-length mismatch now hard-fail)
  - active public/control docs were aligned on the locked tool surface (`crushr-extract --verify`; `crushr-fsck` retired/deprecated shim only)
- CRUSHR-HARDEN-03E decomposed `crushr-lab-salvage` comparison engine into responsibility modules under `lab/comparison/` (`common`, `experimental`, `format06_to12`, `format13_to15`) with top-level command dispatch preserved through `comparison/mod.rs` and stable command wiring.
- CRUSHR-HARDEN-03F decomposed `crushr-pack` around explicit pipeline stages (`collect_files`/duplicate rejection, `build_pack_layout_plan`, `build_dictionary_plan`, `emit_archive_from_layout`) and separated layout planning from low-level byte emission.
- CRUSHR-HARDEN-03F isolated dictionary construction into a bounded builder stage (`DictionaryPlan`) and kept experimental profile toggles in a typed `MetadataPlan` surface consumed by the emitter.
- CRUSHR-HARDEN-03F added focused writer regressions for metadata-profile determinism and redundant-map profile recording while preserving existing canonical/experimental pack behavior.
- CRUSHR-HARDEN-03G extracted experimental metadata JSON construction into dedicated helper builders (`build_*record` / `build_*snapshot` helpers), reducing in-loop JSON assembly coupling inside `emit_archive_from_layout` while preserving semantics.
- CRUSHR-HARDEN-03G follow-up completed redundant-file-map/tail closeout extraction into bounded helpers (`build_redundant_file_map`, `write_tail_with_redundant_map`).
- CRUSHR-HARDEN-03G follow-up also typed the redundant-file-map closeout model (`RedundantFileMap`, `RedundantFileMapFile`, `RedundantFileMapExtent`) so tail ledger assembly no longer builds that structure via ad-hoc `serde_json::Value`.
- CRUSHR-HARDEN-03A finalized API-boundary truth for the current hardened runtime:
  - removed accidental public `crushr::extraction_path` exposure and kept confinement helpers internal-only
  - added compile-level visibility guard via `compile_fail` doctest in `crushr/src/lib.rs`
  - updated README/crate docs to classify stable product vs bounded internal vs experimental/lab surfaces
  - retained explicit stable-facing library surfaces (`crushr::format`, `crushr::index_codec`) used by tool binaries/tests.
- Rendering and emission remain separated from salvage metric derivation paths for typed summary commands (redundant/externalized grouped comparisons), and schema-backed comparison artifact checks remain active.
- CRUSHR-HARDEN-03G follow-on hardening added a canonical typed verification model (`VerificationModel`) in `crushr-core`; `crushr-extract --verify` now derives output/report fields from that model instead of assembling verify truth directly from raw extraction internals.
- CRUSHR-HARDEN-03G carry-forward salvage classification lint (`if_same_then_else`) in verified-graph classification was removed by collapsing redundant branching to a single deterministic orphan classification return path.
- CRUSHR-HARDEN-03H completed verification-truth boundary enforcement:
  - removed CLI-local duplicate verify summary/output truth (`VerifyReport`) from `crushr-extract`
  - added canonical model-owned render projection (`VerificationReportView`) in `crushr-core::verification_model`
  - moved refusal-reason label mapping to canonical model boundary (`to_report_view`) so verify output no longer keeps parallel classification/summary assembly paths in the output layer
  - reran deterministic verify output check twice on the same archive and confirmed byte-for-byte identical JSON output
- CRUSHR-HARDEN-03I partial progress landed:
  - `crushr-pack` experimental metadata writers now build typed structs/enums (self-describing records, file-identity records, payload-identity records, checkpoints, manifests, and dictionary-copy bodies) and serialize only at the write boundary.
  - `write_experimental_metadata_block` now accepts typed serializable records instead of requiring `serde_json::Value`.
  - salvage redundant-map ledger parsing moved to typed serde structs (`RedundantMapLedger*`) instead of ad-hoc object/array field walking via `Value`.
- CRUSHR-HARDEN-03I-FIX1 completed the remaining salvage typing gap in `crushr_salvage/core/metadata.rs`:
  - active metadata scanning now produces typed `ExperimentalMetadataRecord` variants instead of `Vec<Value>`
  - active salvage metadata parsers/classifiers (`path checkpoints`, `path dictionary`, `payload identity`, `file identity`, `manifest`) now consume typed structs/enums
  - bootstrap-anchor availability checks now run against typed metadata variants
  - typed salvage metadata/unit coverage is green (`crushr-salvage` bin tests), and canonical verification-model determinism tests remain green in `crushr-core`
- CRUSHR-HARDEN-03I-FIX2 removed the last localized active-path `serde_json::Value` carrier from dictionary-copy-v2 parity parsing:
  - `PathDictionaryCopyV2RawRecord` no longer stores `body: Value`
  - dictionary `body_raw_json` extraction now uses direct raw-slice extraction (`extract_top_level_field_raw_json`) from verified metadata block bytes
  - dictionary hash/length parity checks remain deterministic and green under focused tests


## Active constraints

- Workspace crate policy is locked: resolver `3`, edition `2024`, MSRV `1.88`, publishable crates must carry crates.io-facing workspace metadata inheritance, and internal crates must set `publish = false`.
- Unified policy gate baseline is active on PRs/pushes to `main`: secrets, dependency audit, MSRV, style (crate policy + clippy + fmt), and VERSION drift checks.
- No speculative reconstruction/repair in `crushr-extract`; `--recover` must preserve explicit trust segregation and no guessed naming/path claims.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation.
- Comparison workflows remain bounded and storage-conscious; do not rerun the full Phase 2 matrix without explicit instruction.
- FORMAT-08 placement strategy changes metadata placement only; payload layout semantics remain unchanged.
- Current packer writes one payload block/extent per file in baseline behavior; stress fragmentation scenarios use deterministic logical-file fragment sets and report grouped extents-per-logical-file distributions.

## Active recovery-graph layering

1. payload truth
2. extent/block identity truth
3. file manifest truth
4. path truth

Recovery should degrade in reverse order:
1. full named recovery
2. full anonymous recovery
3. partial ordered recovery
4. orphan evidence

## Next actions

1. Execute `CRUSHR_CLEANUP_03` to deduplicate recover metadata-degraded routing and consolidate shared manifest placement helpers.
2. Execute `CRUSHR_CLEANUP_04` to decompose `commands/pack.rs` into clearer internal responsibility boundaries.
3. Execute `CRUSHR_CLEANUP_05` for `info` wording/structure truth alignment (no overstated model claims).
4. Execute `CRUSHR_CLEANUP_06` to centralize benchmark matrix/config authority between generator/runner/docs.

## Near-term product-completeness track (not active yet)

Once the current resilience evaluation arc settles, the next product-facing completeness gap to close is Unix metadata preservation:
- file type
- mode
- uid/gid
- optional uname/gname policy
- mtime policy
- symlink target
- xattrs

## Later optimization track (not active yet)

Once resilience and metadata pruning decisions settle, revisit distributed dictionary work:
- explicit dictionary identity
- verifiable block -> dictionary dependency
- deterministic degradation when a dictionary is missing
- no silent decode fallback that changes truth


- CRUSHR-HARDEN-03C introduced explicit schema files for active FORMAT-12/13/14A/15 comparison outputs and added schema-backed artifact checks in integration tests.
- Remaining follow-up debt: pack/salvage typed metadata conversion is still open under CRUSHR-HARDEN-03G follow-through; no additional verify-boundary debt identified after CRUSHR-HARDEN-03H.
Latest maintenance fix (2026-03-23):
- **CRUSHR_UI_POLISH_02 complete**: expanded `cli_presentation` with reusable presentation primitives for phase/progress rows (`phase`), informational/warning/failure banners (`banner` + `BannerLevel`), and standardized result summaries (`result_summary`) while preserving no-color-safe output.
- **CRUSHR_UI_POLISH_02 complete**: migrated core command presentation to shared primitives across `verify`, `extract`, `extract --recover`, `pack`, and `info`; result sections now render through one helper and verify/extract flows use explicit target/progress/result hierarchy.
- **CRUSHR_UI_POLISH_02 complete**: added banner-based failure/warning framing for verify/refusal and recover non-canonical notes, and refreshed golden fixtures for shared output shape contracts.
- **CRUSHR_UI_POLISH_02 validation**: `cargo fmt --all`, `cargo test -p crushr --test cli_presentation_contract`, `cargo test -p crushr --test recovery_extract_contract`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` are green.
