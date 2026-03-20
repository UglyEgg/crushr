<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Handoff

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
