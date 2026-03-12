# crushr Repository Guardrails

This file exists to prevent AI-assisted development drift.

## Canonical reading order

Before any code changes, an AI instance must read:

1. `AI_BOOTSTRAP.md`
2. `PROJECT_STATE.md`
3. `.ai/STATUS.md`
4. `.ai/PHASE_PLAN.md`
5. `docs/CONTRACTS/PROJECT_SCOPE.md`
6. `docs/CONTRACTS/ERROR_MODEL.md`
7. `docs/RESEARCH/FAILURE_DOMAIN_FORMALIZATION.md`

No implementation work may begin until the AI can summarize:

- current product surface
- current active step
- current thesis
- current limitations

## Canonical truth hierarchy

If files conflict, use this order:

1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `.ai/PHASE_PLAN.md`
5. `PROJECT_STATE.md`
6. `docs/CONTRACTS/*`
7. `docs/RESEARCH/*`

If a conflict is detected:

- stop
- report the conflict
- do not continue implementation until resolved

## Forbidden AI behavior

AI instances must not:

- present legacy salvage/recovery surfaces as canonical Phase 2 workflow
- treat legacy monolith code as canonical product surface
- change the project thesis without explicit user approval
- silently broaden scope
- update docs to justify code drift
- implement reconstruction/parity behavior

## Required implementation discipline

Every bounded implementation task must:

- name the active step
- state what is in scope
- state what is out of scope
- identify affected files
- include verification commands
- update `.ai/STATUS.md` and `.ai/CHANGELOG.md` only if the task completes

## Required post-task output

Every AI implementation response must provide:

1. what changed
2. files modified
3. tests run
4. remaining blockers
5. whether the task is complete or partial

## Research boundary

crushr is an integrity-first archival container and research artifact.

It is not:

- a parity system
- a speculative recovery system
- a general-purpose replacement for zip/7z

Current white-paper-aligned goals are:

1. bounded failure domains
2. deterministic corruption impact enumeration
3. maximum safe extraction
4. reproducible corruption experiments
