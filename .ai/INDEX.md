<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/INDEX.md

AI continuity workspace for crushr.

## Startup order

1. `../AGENTS.md`
2. `../AI_BOOTSTRAP.md`
3. `../REPO_GUARDRAILS.md`
4. `../PROJECT_STATE.md`
5. `STATUS.md`
6. `PHASE_PLAN.md`
7. `DECISION_LOG.md`
8. `HANDOFF.md`

## Authority map

- `STATUS.md` — single source of truth for current phase/step and immediate next action
- `PHASE_PLAN.md` — phase checklist execution state
- `DECISION_LOG.md` — resolved decisions and rationale
- `CHANGELOG.md` — completed-step history
- `HANDOFF.md` — short takeover instructions
- `BACKLOG.md` — deferred/non-active work only

## Documentation rigor rule

AI-facing control docs must use the same canonical vocabulary as public product docs.

Use:

- **validate** = structural correctness
- **verify** = integrity correctness
- **recover** / **recovery** = explicit bounded non-canonical extraction path
- trust classes:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

Do not reintroduce as current product language:

- salvage
- fsck
- repair
- fixer semantics
- wrapper binaries as canonical product surface
