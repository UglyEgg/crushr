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

## Phase 2 operator path

1. Generate manifest: `crushr-lab write-phase2-manifest --output PHASE2_RESEARCH/manifests/phase2_manifest.json`
2. Build foundation: `crushr-lab build-phase2-foundation --manifest PHASE2_RESEARCH/manifests/phase2_manifest.json`
3. Run pre-trial audit: verify `PHASE2_RESEARCH/generated/foundation/foundation_report.json` and output path readiness before execution
4. Run execution: `crushr-lab run-phase2-execution --manifest PHASE2_RESEARCH/manifests/phase2_manifest.json --foundation-report PHASE2_RESEARCH/generated/foundation/foundation_report.json`
5. Inspect outputs: raw evidence in `PHASE2_RESEARCH/generated/execution/`, normalized mappings in `PHASE2_RESEARCH/normalized/`, summaries in `PHASE2_RESEARCH/summaries/`

## Policy boundary

Product-facing docs remain under `docs/`.
Generated Phase 2 research state and artifacts must not be written into `docs/`.
