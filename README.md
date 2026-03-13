AI contributors should begin with `AI_BOOTSTRAP.md`.

# crushr

crushr is an integrity-first archival compression container and tool suite.

It is built around one core promise: when corruption occurs, the system reports what is trustworthy and what is not, deterministically.

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
11. `docs/SNAPSHOT_FORMAT.md`
12. `docs/CONTRACTS/README.md`
13. `PHASE2_RESEARCH/README.md`

## What is active

- Phase 1 is complete.
- Active focus is Phase 2 comparative corruption research, with canonical Phase 2 materials rooted at `PHASE2_RESEARCH/`.
- Next packet is Phase 2.1 controlled corruption matrix manifest/schema.

## Tooling surface

- `crushr-pack` — writes archives
- `crushr-info` — read-only structural/reporting views
- `crushr-fsck` — read/verify impact analysis and bounded diagnostics
- `crushr-extract` — strict extraction of verified-safe files
- `crushr-lab` — deterministic experiment harness support

## Non-goals

- recovery/salvage/reconstruction workflows
- parity-based repair systems
- generic replacement for zip/7z
