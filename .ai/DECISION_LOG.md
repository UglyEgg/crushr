<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/DECISION_LOG.md

## 2026-03-27 — CRUSHR_CLEANUP_06 info/introspection truth authority centralization

- Decision:
  - Introduce one canonical info-side reporting authority in `crates/crushr/src/commands/info.rs` (`build_info_truth_view`) to classify preservation contract wording, metadata visibility states, and archive-state language before rendering.
  - Introduce canonical list-result fallback classification (`build_listing_truth_view`) so degraded/complete listing result semantics are decided once and consumed by presentation code.
  - Keep presentation code policy-free: output rendering now consumes pre-classified truth rows/messages rather than re-evaluating profile/fallback semantics branch-by-branch.
- Alternatives considered:
  1. Keep branch-local `match profile` and repeated metadata visibility calls in render flow, with comments only.
  2. Spread small helper functions across section-local rendering blocks without one obvious authority boundary.
- Rationale:
  - Hostile review and CRUSHR_CLEANUP_06 packet require one auditable location answering “where does info decide what this means?” for operators.
  - Centralized truth mapping reduces wording/classification drift between profile semantics, fallback semantics, and metadata visibility semantics.
- Blast radius:
  - `crates/crushr/src/commands/info.rs` internal reporting structure and introspection presentation tests only.
  - No pack/extract/recover behavior changes, no archive/schema/CLI-shape changes, and no version bump.


## 2026-03-27 — CRUSHR_CLEANUP_05 pack ownership-layer decomposition

- Decision:
  - Introduce explicit internal pack ownership layers (`discovery`, `planning`, `emission`) and route top-level command orchestration through those bounded interfaces.
  - Keep orchestration thin (`run`/`pack_minimal_v1` hold user-facing flow only) and keep low-level mechanics behind the internal layer boundaries.
  - Preserve existing pack semantics and the canonical profile/planning authority model from CRUSHR_CLEANUP_02.
- Alternatives considered:
  1. Keep `commands/pack.rs` as a monolithic module and only annotate section comments.
  2. Perform a full multi-file extraction in the same packet with broader mechanical relocation risk.
- Rationale:
  - Hostile review flagged pack as an overgrown mixed-responsibility accretion zone; explicit ownership layers reduce review blast radius and make discovery/planning/emission boundaries obvious.
  - This packet is structural cleanup only, so bounded internal decomposition with behavior lock is preferred over semantic or format change.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` internal structure and pack-local tests only.
  - No CLI contract changes, no archive/schema changes, and no version bump.


## 2026-03-27 — CRUSHR_CLEANUP_03 recover metadata-degraded routing authority collapse

- Decision:
  - Establish one canonical recover-path authority (`route_metadata_degraded_entry`) for metadata-degraded routing across all recover-supported entry kinds.
  - Centralize metadata-degraded recover manifest entry assembly in one helper (`build_metadata_degraded_manifest_entry`) with shared degradation reason/trust mapping.
  - Remove branch-local duplication of degraded rename routing and manifest-entry field construction from recover extraction branches.
- Alternatives considered:
  1. Keep branch-local routing/manifest assembly and only extract shared string constants.
  2. Keep per-entry-kind helpers with duplicated manifest field population.
- Rationale:
  - Hostile review and cleanup packet CRUSHR_CLEANUP_03 require one auditable authority for degraded metadata routing and recover-side manifest assembly.
  - Single-owner routing/assembly reduces drift risk when metadata obligations change and keeps recover safety semantics explicit.
- Blast radius:
  - `crates/crushr/src/recover_extract_impl.rs` internal recover extraction path only.
  - No strict extraction behavior changes, no archive format/schema changes, and no public API changes.

## 2026-03-27 — CRUSHR_CLEANUP_02 pack preservation-profile authority collapse

- Decision:
  - Establish one canonical pack-time preservation-profile decision authority (`plan_pack_profile`) that consumes raw discovery candidates and returns explicit included/omitted outcomes (`PackProfilePlan`).
  - Carry omission classification (`ProfileOmissionReason`) in the plan and centralize omission warning emission through one path (`emit_profile_warnings`) based on those plan outcomes.
  - Remove discovery-time profile omission/warning behavior and remove separate post-discovery profile mutation helper (`apply_preservation_profile`) to eliminate split ownership.
  - Keep emission/finalization policy-free by consuming authoritative plan outcomes only (`layout.profile_plan.included`), without profile rule re-evaluation.
- Alternatives considered:
  1. Keep profile-aware omission/warnings in discovery and only simplify `apply_preservation_profile`.
  2. Keep both discovery and post-discovery profile handling with additional assertions to detect drift.
- Rationale:
  - Hostile review identified multi-owner profile semantics in pack as a drift risk and warning-duplication risk.
  - A single explicit planning authority makes review/audit straightforward: one place decides inclusion/omission/classification/warning intent.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` discovery/planning/emission internals and pack unit tests only.
  - No public profile semantic redesign, no schema/version changes, no extract/recover/info behavior changes.

