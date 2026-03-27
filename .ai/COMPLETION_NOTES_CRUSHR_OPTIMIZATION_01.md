<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# COMPLETION_NOTES — CRUSHR_OPTIMIZATION_01

Date: 2026-03-27 (UTC)

## Root-cause summary

Discovery was doing profile-agnostic metadata work for every walk entry and then discarding much of it in `apply_preservation_profile`.

Dominant avoidable costs identified:

1. Eager xattr/security metadata probes in discovery even for `basic` and `payload-only`.
2. Eager ownership-name resolution (`getpwuid`/`getgrgid`) per entry.
3. Eager sparse probing for `payload-only` entries even though sparse metadata is intentionally omitted there.
4. Duplicate regular-file `stat` work (`collect_files` and `build_pack_layout_plan`).

## What changed

- Discovery is now profile-aware up front (`collect_files(inputs, profile)`), with explicit capture policy per profile.
- Omitted entry kinds are filtered during discovery with the same warning semantics instead of being fully captured first.
- Metadata classes omitted by profile are not captured during discovery:
  - `basic`: ownership names, ownership IDs, xattrs/security metadata
  - `payload-only`: ownership, xattrs/security metadata, sparse metadata, symlink/special entries
- Regular-file planned length now reuses discovery-captured `raw_len` and no longer re-stats in layout planning.
- UID/GID name lookup now uses in-memory per-run caches to avoid repeated lookups for identical IDs.

## Truthfulness / phase accounting

No profiling relabel was introduced.

- Work genuinely removed from discovery is no longer executed.
- The remaining phases still measure the same pipeline boundaries (`discovery`, `metadata`, `hashing`, `compression`, `emission`, `finalization`).

## Remaining intentionally broad discovery work

Some discovery operations remain intentionally broad because they are required by active semantics:

1. `symlink_metadata` per entry to determine file kind and obtain required mutation-relevant metadata.
2. Full walk enumeration to preserve deterministic path ordering and duplicate-logical-path detection correctness.
3. Sparse detection for `full`/`basic` regular files (required by those profile semantics).

## Local validation commands for Rich

From repo root:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets
mkdir -p .bench/results
```

1) medium dataset, full profile, with pack profiling:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_full.txt
```

2) medium dataset, basic profile, with pack profiling:

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_basic.txt
```

3) large dataset, full profile, with pack profiling:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_full.txt
```

4) large dataset, basic profile, with pack profiling:

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_basic.txt
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

## Success deltas to look for

Primary success signals:

1. Material reduction in `discovery` time (medium and large datasets).
2. `basic` discovery clearly below `full` discovery on the same dataset.
3. Total pack time drops commensurately (not just phase redistribution).

Suggested target expectations (directional, not hard gates):

- medium: discovery reduction in clear double-digit percent range
- large: discovery reduction in meaningful double-digit percent range
- basic-vs-full: basic discovery no longer near-equal or worse than full

## Builder-run validation gates

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `./scripts/check-version-sync.sh`
- `cargo test -p crushr --test version_contract`
