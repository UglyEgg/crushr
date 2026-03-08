# .ai/BACKLOG.md

This backlog is intentionally short and specific. New items should be inserted with an owner (tool/crate) and a clear done-condition.

## Immediate (Phase 0)

- [ ] Fix and remove all known compilation breakpoints in legacy `crates/crushr` code (prototype remnants). Done when `cargo test --workspace` passes.
- [ ] Ensure `SPEC.md` stays authoritative: each format primitive implemented in `crushr-format` must have unit tests and a spec cross-check note.

## After Phase 0 Gate 1

- [ ] Decide defaults: block size, tail frame count/checkpoint cadence, block hashing defaults, cross-platform xattr posture.

## Phase 1 Gate 2

- [ ] Decide partial extraction default behavior: truncate vs sparse vs fill.

## Phase 2 (Cool roadmap, ordered)

1. Compression ledger enrichment (richer plan/results fields)
2. Integrity heatmap + blast radius simulator (info + TUI)
3. Opt-in adaptive planning (`--auto-plan`) via cheap sampling
4. Policy packs (profiles)
5. Incremental tail-frame checkpointing
6. Near-duplicate layout optimization (planner module)

## Failure-Domain Validation
- controlled datasets
- comparative baseline experiments (7z / zip / tar+zstd)
- recorded results with no overstated claims

## Packaging / Distribution
- optional detachable SFX wrapper that can be stripped off in-place to recover the raw `.crushr` archive

## Market / Positioning Notes
- CI/CD artifact integrity and gaming asset pipelines are promising later positioning angles
