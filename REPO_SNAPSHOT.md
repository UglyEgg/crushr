# REPO_SNAPSHOT

## Workspace crates
- `crushr-format` — format primitives and validation
- `crushr-core` — engine/data-model layer
- `crushr` — integration crate (legacy implementation in transition)
- `crushr-cli-common` — common CLI behavior skeleton
- `crushr-tui` — live/snapshot interface skeleton
- `crushr-lab` — deterministic corruption harness and research utilities

## Documentation pillars
- `SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/CONTRACTS/*`
- `docs/RESEARCH/*`
- `.ai/*`


## Hygiene note

Root-level documentation and CI are canonical. Misplaced crate-local CI and transitional duplicate docs were removed or moved to legacy in this consolidation pass.
