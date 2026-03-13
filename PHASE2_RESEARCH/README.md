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

1. Generate manifest (default path shown explicitly): `crushr-lab write-phase2-manifest --output PHASE2_RESEARCH/manifests/phase2_core_manifest.json`
2. Build foundation: `crushr-lab build-phase2-foundation --artifact-dir PHASE2_RESEARCH/generated/foundation`
3. Run execution: `crushr-lab run-phase2-execution --manifest PHASE2_RESEARCH/manifests/phase2_core_manifest.json --foundation-report PHASE2_RESEARCH/generated/foundation/foundation_report.json --artifact-dir PHASE2_RESEARCH/generated/execution`
4. Inspect outputs: raw evidence in `PHASE2_RESEARCH/generated/execution/`, normalized mappings in `PHASE2_RESEARCH/normalized/`, summaries in `PHASE2_RESEARCH/summaries/`
5. Pre-trial audit is the next planned gate and is not yet part of the implemented CLI in this snapshot.

## Policy boundary

Product-facing docs remain under `docs/`.
Generated Phase 2 research state and artifacts must not be written into `docs/`.

## Experimental Evidence Model

The Phase-2 trials generate a complete experimental evidence corpus.

Each scenario is defined by a deterministic manifest entry:

- dataset
- archive format
- corruption type
- corruption target
- corruption magnitude
- seed

From this manifest the system produces:

1. raw execution records
2. normalized result records
3. audit reports verifying trial completeness

All outputs are machine-readable.

This ensures that:

- every result in the white paper can be traced to raw data
- missing runs are detected
- the experiment can be rerun in the future

This design intentionally prioritizes reproducibility over convenience.
