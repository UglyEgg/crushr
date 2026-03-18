# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-HARDEN-03A complete** (API boundary truth + visibility cleanup across runtime/library/docs)

Immediate Next Step: **metadata-pruning evidence review** (use FORMAT-10/11/12/13/14A results to lock keep/prune boundaries)

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

1. Preserve strict extraction interfaces/semantics untouched (including hardened `crushr-extract --verify` refusal behavior).
2. Use FORMAT-10/11 output to classify metadata layers into keep/prune candidates by measurable recovery delta and overhead cost.
3. Use FORMAT-12/13/14A evidence to lock the dictionary-placement winner and de-risk direct dictionary-target corruption.
4. Keep Phase 2 corpus and frozen artifacts unchanged.
5. Treat the newly documented public/internal/lab boundary classes as canonical unless explicitly revised by a future decision/packet.

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
- CRUSHR-HARDEN-03G cleanup now covers both metadata-record builders and redundant-map/tail closeout helpers; no dedicated 03H cleanup step remains.