## 2026-03-27 — CRUSHR_OPTIMIZATION_03 zstd context-reuse lock

- Decision:
  - Replace per-block stream-encoder construction in production `pack` with a reusable `zstd::bulk::Compressor` context scoped to the pack run.
  - Keep deterministic zstd frame flags explicitly configured on the reusable context (`checksum=false`, `contentsize=true`, `dictid=false`) and preserve existing compression-level behavior.
  - Route payload and metadata block compression through one reusable compressor-owned output buffer (`compress_to_buffer`) to reduce allocation/setup churn in the hot path.
- Alternatives considered:
  1. Keep per-unit `zstd::Encoder` setup/finish and only tune buffer capacities.
  2. Add compression parallelism before serial context-reuse improvements.
  3. Lower compression level to claim phase reduction.
- Rationale:
  - Profiling from prior packets showed compression as dominant; encoder setup/teardown per unit remained a high-confidence avoidable cost.
  - Reusing compression context is a serial-path efficiency optimization that preserves archive validity, deterministic ordering, and packet guardrails.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` compression internals only.
  - No format, codec, level-default, preservation-profile, or verification semantic changes.
  - Canonical version advanced to `0.4.20`.

## 2026-03-27 — CRUSHR_OPTIMIZATION_02 compression/emission overhead reduction lock

- Decision:
  - Keep production compression codec/level semantics unchanged, but reuse a per-run compression output buffer for payload and metadata block zstd writes to reduce repeated allocation/setup overhead.
  - Route archive emission through a buffered writer (1 MiB `BufWriter`) and keep deterministic block offset accounting explicitly in the pack emitter so buffered I/O does not alter identity-record offset truth.
  - Preserve existing phase boundaries (`compression` vs `emission`) by timing compression/hashing/writes at unchanged logical points after refactor.
- Alternatives considered:
  1. Lower default compression level or swap codecs to force apparent phase wins.
  2. Skip/relax hashing or mutation checks to shrink measured phase time.
  3. Collapse compression/emission timers into one combined phase after buffering changes.
- Rationale:
  - Packet requires measurable compression/emission gains without semantic drift or benchmark-only shortcuts.
  - Buffer reuse and write buffering are implementation-level overhead reductions that keep archive bytes, profile behavior, and validation semantics intact.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` compression/emission internals and write plumbing only.
  - No public API/format changes; preservation profile, mutation detection, and finalization contracts remain unchanged.
  - Canonical version advanced to `0.4.19`.

## 2026-03-27 — CRUSHR_OPTIMIZATION_01 discovery-phase profile-aware capture lock

