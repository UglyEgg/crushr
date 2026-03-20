<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Repository Layout and Source of Truth

This document defines where active truth lives.

## Canonical control files

- `AGENTS.md` — durable repo operating contract
- `AI_BOOTSTRAP.md` — startup checklist
- `REPO_GUARDRAILS.md` — implementation guardrails
- `PROJECT_STATE.md` — concise current-state summary
- `.ai/STATUS.md` — single source of truth for current phase/step and next action
- `.ai/PHASE_PLAN.md` — active phase checklist
- `.ai/DECISION_LOG.md` — resolved decisions

## Public product/spec docs

The website-facing documentation lives only under `docs/`.

Primary entry points:

- `docs/index.md`
- `docs/why-crushr.md`
- `docs/whitepaper/index.md`
- `docs/foundational_docs/index.md`

## Internal contracts and project-control docs

- `.ai/contracts/` — policy contracts, quality gates, and interface/reference contracts used for internal development and review
- `.ai/` — continuity memory and project-control material

## Code and supporting dirs

- `schemas/` — versioned JSON schemas
- `docs/` — website only
- `.ai/` — internal project control

## Documentation rule

If a document is not part of the website, it must not live in `docs/`.
