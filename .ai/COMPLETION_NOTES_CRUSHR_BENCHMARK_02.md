<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_BENCHMARK_02 completion notes

Date: 2026-03-26 (UTC)

## Commands run

1. Build benchmark target binary:
   - `cargo build --release -p crushr`
2. Generate deterministic datasets:
   - `python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets`
3. Initial harness attempt (failed prerequisite):
   - `python3 scripts/benchmark/run_benchmarks.py --datasets .bench/datasets --crushr-bin target/release/crushr --output .bench/results/benchmark_results.json`
   - failure: `required tool not found in PATH: zstd`
4. Install missing runtime comparator tool:
   - `apt-get update && apt-get install -y zstd`
5. Full benchmark matrix run:
   - `python3 scripts/benchmark/run_benchmarks.py --datasets .bench/datasets --crushr-bin target/release/crushr --output .bench/results/benchmark_results.json`
6. JSON schema validation:
   - `python3 -m jsonschema -i .bench/results/benchmark_results.json schemas/crushr-benchmark-run.v1.schema.json`
7. Validation gates:
   - `cargo fmt --all`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `./scripts/check-version-sync.sh`

## Reruns

- Benchmark harness rerun count after successful prerequisites: **0 additional reruns**.
- One initial harness invocation failed before any benchmark data collection due to missing `zstd`; after installing `zstd`, the full matrix completed successfully in one pass.

## Anomalies observed

1. `peak_rss_kb` and CPU timing fields are `null` in produced results.
   - Cause: benchmark harness relies on external GNU `/usr/bin/time`; this environment only provides shell-keyword `time`, so harness falls back to wall-clock-only measurements.
2. `apt-get update` reported a non-fatal 403 for the `mise.jdx.dev` apt source during package index refresh.
   - Impact: none on benchmark execution; Ubuntu repositories remained usable and `zstd` installed successfully.

## Artifacts produced

- `.bench/results/benchmark_results.json` (runtime artifact)
- `docs/reference/benchmarks/benchmark_results_v0.4.15.json` (committed baseline artifact)
- `docs/reference/benchmark-baseline.md` (committed baseline analysis)
