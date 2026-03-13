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
- `docs/RESEARCH/*` — product-level experiment formalization/reference docs (not generated artifacts)
- `PHASE2_RESEARCH/` — canonical Phase 2 research workspace (methodology, manifests, generated outputs, normalized results, summaries, whitepaper support)

## Code and supporting dirs

- `crates/` — Rust workspace crates
- `schemas/` — versioned JSON schemas
- `TASK_PACKETS/` — bounded implementation packets
- `.ai/` — continuity memory (internal project control)

## Current phase direction

- Phase 1 complete
- Phase 2 active
- Next packet: Phase 2.1 controlled corruption matrix manifest/schema
