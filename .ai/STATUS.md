# crushr Development Status

Current Phase: Phase 3 — Salvage Planning and Recovery-Graph Research Boundary

Current Step: **CRUSHR-FORMAT-09 complete** (metadata survivability + necessity audit harness)

Immediate Next Step: **CRUSHR-FORMAT-10** (metadata pruning/redesign decision packet informed by FORMAT-09 evidence)

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
- FORMAT-09 now adds an expanded corruption matrix (metadata regime × metadata target × payload topology) and emits `format09_comparison_summary.{json,md}` with survivability/gain metrics.
- The next major decision point is metadata pruning/redesign scope in FORMAT-10 based on FORMAT-09 strategy deltas and survivability evidence.

## Active constraints

- No speculative recovery/reconstruction/repair in `crushr-extract`.
- `crushr-salvage` output is unverified research output and not canonical extraction.
- No guessed mappings, guessed extents, speculative byte stitching, or archive mutation.
- Comparison workflows remain bounded and storage-conscious; do not rerun the full Phase 2 matrix without explicit instruction.
- FORMAT-08 placement strategy changes metadata placement only; payload layout semantics remain unchanged.

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
2. Use FORMAT-09 output to identify metadata layers that never survive and candidates that produce measurable recovery gains.
3. Prepare FORMAT-10 proposals for metadata pruning/demotion/retention by strategy and damage regime.
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
