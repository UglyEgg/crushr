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
7. `docs/ARCHITECTURE.md`
8. `docs/SNAPSHOT_FORMAT.md`
9. `docs/CONTRACTS/*`
10. `docs/RESEARCH/*`

If conflict remains, stop and resolve explicitly.

## Boundaries

AI contributors must not:

- reintroduce recovery/salvage/reconstruction product surfaces
- treat legacy monolith behavior as canonical direction
- silently broaden scope beyond the packet
- alter thesis/scope/format contracts without explicit decision logging

## Required output discipline

Each implementation response must include:

1. what changed
2. files modified/deleted
3. verification commands and outcomes
4. remaining risks or blockers
5. completion status

## Current direction

- Phase 1 complete.
- Phase 2 active.
- Next packet: Phase 2.1 controlled corruption matrix manifest/schema.
