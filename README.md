AI contributors should begin with `AI_BOOTSTRAP.md`.

# crushr

crushr is an integrity-first archival compression container and tool suite.

It is built around one core promise: when corruption occurs, the system reports what is trustworthy and what is not, deterministically.

This repository contains the baseline crushr format evaluated in the Phase-2 white-paper trials.

## Read first

Canonical reading/authority path:

1. `AGENTS.md`
2. `AI_BOOTSTRAP.md`
3. `REPO_GUARDRAILS.md`
4. `PROJECT_STATE.md`
5. `.ai/INDEX.md`
6. `.ai/STATUS.md`
7. `.ai/PHASE_PLAN.md`
8. `.ai/DECISION_LOG.md`
9. `SPEC.md`
10. `docs/ARCHITECTURE.md`
11. `docs/ARCHITECTURE_V2.md`
12. `docs/SNAPSHOT_FORMAT.md`
13. `docs/CONTRACTS/README.md`
14. `PHASE2_RESEARCH/README.md`
15. `PHASE2_RESEARCH/methodology/PHASE2_LOCKS.md`
16. `docs/ROADMAP.md`

## What is active

- Phase 1 is complete.
- Active focus is Phase 2 comparative corruption research, with canonical Phase 2 materials rooted at `PHASE2_RESEARCH/`.
- Next required milestone is the Phase 2 pre-trial audit pass under `PHASE2_RESEARCH/`.
- Next packet after audit is Phase 2.2 cross-format comparison and normalized result mapping.

## Tooling surface

- `crushr-pack` — writes archives
- `crushr-info` — read-only structural/reporting views
- `crushr-fsck` — read/verify impact analysis and bounded diagnostics
- `crushr-extract` — strict extraction of verified-safe files
- `crushr-lab` — deterministic experiment harness support

## Non-goals

- recovery/salvage/reconstruction workflows
- parity-based repair systems
- generic replacement for zip or tar-family formats

## White-paper baseline and future direction

The white-paper trials evaluate the baseline crushr format as currently implemented.

Planned future capabilities such as recoverable extraction, true random access, and deduplication are intentionally deferred until after the baseline trial phase so the published results remain methodologically clean.

The locked long-term v2 direction is content-addressed block identity with deterministic on-disk indexing. See `docs/ARCHITECTURE_V2.md` and `docs/ROADMAP.md`.
