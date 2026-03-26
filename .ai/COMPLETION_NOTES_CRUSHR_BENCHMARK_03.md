<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# COMPLETION_NOTES — CRUSHR_BENCHMARK_03

## What changed

- Added production pack attribution surface: `crushr pack --profile-pack`.
- Added deterministic phase totals for:
  - discovery
  - metadata
  - hashing
  - compression
  - emission
  - finalization
- Kept default behavior unchanged: phase output is opt-in only.

## Implementation-boundary explanation

The current production pack pipeline naturally separates into:

1. **discovery**: `collect_files` input walk and metadata capture from filesystem.
2. **metadata**: preservation-profile filtering, duplicate-path guard, and layout planning (`build_pack_layout_plan`).
3. **hashing**: BLAKE3 digest work for payload/raw hashes and metadata/path digest records.
4. **compression**: zstd compression for payload blocks and experimental metadata blocks.
5. **emission**: serialized write operations (BLK3 headers/payload bytes/metadata blocks).
6. **finalization**: tail/index closeout and end-of-run metadata checkpoint/tail emission.

These are measured at real implementation boundaries. No synthetic phase splitting was introduced.

## Operator local validation commands (Rich)

Build and dataset generation:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets
mkdir -p .bench/results
```

1) medium dataset pack attribution (`basic`):

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_basic.txt
```

2) large dataset pack attribution (`basic`):

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_basic.txt
```

3) optional profile comparison (`full` vs `basic` on medium):

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_full.txt
```

## Expected output shape

```text
status=COMPLETE archive=<...> files=<N> time=<...>

Pack phases
  discovery         <ms>
  metadata          <ms>
  hashing           <ms>
  compression       <ms>
  emission          <ms>
  finalization      <ms>
```

## What to capture

- Capture full stdout from each profiled run into `.bench/results/pack_phases_*.txt`.
- Keep the produced `.crs` archive paths alongside logs for traceability.

## Interpretation hints / suspicious patterns

- **compression dominates**: codec work is likely primary optimization target.
- **hashing dominates**: digest pipeline overhead may be unexpectedly high.
- **metadata dominates**: discovery/planning likely scales poorly with tree shape.
- **finalization dominates**: tail/index/closeout work may be too back-loaded.
- **large `full` vs `basic` differences**: metadata envelope work is materially impacting pack cost.

## Validation commands run by builder

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./scripts/check-version-sync.sh`
- `cargo test -p crushr --test version_contract`
