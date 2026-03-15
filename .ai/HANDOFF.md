# HANDOFF

## Current focus
- CRUSHR-FORMAT-05, -f1, -f2, and -f3 are complete; FORMAT-05 comparison now has behavioral runner/packer contract checks and no `crushr-pack --help` dependency.
- The next active packet is **CRUSHR-FORMAT-06**: verified file manifest checkpoints as the next recovery-graph layer.

## Important behavior locks
- `crushr-extract` remains strict-only and unchanged.
- Experimental recovery direction now follows the locked inversion principle:
  - verified payload-adjacent truth is preferred over centralized metadata authority
  - recovery builds upward from surviving verified payload
- The locked recovery-graph layering is:
  - payload truth
  - extent/block identity truth
  - file manifest truth
  - path truth
- Recovery degrades in reverse order:
  - full named recovery
  - full anonymous recovery
  - partial ordered recovery
  - orphan evidence
- Pseudo-random / low-discrepancy checkpoint placement is deferred backlog research, not the current active experiment.

## Current experimental surfaces
- `crushr-pack --experimental-self-identifying-blocks` (canonical FORMAT-05 writer flag; contract is enforced in lab runner)
- `crushr-lab-salvage run-format05-comparison --output <dir>`
- FORMAT-06 will extend the current experimental path with verified file manifest checkpoints rather than replacing it.

## Watch items
- Keep salvage-plan schema and emitted provenance aligned with the currently implemented experimental recovery paths.
- Preserve deterministic ordering in anonymous naming, comparison row ordering, and grouped metrics.
- Do not let builder drift back toward centralized-metadata-only solutions; current evidence supports payload identity -> manifest truth as the active path.

## Update: CRUSHR-SCRUB-01 complete
- Shared extraction path confinement is now enforced in canonical (`crushr-extract`), legacy extraction, and API-routed extraction.
- Unsafe archive entry paths now hard-fail deterministically; no silent fixups are allowed.
- Symlink extraction is currently rejected in hardened mode by policy.
