# PHASE2_RESEARCH

Canonical workspace for all Phase 2 research materials.

## Directory layout

- `methodology/` — lock files and execution methodology (canonical `PHASE2_LOCKS.md` lives here)
- `manifests/` — generated and curated Phase 2 manifest inputs
- `generated/` — deterministic pipeline outputs
  - `foundation/` — dataset fixtures, archive builds, and `foundation_report.json`
  - `execution/` — raw run artifacts and `execution_report.json`
- `normalized/` — normalized cross-tool result contracts and mapped records
- `summaries/` — aggregate tables/CSVs and publication-facing summary views
- `whitepaper_support/` — figures, tables, and source support files for whitepaper claims

## Policy boundary

Product-facing docs remain under `docs/`.
Generated Phase 2 research state and artifacts must not be written into `docs/`.
