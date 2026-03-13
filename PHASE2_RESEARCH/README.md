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
- `outputs/` — operator-side machine-readable checkpoints (for example pre-trial audit snapshots)

## Canonical Phase 2 operator flow

1. Generate manifest:
   `crushr-lab write-phase2-manifest --output PHASE2_RESEARCH/manifests/phase2_core_manifest.json`
2. Run pre-trial audit gate:
   `crushr-lab run-phase2-pretrial-audit --manifest PHASE2_RESEARCH/manifests/phase2_core_manifest.json --output PHASE2_RESEARCH/outputs/pretrial_audit.json`
3. Build foundation artifacts:
   `crushr-lab build-phase2-foundation --artifact-dir PHASE2_RESEARCH/generated/foundation`
4. Run execution matrix:
   `crushr-lab run-phase2-execution --manifest PHASE2_RESEARCH/manifests/phase2_core_manifest.json --foundation-report PHASE2_RESEARCH/generated/foundation/foundation_report.json --artifact-dir PHASE2_RESEARCH/generated/execution`
5. Inspect outputs:
   - audit report: `PHASE2_RESEARCH/outputs/pretrial_audit.json`
   - foundation report + fixtures: `PHASE2_RESEARCH/generated/foundation/`
   - raw execution evidence: `PHASE2_RESEARCH/generated/execution/`
   - normalized mappings: `PHASE2_RESEARCH/normalized/`
   - summary tables: `PHASE2_RESEARCH/summaries/`

## Policy boundary

Product-facing docs remain under `docs/`.
Generated Phase 2 research state and artifacts must not be written into `docs/`.
