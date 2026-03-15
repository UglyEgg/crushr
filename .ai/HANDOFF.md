# HANDOFF

## Current focus
- CRUSHR-FORMAT-05, -f1, -f2, and -f3 are complete; FORMAT-05 comparison now has behavioral runner/packer contract checks and no `crushr-pack --help` dependency.
- CRUSHR-SCRUB-02 and CRUSHR-SCRUB-02-f1 are complete: `crushr-pack` now rejects duplicate logical archive paths before archive emission with deterministic, stably ordered collision source errors.
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


## Update: CRUSHR-SCRUB-02 complete
- `crushr-pack` now normalizes logical paths once (`\` to `/`) and rejects duplicate final logical archive paths before creating/writing output archives.
- Duplicate collision errors are explicit and deterministic (colliding logical path + conflicting source inputs).
- Regression tests now cover distinct success, basename collisions, normalization-only collisions, walked-tree collisions, three-way collisions, deterministic source-order assertions, and no partial archive emission behavior.


## Update: CRUSHR-SCRUB-02-f1 complete
- Duplicate-collision source listing is now explicitly sorted for deterministic error output.
- Input ordering for identical logical paths is stabilized by sorting collected files by `(rel_path, abs_path)` before duplicate detection.


## Update: CRUSHR-PLAN-LEGACY-01 complete
- Supported extraction authority is now explicit: `crushr-extract` only.
- Root `crushr extract` and `crates/crushr/src/api.rs::extract_all` are quarantined legacy surfaces that fail with explicit unsupported errors.
- Regression tests guard quarantine behavior for both all-entry and path-filtered root CLI extraction invocation modes.


## Update: CRUSHR-PLAN-LEGACY-01-f1 complete
- `crates/crushr/tests/mvp.rs` now names the root extract test by its quarantine behavior instead of roundtrip semantics.
- Added positive integration evidence that `crushr-pack` archives still roundtrip through authoritative `crushr-extract`.


## Update: CRUSHR-PLAN-LEGACY-01-f2 complete
- Preferred boundary implementation is now active: root `crushr extract` and API `extract_all` delegate to the same strict extraction implementation used by `crushr-extract`.
- Extraction authority remains singular by implementation, not quarantine: supported surfaces now share strict semantics.
