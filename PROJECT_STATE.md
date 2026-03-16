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
- Current experimental direction after FORMAT-08: payload identity + file truth + graph reasoning are in place; the next packet is **CRUSHR-FORMAT-09**, which increases evaluation pressure via a curated corruption grid rather than changing the format again.

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

### FORMAT-09 next-step boundary

- FORMAT-09 is an evaluation-harness packet, not a format-layout packet.
- Purpose:
  - apply a richer corruption grid
  - stress truth-layer survivability and downgrade behavior
  - determine whether weak duplicated metadata surfaces are actually worth their archive-size overhead
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

These remain explicitly deferred until FORMAT-09 evidence exists:

- deciding which duplicated metadata surfaces should be removed
- larger placement-strategy bakeoffs beyond the three current strategy names
- generalized graph-engine abstraction beyond bounded packet needs
- distributed dictionary optimization work

## Out-of-scope invariants (unchanged)

- no speculative stitching/reconstruction
- no guessed byte emission
- no archive mutation in place
- no integration of experimental recovery semantics into `crushr-extract`
