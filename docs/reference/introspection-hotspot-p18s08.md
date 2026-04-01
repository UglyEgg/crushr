# Phase 18 Step 08 — introspection state reuse/caching results

## Scope and method

- Harness: `python3 scripts/perf_introspection_hotspot.py --runs 1`.
- Dataset focus: `large` and `very_large_stress` classes.
- Comparison: cold-call stage timings vs repeated-call medians for `find` and `entry`.

## CLI hotspot summary (ms)

| Class | op | cold total | repeated total | cold summary prep | repeated summary prep |
|---|---|---:|---:|---:|---:|
| large | find | 78.033 | 0.491 | 73.448 | 0.000 |
| very_large_stress | find | 173.698 | 1.135 | 166.186 | 0.000 |
| large | entry | 0.076* | 0.027 | 0.000* | 0.000 |
| very_large_stress | entry | 0.049* | 0.030 | 0.000* | 0.000 |

\* In the current harness sequence, `entry` cold is measured after a `find` cold on the same archive inside one process, so the per-archive state is already warm.

## Findings

1. Repeated `find` no longer pays decoded-index/summary rebuild cost; `summary_index_prep` and `index_decode_parse` are `0.0ms` on repeated calls.
2. Large-archive repeated `find` wall time dropped from ~78–174ms cold to ~0.5–1.1ms repeated in this run.
3. Repeated `entry` also avoids prep/decode rebuild cost (`0.0ms` repeated prep/decode), and is now traversal/materialization-dominated.
4. The shared bounded state reuse is active in both CLI path-based introspection and WASM loaded-session flow.

## Notes

- Cache lifecycle is bounded to a single active archive identity (path + metadata for CLI, loaded session for WASM demo) and is replaced on archive change.
- No cross-archive unbounded cache growth is introduced.
