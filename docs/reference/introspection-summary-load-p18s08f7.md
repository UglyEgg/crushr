# Phase 18 Step 08 fix 7 — WASM initial summary-load characterization

## Scope

This note characterizes the browser/WASM **initial archive summary** path and records a bounded optimization focused on initial load cost.

## Characterization findings (evidence-based)

Code-path review and timing runs show the load-summary path currently does:

1. **Full index decode/parse** during summary (`inspect_archive_bytes` decodes IDX3).  
2. **Archive payload hash verification scan** during summary (`verify_block_payloads_v1`), which reads block payload bytes.  
3. **No search-path state prep** on load (`prepare_loaded_archive_state` is still deferred in worker flow).  
4. **No path index construction / per-entry report materialization** on load (that is in `prepare_introspection_state_bytes` / state build for `find`/`entry`).

The browser-side load path additionally paid a full extra in-WASM byte clone in `archive_summary` before this fix (`archive_bytes.to_vec()`), duplicating large archive bytes during initial load.

## Optimization applied in this step

- Removed the extra full byte clone in WASM `archive_summary` by taking ownership of the archive bytes and reusing the same vector for summary inspection + session storage.
- Kept deferred execution model unchanged:
  - load => summary only
  - first `Find`/entry => lazy state prep
- Kept worker/progress stage behavior unchanged.

## Before/after timing (Node wasm-bindgen harness, same archive set, 1 run)

| Archive class | `archive_summary` before (ms) | after (ms) | delta |
|---|---:|---:|---:|
| `large` | 21.103 | 13.576 | **-35.7%** |
| `very_large_stress` | 89.722 | 62.322 | **-30.5%** |

Artifacts:

- baseline: `.bench/introspection_baseline/wasm_baseline.json`
- post-change: `.bench/introspection_baseline/wasm_baseline_p18s08f7.json`

## Notes

- This packet removes avoidable summary-time memory/transfer work without altering summary semantics.
- Remaining summary load cost is still primarily Rust-side archive summary work (index parse + block verification).
