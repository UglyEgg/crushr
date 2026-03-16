# HANDOFF

## Current focus

- **CRUSHR-FORMAT-07 is complete**: salvage now reasons over verified relationships (`block -> extent -> manifest -> path`) and emits explicit recovery classes.
- **CRUSHR-FORMAT-08 is complete**: experimental metadata placement strategies now exist (`fixed_spread`, `hash_spread`, `golden_spread`) and the bounded placement comparison command is wired and covered.
- **CRUSHR-FORMAT-09 is complete**: `run-format09-comparison` is now wired and emits survivability/gain evidence across metadata regimes, metadata targets, payload topologies, and placement strategies.
- **CRUSHR-FORMAT-10 is next**: use FORMAT-09 evidence to decide metadata pruning/demotion/retention and potential redesign scope.

## Important behavior locks

- `crushr-extract` remains strict-only and unchanged.
- Recovery direction remains governed by the locked inversion principle:
  - verified payload-adjacent truth is preferred over centralized metadata authority
  - recovery builds upward from surviving verified payload
- Locked graph layering remains:
  - payload truth
  - extent/block identity truth
  - file manifest truth
  - path truth
- Recovery degrades in reverse order:
  - full named recovery
  - full anonymous recovery
  - partial ordered recovery
  - orphan evidence
- FORMAT-08 placement strategy applies only to graph-supporting metadata checkpoints; payload layout remains unchanged.

## Current experimental surfaces

- `crushr-pack --experimental-self-identifying-blocks`
- `crushr-pack --experimental-file-manifest-checkpoints`
- `crushr-pack --placement-strategy <fixed_spread|hash_spread|golden_spread>`
- `crushr-lab-salvage run-format05-comparison --output <dir>`
- `crushr-lab-salvage run-format06-comparison --output <dir>`
- `crushr-lab-salvage run-format07-comparison --output <dir>`
- `crushr-lab-salvage run-format08-placement-comparison --output <dir>`
- `crushr-lab-salvage run-format09-comparison --output <dir>`

## Watch items

- Keep salvage-plan schema and emitted provenance aligned with the currently implemented recovery paths.
- Preserve deterministic CLI dispatch/help registration for every new comparison command; builder has repeatedly missed this.
- Preserve deterministic ordering in anonymous naming, comparison row ordering, grouped metrics, and strategy labeling.
- Use FORMAT-09 results to decide whether weak duplicated metadata surfaces should be pruned rather than preserved out of habit.

## Product / optimization tracks to remember

### Near-term product-completeness track
After the current resilience-evaluation arc settles, add Unix metadata preservation so crushr can preserve the expected Unix file object, not just file bytes:
- file type
- mode
- uid/gid
- optional uname/gname policy
- mtime policy
- symlink target
- xattrs

### Later optimization track
After structural stability and metadata pruning decisions settle, revisit distributed dictionaries:
- explicit dictionary identity
- verifiable block -> dictionary dependency
- deterministic degradation when dictionaries are missing
- no silent truth-changing decode fallback

## Immediate next packet expectation

- FORMAT-10 should use FORMAT-09 evidence to classify metadata layers into:
  - retain (material recovery gain)
  - demote (weak/conditional gain)
  - prune (no meaningful survivability or gain)
- Keep extraction semantics and payload structure unchanged while evaluating pruning/redesign options.