- Decision:
  - Move preservation-profile omission behavior into discovery capture so `basic`/`payload-only` do not eagerly probe metadata classes they intentionally omit.
  - Add discovery capture policy gates for ownership/name lookup, xattr/security probes, sparse probing, symlink/special-entry inclusion, and hard-link key capture.
  - Remove duplicate planning-time regular-file `stat` calls by carrying discovery-captured `raw_len` into layout planning.
- Alternatives considered:
  1. Keep profile filtering in `apply_preservation_profile` only and attempt micro-optimizations around path allocation.
  2. Move omitted metadata probes to a later phase while keeping eager discovery scans unchanged.
- Rationale:
  - Benchmark attribution identified discovery as dominant and showed `basic` paying near/full discovery cost; eager omitted-metadata probing was the highest-confidence avoidable source.
  - Eliminating unnecessary syscalls in discovery is safer and more truthful than phase relabeling or deferred no-op work.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` discovery/planning path now captures only profile-required metadata and reuses discovery `raw_len` in planning.
  - `docs/reference/benchmarking.md` operator profiling commands now include required medium+large full/basic runs for validation symmetry.
  - Added packet completion evidence at `.ai/COMPLETION_NOTES_CRUSHR_OPTIMIZATION_01.md`.

## 2026-03-26 — CRUSHR_BENCHMARK_03 pack-phase attribution surface lock

- Decision:
  - Add explicit production-only flag `crushr pack --profile-pack` to emit deterministic human-readable phase timing breakdown.
  - Instrument pack pipeline timings across six attributed phases: `discovery`, `metadata`, `hashing`, `compression`, `emission`, and `finalization`.
  - Keep profiling opt-in only; default pack output and archive semantics remain unchanged unless `--profile-pack` is supplied.
- Alternatives considered:
  1. Keep benchmark attribution external-only (e.g., system profiler/perf) with no product-surface instrumentation.
  2. Emit profiling data by default in normal pack output.
  3. Add per-file tracing/noisy debug output instead of bounded phase totals.
- Rationale:
  - Benchmark deficit needs stage-level attribution evidence before optimization packets can be scoped safely.
  - Opt-in phase totals provide deterministic, low-noise signals for human benchmark investigation without changing pack behavior/contracts.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` gained explicit phase timers and `--profile-pack` parsing/help surface.
  - `crates/crushr/tests/cli_contract_surface.rs` gained profiling-surface contract checks.
  - `docs/reference/benchmarking.md` now includes local operator commands and capture guidance for phase-attribution runs.

## 2026-03-26 — CRUSHR_PRESERVATION_FIX_06 extraction profile-authority lock

- Decision:
  - Treat the archive-recorded preservation profile as authoritative for extraction restore execution, not only for post-hoc metadata failure classification.
  - Skip restoration attempts (and associated warnings) for metadata classes omitted by profile in both strict and recover extraction paths.
  - Keep full-profile behavior unchanged: required metadata restoration continues to attempt restore and surfaces failure warnings/refusal/degraded routing as before.
- Alternatives considered:
  1. Keep current behavior (attempt restore for all classes, then filter omitted classes only in classification).
  2. Suppress warnings only while still performing omitted-class restore syscalls.
- Rationale:
  - Packet requires `info` contract truth and extraction behavior to agree; omitted-by-profile metadata is outside archive obligations and must not produce restoration warnings.
  - Skipping omitted-class syscalls is the only deterministic way to prevent spurious warning emission across strict/recover code paths.
- Blast radius:
  - `strict_extract_impl` and `recover_extract_impl` metadata restore helpers now gate ownership/xattr/ACL/SELinux/capability restore attempts by profile.
  - Metadata preservation integration coverage expanded for omitted-profile warning suppression and full-profile ownership warning assertions.

## 2026-03-26 — CRUSHR_BENCHMARK_01 deterministic benchmark contract lock

