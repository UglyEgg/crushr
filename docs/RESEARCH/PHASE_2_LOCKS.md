# Phase 2 Locks

This document locks deterministic Phase 2 experiment structure for reproducibility.

## Locked core matrix axes

- dataset: `smallfiles`, `mixed`, `largefiles`
- format: `crushr`, `tar+zstd`, `zip`, `7z/lzma`
- corruption_type: `bit_flip`, `byte_overwrite`, `zero_fill`, `truncation`, `tail_damage`
- target_class: `header`, `index`, `payload`, `tail`
- magnitude: `1B`, `256B`, `4KB`
- seed: `1337`, `2600`, `65535`

Scenario id format:

`p2-core-{dataset}-{format_id}-{corruption_type}-{target_class}-{magnitude}-{seed}`

## Locked artifact layout

- Manifest: `docs/RESEARCH/artifacts/phase2_core_manifest.json`
- Foundation report: `docs/RESEARCH/artifacts/phase2_foundation/foundation_report.json`
- Execution root: `docs/RESEARCH/artifacts/phase2_execution/`
  - `raw/<scenario_id>/`
    - corrupted archive (`*.corrupt`)
    - `corruption_provenance.json`
    - `stdout.txt`
    - `stderr.txt`
    - `result.json` (when command stdout is JSON)
  - `raw_run_records.json`
  - `completeness_audit.json`
  - `execution_report.json`
