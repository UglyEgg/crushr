<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# COMPLETION_NOTES — CRUSHR_OPTIMIZATION_03

Date: 2026-03-27 (UTC)

## Root-cause summary

After CRUSHR_OPTIMIZATION_01 and CRUSHR_OPTIMIZATION_02, profiling showed `compression` remained the dominant `pack` phase.

Dominant avoidable costs targeted in this packet:

1. Per-unit zstd stream encoder creation/finish overhead (`zstd::Encoder::new` + setup + `finish`) for every payload and metadata compression call.
2. Remaining compression-path setup churn around repeated context initialization despite prior output-buffer reuse.
3. Repeated compression output lifecycle overhead that could be handled through a reusable compressor-owned buffer.

## What was optimized

- Replaced per-call stream compression setup with a reusable per-run `zstd::bulk::Compressor` context (`DeterministicCompressor`) used by both payload and metadata block compression paths.
- Kept deterministic zstd flags explicitly configured on the reusable context:
  - checksum disabled
  - content size enabled
  - dict id disabled
- Switched compression writes to `compress_to_buffer` into a reusable compressor-owned output buffer to minimize allocation/setup churn in the hot path.

## Correctness / determinism guardrails status

Explicitly unchanged in this packet:

- compression method remains `zstd`
- compression level behavior/default remains unchanged (`--level`, default `3`)
- deterministic serial processing/order behavior in `pack`
- fail-closed mutation detection (`input changed during pack planning`)
- hashing/integrity metadata generation
- archive layout/finalization semantics (headers, tail/index closeout)
- preservation profile semantics
- `--profile-pack` phase labels and attribution boundaries remain truthful (`compression`, `emission`, etc.)

## Builder validation commands run

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./scripts/check-version-sync.sh`
- `cargo test -p crushr --test version_contract`

## Local validation commands for Rich

From repo root:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets
mkdir -p .bench/results .bench/extract_check
```

1) medium dataset, full profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_full.v0.4.20.txt
```

2) medium dataset, basic profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_basic.v0.4.20.txt
```

3) large dataset, full profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_full.v0.4.20.txt
```

4) large dataset, basic profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_basic.v0.4.20.txt
```

Correctness spot checks:

5) archive inspection:

```bash
target/release/crushr info <archive>
```

6) strict verification:

```bash
target/release/crushr verify <archive>
```

7) extraction sanity (representative profile/dataset pair minimum):

```bash
mkdir -p .bench/extract_check/<name>
target/release/crushr extract <archive> --all --output .bench/extract_check/<name>
```

## Expected success indicators

- `compression` phase drops materially on medium and large datasets versus v0.4.19 at the same dataset/profile/level.
- total pack wall-clock drops materially with similar phase shape (no artificial relabeling).
- archive size remains in-family for same inputs/profile/level.
- `info`, `verify`, and extraction sanity checks remain normal.

## Suspicious / red-flag conditions

- notable speedup paired with meaningful archive-size drift (without a clear deterministic explanation).
- `compression` phase drop that is mostly offset by unexplained growth in another phase with little/no total-time gain.
- verify/extract behavior changes, especially new failures or metadata-behavior drift.
- implausibly similar full/basic behavior where metadata envelope differences should still be visible.