- Decision:
  - Introduce a deterministic benchmark contract with three generated dataset classes (`small_mixed_tree`, `medium_realistic_tree`, `large_stress_tree`) using fixed seed + fixed mtime generation semantics.
  - Lock baseline comparator set for this packet to `tar+zstd`, `tar+xz`, `crushr(full)`, and `crushr(basic)` with explicit command forms and level alignment at `3`.
  - Record benchmark runs in a structured JSON artifact (`crushr-benchmark-run.v1`) with required fields for dataset/tool/profile/commands/archive-size/pack+extract timing/peak RSS and environment context.
  - Keep this packet methodology-only: no benchmark tuning, no performance-threshold assertions, and no selective result filtering behavior.
- Alternatives considered:
  1. Keep ad-hoc manual benchmark commands in docs/shell history without a locked data contract.
  2. Build benchmark comparisons only for crushr profiles and defer baseline tool comparisons.
  3. Add CI enforcement immediately in the same packet.
- Rationale:
  - Packet objective is reproducible and defensible methodology first; explicit dataset generation + explicit command provenance prevents drift/cherry-picking.
  - Structured output and schema lock future analysis/review to consistent machine-readable fields before optimization work begins.
- Blast radius:
  - Added `scripts/benchmark/` generation/runner tooling, benchmark reference docs, schema file, docs index link, and `.bench` local artifact ignore policy.
  - No changes to archive format/extract semantics and no compression/performance tuning behavior changes.

## 2026-03-26 — CRUSHR_PACK_STREAMING_01 production pack raw-byte retention removal

- Decision:
  - Remove raw payload byte vectors from `emit_archive_from_layout` hard-link reuse cache (`payload_materialized_by_block`) and retain only immutable per-block identity/length/offset metadata.
  - Reuse the already computed per-block raw BLAKE3 digest for file-manifest `file_digest` instead of retaining and re-hashing payload bytes later in the run.
  - Keep existing fail-closed mutation detection timing and behavior unchanged.
- Alternatives considered:
  1. Keep retaining raw payload bytes in the hard-link map and rely on larger host memory.
  2. Re-read source files only for manifest hashing in a second pass.
- Rationale:
  - The recurring OOM regression was caused by raw payload vectors being retained across the full pack run (`block_id -> ... -> Vec<u8>`), which scaled with archive-total payload size rather than active working set.
  - Reusing already-computed raw hashes preserves deterministic manifest truth while eliminating broad payload residency.
- Blast radius:
  - `crates/crushr/src/commands/pack.rs` pack serialization path and manifest record builder signature.
  - No archive format contract changes, no extraction/verify behavior changes, and no mutation-guard semantics changes.

## 2026-03-25 — CRUSHR_INTROSPECTION_02 profile-aware introspection presentation lock

- Decision:
  - Expand `crushr info` with explicit preservation contract labeling and compact entry-kind summary counts.
  - Represent metadata visibility with three states (`present`, `not present`, `omitted by profile`) so omission intent is visually neutral and not treated as corruption/failure.
  - Keep `crushr info --list` scope explicit (`regular files (metadata/index proven)`) while preserving fail-closed listing rules and deterministic ordering.
- Alternatives considered:
  1. Keep binary present/absent metadata rows and rely on docs for omission semantics.
  2. Add per-line verbose badges for every listed item.
  3. Expand `info --list` to include non-regular item rendering despite current regular-file-only listing contract.
- Rationale:
  - Packet requires operators to differentiate archive contract omission vs extraction degradation from introspection output alone, without adding noisy per-entry clutter.
  - Scope labels and calm omission wording preserve integrity-first signaling and reduce false error interpretation.
- Blast radius:
  - `crushr-info` human output shape changed (`Preservation`, `Entry kinds`, `Metadata`, and `--list` archive context rows), plus updated tests/goldens and README introspection wording.
  - Canonical version advanced to `0.4.12` (`VERSION` + workspace package version sync).

## 2026-03-25 — CRUSHR_PRESERVATION_05 explicit preservation-profile archive contract lock

