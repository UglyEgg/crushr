<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-UI-01 complete** (unified CLI presentation contract + standardized `--silent` mode landed across pack/extract/verify/salvage)

Immediate Next Step: metadata-pruning evidence review and benchmark-harness preparation on top of the unified operator-facing CLI surface


Latest maintenance fix (2026-03-20):
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
- `crushr-extract` remains strict-only as canonical extraction behavior, and now owns strict pre-extraction verification via `--verify`.
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

- No speculative recovery/reconstruction/repair in `crushr-extract`.
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

1. Use FORMAT-10/11 output to classify metadata layers into keep/prune candidates by measurable recovery delta and overhead cost.
2. Use FORMAT-12/13/14A evidence to lock the dictionary-placement winner and de-risk direct dictionary-target corruption.
3. Keep strict extraction interfaces/semantics untouched (including hardened `crushr-extract --verify` refusal behavior).
4. Establish a unified CLI identity/presentation layer before building the benchmark harness so future output/report surfaces inherit one product language.

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
