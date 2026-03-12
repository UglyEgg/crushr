# Repository Layout and Source of Truth

This document exists to eliminate ambiguity about where files belong and which files are authoritative.

## Top-level canonical files

- `AGENTS.md` — repository operating contract for implementation agents
- `PROJECT_STATE.md` — concise current truth of the project
- `SPEC.md` — canonical on-disk format contract
- `README.md` — project-facing overview
- `CONTRIBUTING.md` — contributor and quality-gate entry point
- `TASK_PACKET_TEMPLATE.md` — template for bounded implementation packets
- `REVIEW_CHECKLIST.md` — hostile review checklist used for acceptance

## Canonical directories

- `.ai/` — internal continuity and control documents
- `docs/` — architecture, contracts, research, and other human-facing technical docs
- `schemas/` — versioned JSON schema contracts
- `TASK_PACKETS/` — bounded implementation packets for Codex/Context
- `crates/` — Rust workspace crates
- `.github/workflows/` — workspace-level CI

## Crate ownership

- `crates/crushr-format/` — byte layout, parsers, encoders, strict invariants
- `crates/crushr-core/` — structural engine, verification logic, impact enumeration, snapshots
- `crates/crushr/` — integration crate; legacy implementation currently lives here until refactor/rewrite is complete
- `crates/crushr-cli-common/` — shared CLI args, output, exit-code helpers
- `crates/crushr-tui/` — TUI skeleton and eventual structural explorer
- `crates/crushr-lab/` — deterministic research and corruption-harness tooling; not product surface area

## Legacy material

- `docs/legacy/` — preserved historical or transitional docs; not source of truth
- `.ai/imported_crushr/` — imported continuity material retained for provenance only

## Explicit non-canonical items

These may exist for historical reasons but are not source-of-truth:
- anything under `docs/legacy/`
- anything under `.ai/imported_crushr/`
- any prior single-crate assumptions inside `crates/crushr/` until migrated into the workspace design

## CI location

The canonical CI workflow is at:
- `.github/workflows/ci.yml`

There should be no parallel workspace CI definitions under crate subdirectories.
