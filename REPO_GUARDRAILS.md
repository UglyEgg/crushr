# crushr Repository Guardrails

This file prevents documentation and implementation drift.

## Canonical startup reading order

Before any changes, read in this order:

1. `AGENTS.md`
2. `AI_BOOTSTRAP.md`
3. `PROJECT_STATE.md`
4. `.ai/INDEX.md`
5. `.ai/STATUS.md`
6. `.ai/PHASE_PLAN.md`
7. `.ai/DECISION_LOG.md`
8. Active task packet and affected contracts/spec docs

## Canonical truth hierarchy

If documents conflict, resolve with this order:

1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `.ai/PHASE_PLAN.md`
5. `PROJECT_STATE.md`
6. `SPEC.md`
7. `docs/whitepaper/index.md`
8. `docs/foundational_docs/index.md`
9. `.ai/contracts/*`

If conflict remains, stop and resolve explicitly.

## Boundaries

AI contributors must not:

- reintroduce recovery/salvage/reconstruction into `crushr-extract`
- weaken strict extraction semantics or redefine the integrity-first thesis
- silently broaden scope beyond the packet
- alter thesis/scope/format contracts without explicit decision logging

A separate experimental executable `crushr-salvage` is allowed, but it must remain clearly outside canonical extraction semantics.

## Documentation rule

`docs/` is for the website only.
Internal contracts and project-control docs belong under `.ai/` or the repo root.
