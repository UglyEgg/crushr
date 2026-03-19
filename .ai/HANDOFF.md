# Handoff

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
