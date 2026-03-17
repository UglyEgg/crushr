# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-HARDEN-02 complete** (de-cruft pass: runtime/lab module boundary clean-up, salvage planner consolidation, and architecture doc alignment)

Immediate Next Step: **CRUSHR-HARDEN-03 planning** (finish CLI-surface minimization and retire legacy format comparison commands)

## Current truth

- Phase 1 is complete.
- Phase 2 execution is complete and frozen.
- Phase 2 normalization is complete and frozen.
- Phase 2 comparison analysis is complete and frozen.
- `crushr-extract` remains strict-only and unchanged as canonical extraction behavior.
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

1. Preserve strict extraction interfaces/semantics untouched.
2. Use FORMAT-10/11 output to classify metadata layers into keep/prune candidates by measurable recovery delta and overhead cost.
3. Use FORMAT-12/13/14A evidence to lock the dictionary-placement winner and de-risk direct dictionary-target corruption.
4. Keep Phase 2 corpus and frozen artifacts unchanged.
5. Keep builder honest on CLI wiring; every new comparison command must be proven callable via the documented runtime command.

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
