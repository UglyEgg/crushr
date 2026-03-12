AI contributors should begin with `AI_BOOTSTRAP.md`.

# crushr

An integrity-first archival compression container and tool suite.

> A compression format with database-grade integrity and forensic transparency.
>
> It tells you what survived.

crushr is a systems-engineering project focused on **bounded failure domains** and **deterministic corruption impact enumeration**. It is designed to answer an unusually practical question for compression containers: when corruption happens, what exactly is affected, and what remains trustworthy?

This repository is the canonical workspace for the v1 rewrite. The architecture is intentionally split into multiple crates and multiple tools. Legacy implementation code still exists under `crates/crushr/`, but it is not the authority for the format or project direction.

## Read this first

For canonical project state and source-of-truth documents, read in this order:

1. `AGENTS.md`
2. `PROJECT_STATE.md`
3. `.ai/STATUS.md`
4. `.ai/DECISION_LOG.md`
5. `SPEC.md`
6. `docs/ARCHITECTURE.md`
7. `docs/CONTRACTS/README.md`
8. `docs/RESEARCH/FAILURE_DOMAIN_MODEL.md`

## What crushr is

- A formally specified archival container
- A case study in integrity-first compression design
- A tool suite for packing, inspecting, verifying, and analyzing archives
- A research-backed exploration of failure-domain determinism (FDD)

## What crushr is not

- A replacement for zip or 7z
- A parity/reconstruction system
- A speculative recovery tool
- A “compress everything better” benchmark project

## Workspace layout

- `crates/crushr-format/` — byte-level format contracts and strict parsers/encoders
- `crates/crushr-core/` — structural engine, verification, impact enumeration
- `crates/crushr/` — integration crate and legacy implementation surface
- `crates/crushr-cli-common/` — shared CLI primitives
- `crates/crushr-tui/` — live/snapshot TUI skeleton
- `crates/crushr-lab/` — corruption harness and research tooling

## Canonical docs

- `SPEC.md` — on-disk format contract
- `docs/ARCHITECTURE.md` — crate graph and system shape
- `docs/CONTRACTS/` — project scope, stability, error/security/perf contracts
- `docs/RESEARCH/` — formalization, experiment method, results scaffolding

## Status

The project is still in foundation and validation work. Some code exists only as scaffolding, some code is legacy, and some code is canonical rewrite work. Use `PROJECT_STATE.md` and `.ai/STATUS.md` for the current truth.
