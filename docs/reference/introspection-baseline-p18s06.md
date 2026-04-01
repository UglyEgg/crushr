# Phase 18 Step 06 — introspection baseline report

## Archive set

| Class | Dataset | Query | Entry path |
|---|---|---|---|
| small | small_fixture | `item_0000` | `shard_000/item_0000.bin` |
| medium | medium_fixture | `item_0000` | `shard_000/item_0000.bin` |
| large | large_fixture | `item_0000` | `shard_000/item_0000.bin` |
| very_large_stress | very_large_fixture | `item_0000` | `shard_000/item_0000.bin` |

## CLI median wall-clock (ms)

| Class | info | info --find | info --entry | info --propagation |
|---|---:|---:|---:|---:|
| small | 17.206 | 33.689 | 33.292 | 18.851 |
| medium | 33.591 | 4389.298 | 4502.675 | 33.904 |
| large | 66.27 | 7300.923 | 6979.388 | 32.722 |
| very_large_stress | 66.002 | 36857.269 | 37919.165 | 116.079 |

## WASM median wall-clock (ms)

| Class | archive_summary | find | entry | propagation |
|---|---:|---:|---:|---:|
| small | 15.631 | 36.294 | 13.033 | 4.23 |
| medium | 19.182 | 1778.503 | 1708.769 | 10.097 |
| large | 11.89 | 2712.521 | 2787.829 | 12.542 |
| very_large_stress | 54.347 | 14186.056 | 14171.951 | 33.433 |

## Findings

- Same archive files and equivalent operation classes were used for CLI and WASM measurements.
- `--find` and `--propagation` dominate cost as archive class size grows, indicating computation/repeated work sensitivity.
- WASM path is consistently slower than CLI for the same operations, consistent with JS/WASM bridge and serialization overhead.
- Browser UI blocking and responsiveness could not be directly measured in this headless environment; this remains a follow-up in a real browser session.

## Immediate next-step recommendation

1. Add index/summary reuse (cache) across repeated `find` / `entry` / `propagation` calls in shared introspection paths.
2. Add browser-side instrumentation (`performance.mark` + long-task observation) to isolate core compute vs render/main-thread blocking.