- Decision:
  - Add production pack flag `--preservation <full|basic|payload-only>` with default `full`; do not add `--strip` alias.
  - Advance production index encoding to IDX7 and record the selected preservation profile explicitly in archive metadata.
  - Treat legacy archives without profile metadata (IDX3/IDX4/IDX5/IDX6) as `full` compatibility profile.
  - Make strict/recover canonical metadata-degraded classification profile-aware so profile-omitted classes do not count as restoration failure.
  - In non-`full` profiles, excluded entry kinds are warned and omitted; no flattening/fabrication fallback is allowed.
- Alternatives considered:
  1. Keep implicit full-preservation semantics and infer profile from missing metadata.
  2. Add a `--strip` compatibility alias.
  3. Keep canonical extraction requirements fixed at max format capability regardless of archive profile.
- Rationale:
  - Packet requires explicit archive self-description so omission intent is distinguishable from corruption/restore failure/legacy limits.
  - Profile-aware canonicality preserves trust-model honesty and removes false metadata-degraded outcomes for intentional omissions.
- Blast radius:
  - `crushr-pack` CLI/help/planning/emission, `index_codec`/tailframe magic acceptance, `strict_extract_impl` and `recover_extract_impl` metadata-failure classification, `crushr info` presentation, and metadata/index/CLI contract tests were updated.
  - Workspace version bumped to `0.4.10` (`VERSION` + workspace package version).

## 2026-03-25 — CRUSHR_RECOVERY_MODEL_07 metadata-degraded trust lock

- Decision:
  - Extend extract/recover trust classes with explicit `metadata_degraded`.
  - Canonical now requires successful restoration of required preserved metadata classes (not only path/name/data proof).
  - Strict extraction must refuse when any selected entry is metadata-degraded.
  - Recover extraction must emit metadata-degraded outputs under `metadata_degraded/` and must not merge them into `canonical/`.
  - Recovery manifest schema/entries now carry explicit metadata degradation fields: `trust_class`, `missing_metadata_classes`, `failed_metadata_classes`, and optional `degradation_reason`.
- Alternatives considered:
  1. Keep treating metadata-restore failures as canonical with warning-only output.
  2. Collapse metadata-degraded outcomes into `recovered_named`.
- Rationale:
  - Packet requires trust-model honesty: metadata restoration failure is a distinct non-canonical outcome when content/path/name are still proven.
  - Warning-only canonical classification was misleading once Linux tar-class metadata preservation became part of the archive contract.
- Blast radius:
  - `strict_extract_impl`, `recover_extract_impl`, recover manifest schema, CLI trust/summary rendering, and metadata/recovery contract tests were updated.
  - Recover output layout now includes `metadata_degraded/`, and strict error messaging now references metadata restoration failure explicitly.

## 2026-03-25 — CRUSHR_PRESERVATION_04 ACL/SELinux/capability preservation lock

- Decision:
  - Advance production index encoding to IDX6 to carry explicit structured metadata for POSIX ACLs (`acl_access`, `acl_default`), SELinux labels (`selinux_label`), and Linux file capabilities (`linux_capability`).
  - Capture these metadata classes separately from generic xattrs during pack to avoid silent omission and provide explicit visibility in archive semantics.
  - Restore ACL/SELinux/capability metadata best-effort in strict/recover extraction with explicit warning classes (`WARNING[acl-restore]`, `WARNING[selinux-restore]`, `WARNING[capability-restore]`) when blocked by privilege/platform/runtime policy.
  - Extend `crushr info` metadata presence summary with `ACLs`, `SELinux labels`, and `capabilities`.
- Alternatives considered:
  1. Keep IDX5 and rely entirely on generic xattrs for ACL/SELinux/capability behavior.
  2. Defer ACL/SELinux/capability support to a later packet and continue current metadata envelope.
- Rationale:
  - Packet requires explicit enterprise-relevant Linux security metadata preservation and truthful restoration/degradation semantics.
  - Structured IDX6 fields make these classes first-class and inspectable instead of implicit side effects.
