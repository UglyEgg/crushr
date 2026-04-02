# Phase 18 Step 08 fix 8 — WASM summary-load decomposition and bounded reduction

This packet characterizes the remaining WASM `archive_summary` cost and applies a bounded optimization that keeps summary semantics and deferred execution unchanged.

## What was measured

Using the existing baseline archive set and Node + wasm-bindgen harness (`scripts/perf_wasm_runner.mjs`, `--runs 1`), we measured:

- end-to-end `archive_summary` wall time
- stage decomposition samples captured by a new WASM-only breakdown export:
  - `index_decode_parse`
  - `block_verification_scan`

Artifact:

- `.bench/introspection_baseline/wasm_baseline_p18s08f8.json` (local benchmark artifact)

## Decomposition results (large classes)

| Archive class | archive_summary median (ms) | index decode/parse (ms) | block verification scan (ms) | decode+verify subtotal (ms) |
|---|---:|---:|---:|---:|
| `large` | 11.702 | 0.188 | 4.558 | 4.747 |
| `very_large_stress` | 54.034 | 0.258 | 14.434 | 14.692 |

Interpretation:

- In this measured path, block verification scan clearly dominates index decode/parse.
- Index decode/parse is a minor fraction of summary time on both `large` and `very_large_stress`.

## Optimization applied

Summary verification path now uses a streamlined clean/boolean verifier (`verify_block_payloads_clean_v1`) for summary checks instead of the richer corrupted-block collection path.

Key properties:

- verification meaning is unchanged for summary (`extents_valid` remains true only when all hashed blocks verify)
- no summary fields or strict-extraction-support semantics changed
- deferred execution remains unchanged (`load => summary only`; search/entry prep still lazy)

## Before/after summary timing (same archive classes)

Before values are from P18S08f7 report; after values are from this packet run.

| Archive class | before (P18S08f7) ms | after (P18S08f8) ms | delta |
|---|---:|---:|---:|
| `large` | 13.576 | 11.702 | -1.874 ms (~13.8%) |
| `very_large_stress` | 62.322 | 54.034 | -8.288 ms (~13.3%) |

## Recommendation

After this pass, the dominant remaining summary cost is payload block verification scan work. For the current demo scope, remaining cost is mostly correctness-essential verification plus baseline WASM/runtime overhead; further optimization should be considered lower priority unless UI-level responsiveness goals tighten further.
