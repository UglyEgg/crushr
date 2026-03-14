# Phase 2 Normalization Rules (CRUSHR-P2-EXEC-04)

This document defines deterministic rules used by `crushr-lab run-phase2-normalization`.

## Inputs

- `PHASE2_RESEARCH/trials/raw_run_records.json`
- per-scenario `stdout.txt` / `stderr.txt`
- optional per-scenario `result.json`

No trials are rerun.

## Normalized fields

Each normalized record preserves scenario axes and adds:

- `result_class`: `SUCCESS | PARTIAL | REFUSED | STRUCTURAL_FAIL | TOOL_ERROR`
- `failure_stage`: `NONE | PRE_EXTRACT | EXTRACTION | UNKNOWN`
- `diagnostic_specificity`: `NONE | GENERIC | STRUCTURAL | PRECISE`
- `detected_pre_extract`: derived boolean (`failure_stage == PRE_EXTRACT`)
- `files_safe/files_refused/files_unknown`: nullable file-level counts
- `normalization_notes`: deterministic notes about evidence limitations
- `evidence_strength`: `structured_json | stdout_stderr | mixed`

## Deterministic mapping

1. **Failure stage**
   - `exit_code == 0` => `NONE`
   - Non-zero + structural markers (`bad footer magic`, `bad magic`, `hash mismatch`, `missing end signature`, `not a zip file`, `not in gzip format`, `file format not recognized`, `unsupported format`, `unknown header`, `premature end`, `unexpected end`) => `PRE_EXTRACT`
   - Non-zero + extraction markers (`invalid compressed data to inflate`, `skipping to next header`, `bad zipfile offset`, `crc error`, `length error`, `format violated`, `filename too long`) => `EXTRACTION`
   - Otherwise => `UNKNOWN`

2. **Result class**
   - `exit_code == -1` => `TOOL_ERROR`
   - `has_json_result == true` but `json_result_path` missing on disk => `TOOL_ERROR`
   - `exit_code == 0` => `SUCCESS`
   - Non-zero + refusal markers + `failure_stage == EXTRACTION` => `REFUSED`
   - Non-zero + refusal markers (other stages) => `PARTIAL`
   - Non-zero + `failure_stage == PRE_EXTRACT` => `STRUCTURAL_FAIL`
   - Non-zero + `failure_stage == EXTRACTION` => `PARTIAL`
   - Otherwise => `TOOL_ERROR`

3. **Diagnostic specificity**
   - Empty stdout+stderr => `NONE`
   - Markers with concrete scope (`payload/`, `bin/`, `cfg/`, `file #`, `inflate`, `IDX3`, `FTR4`, `header`) => `PRECISE`
   - Structural/family markers (`magic`, `archive`, `format`, `checksum`, `crc`, `corrupt`, `hash mismatch`, `footer`, `index`) => `STRUCTURAL`
   - Otherwise => `GENERIC`

4. **File-level counts**
   - Current Phase 2 corpus has no extraction-result JSON with per-file outcomes.
   - `crushr` JSON artifacts are `crushr-info` structural metadata probes, not extraction outcomes.
   - Therefore all file-level count fields are currently `null` and notes are attached.

## Non-claims

Normalization does **not** infer recovered file counts from unstructured text and does **not** claim extraction success where corpus evidence is structural-only.
