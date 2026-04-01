# Phase 18 Step 07 — `find` / `entry` hotspot characterization

## Scope and method

- Archive set: existing baseline `small`, `medium`, `large`, `very_large_stress` fixtures.
- Focus for findings: `large` and `very_large_stress` classes.
- Measurements include cold call and repeated-call medians for both CLI and WASM-byte path.

## CLI repeated-call stage medians (ms)

| Class | op | total | open/read | decode/parse | summary prep | traversal | materialization |
|---|---|---:|---:|---:|---:|---:|---:|
| large | find | 18658.212 | 0.026 | 1.465 | 18656.107 | 0.582 | 0.03 |
| large | entry | 18626.609 | 0.027 | 1.514 | 18624.826 | 0.197 | 0.042 |
| very_large_stress | find | 100604.205 | 0.025 | 3.212 | 100598.644 | 1.435 | 0.886 |
| very_large_stress | entry | 100543.672 | 0.033 | 4.819 | 100537.946 | 0.811 | 0.06 |

## WASM repeated-call medians (ms)

| Class | find wall | entry wall |
|---|---:|---:|
| large | 2661.398 | 2641.637 |
| very_large_stress | 13665.365 | 13725.866 |

## Findings

1. `find` is dominated by summary/index preparation on large archives (CLI very_large_stress median share: 100.0%).
2. `entry` is also dominated by summary/index preparation on large archives (CLI very_large_stress median share: 100.0%).
3. Repeated calls remain close to cold calls for both `find` and `entry`, indicating repeated index+record reconstruction work instead of reuse.
4. Traversal and result materialization are small contributors relative to preparation, even at very_large_stress size.
5. WASM wall-clock remains higher than CLI for the same archive classes, consistent with added byte-bridge/setup overhead on top of shared core work (inference from CLI stage dominance + WASM totals).

## Recommendation for next bounded optimization packet

- Highest-value target: cache/reuse decoded index and entry report surfaces per loaded archive for `find`/`entry` (shared by CLI and WASM).
- Follow-on bounded work should separately measure post-cache bridge/materialization cost, then decide if JS/WASM boundary slimming is the next priority.
