# .ai/INDEX.md

This directory contains AI-internal continuity documents. These files exist to minimize drift during development and to enable clean handoff to a fresh AI instance.

## Start Here

1. Read `../AGENTS.md`
2. Read `STATUS.md`
3. Read `DECISION_LOG.md`
4. Read `PHASE_PLAN.md`
5. Read `HANDOFF.md`
6. Read `AGENTS_EXECUTION_ENVIRONMENT_SUPPLEMENT.md` (only if working in a restricted/sandbox environment)

## Canonical Documents

- `STATUS.md` — **single source of truth** for current Phase/Step and active state
- `PHASE_PLAN.md` — current Phase plan + Step checklist
- `DECISION_LOG.md` — resolved decisions (date, decision, alternatives, rationale, blast radius)
- `BACKLOG.md` — deferred work (not currently active)
- `HANDOFF.md` — takeover instructions for a fresh instance
- `CHANGELOG.md` — terse history of completed Steps

## Supplements

- `AGENTS_EXECUTION_ENVIRONMENT_SUPPLEMENT.md` — environment constraints (AI-internal)

## Maintenance Rules

- Update `STATUS.md` at the end of every Step.
- Record every resolved decision in `DECISION_LOG.md`.
- Keep `PHASE_PLAN.md` aligned with reality (checkboxes reflect actual completion).
- Keep `HANDOFF.md` concise and actionable (first 5 actions should be obvious).

If any of these documents conflict, stop and resolve explicitly. `STATUS.md` is authoritative for “what is true right now.”
