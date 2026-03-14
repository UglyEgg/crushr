# Phase 2 Normalization Rules (CRUSHR-P2-EXEC-06A)

This document defines deterministic rules used by `crushr-lab run-phase2-normalization`.

## Inputs

- `PHASE2_RESEARCH/trials/raw_run_records.json`
- per-scenario `stdout.txt` / `stderr.txt`
- per-scenario extraction tree under `raw/<scenario_id>/extracted`
- per-scenario `recovery_report.json`

No full matrix rerun is performed in this packet.

## Recovery accounting model

Each raw run now includes:

- `extraction_output_dir`
- `recovery_report_path`
- `recovery_accounting`:
  - `files_expected`
  - `files_recovered`
  - `files_missing`
  - `bytes_expected`
  - `bytes_recovered`
  - `recovery_ratio_files`
  - `recovery_ratio_bytes`

### Deterministic file and byte rules

- **Expected files/bytes** come from the dataset inventory (`PHASE2_RESEARCH/datasets/<dataset>/inventory.json`).
- **Recovered file** means a regular file exists at the same relative path in extraction output.
- **Missing file** means no regular file exists at the expected relative path.
- **Zero-byte files** count as recovered only when the expected file is zero-byte and the output file exists.
- **Truncated output** counts as recovered for file-presence, but contributes only `min(actual_size, expected_size)` to `bytes_recovered`.
- **Tool refusal before extraction** produces no files in extraction output; recovered counts remain zero.
- **Partial directory extraction** naturally contributes only files present in the extracted tree.

Content checksum validation is not yet active in the normalization contract; evidence remains file+byte counts only.

## Normalized fields

Each normalized record preserves scenario axes and adds:

- `result_class`: `SUCCESS | PARTIAL | REFUSED | STRUCTURAL_FAIL | TOOL_ERROR`
- `failure_stage`: `NONE | PRE_EXTRACT | EXTRACTION | UNKNOWN`
- `diagnostic_specificity`: `NONE | GENERIC | STRUCTURAL | PRECISE`
- `detected_pre_extract`: derived boolean (`failure_stage == PRE_EXTRACT`)
- `files_expected`, `files_recovered`, `files_missing`
- `bytes_expected`, `bytes_recovered`
- `recovery_ratio_files`, `recovery_ratio_bytes`
- `blast_radius_class`: `NONE | LOCALIZED | PARTIAL_SET | WIDESPREAD | TOTAL`
- `recovery_evidence_strength`: `FILE_PRESENCE_ONLY | FILE_AND_BYTE_COUNTS | FILE_BYTE_AND_CONTENT_VALIDATION`

Current packet emits `FILE_AND_BYTE_COUNTS` (checksum/content validation deferred).

## Blast-radius thresholds

`blast_radius_class` is based on `recovery_ratio_files`:

- `NONE`: `ratio == 1.0`
- `LOCALIZED`: `0.9 <= ratio < 1.0`
- `PARTIAL_SET`: `0.5 <= ratio < 0.9`
- `WIDESPREAD`: `0.0 < ratio < 0.5`
- `TOTAL`: `ratio == 0.0`

## Summary outputs

`normalization_summary.json` now includes:

- per-format average `recovery_ratio_files`
- per-format average `recovery_ratio_bytes`
- per-format blast-radius class counts
- per-corruption-type average `recovery_ratio_files`
- per-target average `recovery_ratio_files`
- count of runs with recovery accounting
- count by recovery evidence strength

## Non-claims

- This packet does not rerun or commit the full 2700-scenario corpus.
- This packet does not claim byte-for-byte content correctness yet.