- Blast radius:
  - Index codec/tail-frame magic acceptance, pack capture path, strict/recover extract restoration warnings, and `info` metadata summary were updated.
  - Golden fixtures and metadata/index regression tests were expanded for IDX6 and metadata-presence behavior.

## 2026-03-25 — CRUSHR_PRESERVATION_04 restore-order follow-up lock

- Decision:
  - Apply structured security metadata restoration (ACL/SELinux/capabilities) after ownership restore in strict/recover extraction metadata flows.
- Alternatives considered:
  1. Keep security metadata restoration inside generic xattr restore before ownership changes.
- Rationale:
  - Ownership/mode transitions can clear capability state; applying capabilities after ownership restore preserves truthful round-trip behavior.
- Blast radius:
  - `strict_extract_impl` and `recover_extract_impl` metadata ordering changed; manual completion evidence updated to confirm capability round-trip in privileged context and warning-based degradation in non-root context.

## 2026-03-25 — CRUSHR_PRESERVATION_03 sparse/special-entry/ownership-name lock

- Decision:
  - Advance production index encoding to IDX5 to represent sparse regular files (`logical_offset` extent mapping), FIFO entries, character/block device entries, and optional device major/minor metadata.
  - Capture ownership-name enrichment (`uname`/`gname`) at pack time when available, while keeping numeric uid/gid authoritative.
  - Restore sparse files hole-aware and restore special files best-effort in strict/recover extraction; when special restoration is blocked by privilege/platform constraints, continue extraction and surface explicit `WARNING[special-restore]`.
  - Extend `crushr info` metadata presence visibility with `sparse files` and `special files`.
- Alternatives considered:
  1. Keep IDX4 and flatten sparse/special entries into regular files.
  2. Preserve sparse data by materializing holes as zero payload bytes.
- Rationale:
  - Packet requires truthful Linux-first tar-class behavior without silent type falsification.
  - IDX5 avoids lossy inference and keeps sparse/special semantics explicit/verifiable in index truth.
- Blast radius:
  - Pack/index encoding, strict/recover extraction materialization, info metadata visibility, and tail-frame IDX magic acceptance paths were updated.
  - Golden fixtures and metadata-preservation regression coverage expanded to include sparse/FIFO/device/ownership-name cases.

## 2026-03-25 — CRUSHR_PRESERVATION_02 ownership + hard-link + info metadata visibility lock

- Decision:
  - Advance index encoding to IDX4 for production archives to store ownership (`uid`/`gid`, optional `uname`/`gname`) and hard-link group identity explicitly.
  - Preserve hard-linked regular files as one payload block with multiple file mappings that reference the shared block and hard-link group.
  - Restore ownership best-effort during strict/recover extraction; failures are surfaced as `WARNING[ownership-restore]` and extraction continues.
  - Add `info` metadata visibility section with presence/absence rows only (`modes`, `mtime`, `xattrs`, `ownership`, `hard links`).
- Alternatives considered:
  1. Keep IDX3 and infer ownership/hard links heuristically at extract-time.
  2. Preserve ownership but keep hard links as duplicated payload units.
- Rationale:
  - Packet requires truthful, inspectable preservation semantics approaching tar-class Linux behavior.
  - Explicit on-disk representation avoids silent metadata loss and removes extract-time guesswork.
- Blast radius:
  - Pack/index encoding, strict/recover extraction metadata restoration, and info human output contracts changed.
  - Tail-frame and salvage/index-magic compatibility paths now accept IDX3/IDX4.

## 2026-03-24 — CRUSHR_PRESERVATION_01 baseline Linux-first metadata preservation lock

- Decision:
  - Extend IDX3 entry-kind model to include explicit `directory` records (kind `2`) while preserving compatibility with existing regular/symlink decode behavior.
  - Preserve baseline Linux-first metadata in production pack/extract: entry kind (`regular`/`directory`/`symlink`), link target for symlinks, mode, mtime, empty-directory paths, and xattrs.
  - Keep uid/gid out of this packet (deferred) to avoid widening on-disk contract scope beyond the locked baseline.
  - On extract, surface xattr restore failures explicitly as warnings (no silent drop), and keep non-Linux behavior as honest degradation.
