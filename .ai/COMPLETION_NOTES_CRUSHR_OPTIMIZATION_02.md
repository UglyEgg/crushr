<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# COMPLETION_NOTES — CRUSHR_OPTIMIZATION_02

Date: 2026-03-27 (UTC)

## Root-cause summary

After `CRUSHR_OPTIMIZATION_01`, profiling showed compression and emission were now dominant.

Dominant avoidable costs targeted in this packet:

1. Repeated compression output allocations across payload + metadata zstd calls.
2. High small-write overhead in archive emission (header+payload and metadata blocks).
3. Per-block writer-position probing that became unnecessary once emission byte counts were deterministic.

## What was optimized

- Switched archive output to a buffered writer (`BufWriter` with 1 MiB capacity) in production pack emission.
- Added reusable compression scratch buffer for deterministic zstd encoding paths (payload blocks and metadata blocks).
- Replaced `stream_position` sampling with explicit emitted-byte accounting (`BLK3_HEADER_WITH_HASHES_LEN + compressed_len`) to preserve exact block offset truth while keeping buffered writes efficient.

## Correctness guardrail status

Explicitly unchanged in this packet:

- compression codec/method and default level (`zstd`, level default `3`)
- hash computation and integrity metadata
- mutation detection fail-closed check (`input changed during pack planning`)
- manifest/index/tail semantics and finalization path
- preservation profile behavior and omission semantics
- `--profile-pack` phase semantics (`compression`/`emission` timing points remain truthful)

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
mkdir -p .bench/results
```

1) medium dataset, full profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_full.v0.4.19.txt
```

2) medium dataset, basic profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_basic.v0.4.19.txt
```

3) large dataset, full profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_full.v0.4.19.txt
```

4) large dataset, basic profile, with phase attribution:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_basic.v0.4.19.txt
```

Correctness spot checks (run per produced archive, at minimum representative per profile/dataset pair):

5) archive contract/introspection:

```bash
target/release/crushr info <archive>
```

6) strict verification:

```bash
target/release/crushr verify <archive>
```

7) extraction sanity (per pair, or representative subset if full repetition is too costly):

```bash
mkdir -p .bench/extract_check/<name>
target/release/crushr extract <archive> --all --output .bench/extract_check/<name>
```

## Expected success indicators

- `compression` and/or `emission` phase milliseconds decrease meaningfully vs v0.4.18 under same dataset/profile/level.
- total pack wall-clock decreases correspondingly.
- archive size remains in expected range for same input/profile/level (no unexplained ballooning).
- `info`, `verify`, and extraction checks remain successful.

## Red-flag conditions

- large speedup with significant archive-size inflation under unchanged level/profile.
- verify failures or extraction failures on archives that previously passed.
- implausibly similar archive size/behavior between `basic` and `full` where metadata overhead should differ.
- phase shifts that look artificial (e.g., compression drops sharply while finalization/emission rises without total-time gain).
