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

## Active product/spec docs

- `SPEC.md` — canonical on-disk archive contract
- `docs/ARCHITECTURE.md` — truthful crate/tool architecture boundary
- `docs/SNAPSHOT_FORMAT.md` — canonical snapshot contract boundary
- `docs/CONTRACTS/*` — policy contracts
- `PHASE2_RESEARCH/` — canonical Phase 2 research/work-product workspace (not shipped runtime data)
- `PHASE2_RESEARCH/methodology/*` — active Phase 2 methodology and lock docs

## Code and supporting dirs

- `crates/` — Rust workspace crates
- `schemas/` — versioned JSON schemas
- `.ai/` — continuity memory (internal project control)

## Current phase direction

- Phase 1 complete
- Phase 2 active
- Next required milestone: Phase 2 pre-trial audit over `PHASE2_RESEARCH/` controls and outputs
- Next packet after audit: Phase 2.2 cross-format comparison and normalized result mapping