- Alternatives considered:
  1. Keep regular-only strict extraction and defer directory/symlink materialization to a later packet.
  2. Add uid/gid and broader Unix metadata envelope immediately.
- Rationale:
  - Packet requires tar-class baseline semantics for practical Linux workflows now, including empty directories and xattrs.
  - Explicit directory entry kinds avoid flattening behavior and preserve deterministic, inspectable archive semantics.
  - Deferring uid/gid keeps scope bounded while still delivering the locked baseline preservation contract.
- Blast radius:
  - `pack`/`extract`/`recover extract` semantics changed for non-regular entries and metadata restoration.
  - IDX3 encode/decode now accepts/emits directory entry kind.
  - Deterministic output tests and CLI info golden normalization updated for metadata-aware archive hashes.

## 2026-03-24 — CRUSHR_INTROSPECTION_01-FIX2 non-regular omission semantics

- Decision:
  - Treat non-regular entry omissions in `info --list` as informational visibility, not as structural degradation.
  - Keep `DEGRADED` status scoped to structural/listing-proof failures only.
  - Emit `omitted entries` result rows only when non-zero.
- Alternatives considered:
  1. Keep omission warnings contributing to `DEGRADED` status.
  2. Always emit omission count row, including `0`.
- Rationale:
  - Preserves honest structural health signaling while still making omission behavior explicit.
- Blast radius:
  - `crushr-info --list` output semantics changed for omission-only archives (informational note + `COMPLETE`).

## 2026-03-24 — CRUSHR_INTROSPECTION_01-FIX1 omitted-entry and degraded-guidance lock

- Decision:
  - Keep `info --list` output focused on regular file paths but make omitted non-regular IDX3 entries explicit in results/warnings.
  - When IDX3 proof is unavailable, keep fail-closed listing behavior and add explicit operator guidance toward `crushr salvage <archive>` for recovery-oriented evidence.
  - Align canonical version to `0.4.1` for this follow-up fix.
- Alternatives considered:
  1. Keep silent omission of non-regular entries.
  2. Attempt salvage-style inferred listing in `info --list` when IDX3 is unavailable.
- Rationale:
  - Explicit omission counts avoid hidden behavior when index entry kinds evolve.
  - `info --list` remains integrity-first and non-speculative while still giving users a clear next action for degraded archives.
- Blast radius:
  - `crushr-info` list output gains `omitted entries` result row and degraded warning guidance line.
  - CLI presentation contract test updated for degraded guidance text.

## 2026-03-24 — CRUSHR_INTROSPECTION_01 info listing contract lock

- Decision:
  - Add `crushr info --list` as a metadata/index-only introspection path that never extracts payload bytes.
  - Default listing mode is directory-aware tree output; `--flat` emits deterministic full-path listing.
  - Degrade fail-closed on corruption: show only IDX3-proven paths and emit warning banners when archive structure is damaged or listing proof is unavailable.
  - Keep trust labeling scoped to degraded/introspection warnings rather than annotating every listed line.
- Alternatives considered:
  1. Keep listing unavailable unless full `open_archive_v1` succeeds.
  2. Guess missing directories/paths from partial metadata fragments under corruption.
- Rationale:
  - Packet requires pre-extraction visibility with strict no-guess semantics.
  - IDX3-backed paths provide a provable logical content view while preserving integrity-first behavior under partial damage.
- Blast radius:
  - `crushr-info` help/flag surface now includes `--list` and `--flat`.
  - Added CLI integration coverage for tree/flat listing and degraded proof-unavailable behavior.
  - Existing `crushr info` default and `--json` snapshot behavior remain unchanged.

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
