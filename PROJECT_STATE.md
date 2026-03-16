# PROJECT_STATE

## Product thesis (active)

crushr is an integrity-first archive system.

Non-negotiable baseline:

- strict extraction only for `crushr-extract`
- deterministic verification and reporting
- no speculative recovery/reconstruction in canonical extraction

## Active tool boundary

- `crushr-pack` — archive creation
- `crushr-info` — archive inspection/reporting
- `crushr-fsck` — strict verification/corruption analysis
- `crushr-extract` — strict verified extraction with deterministic refusal reporting
- `crushr-lab` — controlled research harness
- `crushr-salvage` — separate experimental salvage-planning executable (unverified research output only)

`crushr-salvage` must not change or weaken `crushr-extract` semantics.

## Current implementation scope

- regular files only in canonical v1 behavior
- one block per file in canonical baseline behavior
- deterministic strict extraction reporting (`safe_files` / `refused_files`)

## Phase status (authoritative summary)

- Phase 1: complete.
- Phase 2 execution matrix: complete and frozen.
- Phase 2 normalization: complete and frozen.
- Phase 2 comparison/ranking analysis: complete and frozen.
- Current experimental direction after FORMAT-12: payload identity remains the primary recovery truth, and metadata layers are under explicit pruning evaluation. FORMAT-12 adds `extent_identity_inline_path` to test repeated local path/name truth per extent against `payload_plus_manifest` and `extent_identity_only` for named-recovery gain vs byte overhead.

Canonical Phase 2 workspace root remains `PHASE2_RESEARCH/`.

## Locked resilience direction

Two architectural locks remain active for resilience-oriented experimental work:

1. **Inversion principle**
   - prefer verified payload-adjacent structures as reconstructive truth
   - treat centralized metadata as an accelerator, not sole authority
   - build recovery upward from surviving verified payload rather than downward from fragile roots

2. **Content-addressed recovery graph direction**
   - payload truth
   - extent/block identity truth
   - file manifest truth
   - path truth

Recovery should degrade in reverse order:
- full named recovery
- full anonymous recovery
- partial ordered recovery
- orphan evidence

## Active experimental boundary

### FORMAT-05 / 06 / 07 / 08 cumulative boundary

- FORMAT-05 added explicit experimental payload self-identity and repeated path checkpoints.
- FORMAT-06 added experimental file-manifest checkpoints as the next file-truth layer.
- FORMAT-07 changed salvage reasoning to verified graph-based recovery classification.
- FORMAT-08 changed only the placement of graph-supporting metadata checkpoints using:
  - `fixed_spread`
  - `hash_spread`
  - `golden_spread`

### FORMAT-09 / FORMAT-12 active boundary

- FORMAT-09 established metadata survivability evidence and showed near-zero verified checkpoint survival in the bounded matrix.
- FORMAT-10/11 are pruning experiments, not canonical format changes.
- FORMAT-10 compares four metadata profiles and records both recovery outcomes and size overhead (`format10_comparison_summary.{json,md}`).
- FORMAT-11 compares `payload_only`, `payload_plus_manifest`, `full_current_experimental`, and `extent_identity_only`, emitting `format11_comparison_summary.{json,md}` for distributed extent-identity evidence.
- FORMAT-12 adds `extent_identity_inline_path` and `extent_identity_distributed_names`, and emits `format12_comparison_summary.{json,md}` with grouped recovery/size analysis including path-length duplication visibility.
- FORMAT-12 stress comparison command (`run-format12-stress-comparison`) now emits `format12_stress_comparison_summary.{json,md}` over deterministic `deep_paths`, `long_names`, `fragmentation_heavy`, and `mixed_worst_case` datasets to measure worst-case inline path-duplication overhead.
- FORMAT-13 adds dictionary-encoded path identity variants (`extent_identity_path_dict_single`, `extent_identity_path_dict_header_tail`, `extent_identity_path_dict_quasi_uniform`) and new lab commands `run-format13-comparison` + `run-format13-stress-comparison` for baseline/stress evidence.
- It must not weaken `crushr-extract` or redefine canonical extraction semantics.

## Near-term product-completeness track (planned, not active yet)

After the current resilience-evaluation arc settles, crushr needs a Unix metadata preservation envelope so it cannot be dismissed as “file bytes only” on Unix-like systems.

Bounded first envelope should cover at least:
- file type
- mode
- uid/gid
- optional uname/gname policy
- mtime policy
- symlink target
- xattrs

## Later optimization track (planned, not active yet)

After resilience structure and metadata-layer pruning decisions settle, revisit distributed dictionaries.

Dictionary work must obey the same integrity-first rules:
- explicit dictionary identity
- verifiable block -> dictionary dependency
- deterministic degradation when a required dictionary is missing
- no silent decode fallback that changes truth

## Deferred-not-active research directions

These remain explicitly deferred until FORMAT-10 pruning evidence is reviewed and accepted:

- deciding which duplicated metadata surfaces should be removed
- larger placement-strategy bakeoffs beyond the three current strategy names
- generalized graph-engine abstraction beyond bounded packet needs
- distributed dictionary optimization work

## Out-of-scope invariants (unchanged)

- no speculative stitching/reconstruction
- no guessed byte emission
- no archive mutation in place
- no integration of experimental recovery semantics into `crushr-extract`
