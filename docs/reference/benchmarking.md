<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Benchmark contract (v0.4.17)

This page defines the locked benchmark methodology for crushr.

It is intentionally about **measurement contract**, not optimization outcomes.

## Principles

1. Benchmarks must be reproducible.
2. Benchmark runs must be attributable to dataset, tool/profile, command, and environment.
3. Preservation profile must be explicit for every crushr run.
4. Results are recorded exactly as observed; no silent exclusions.

## Dataset classes

Deterministic datasets are generated through the canonical harness entrypoint:

- `scripts/benchmark/harness.py datasets`

Legacy direct script paths (`generate_datasets.py`, `run_benchmarks.py`) are compatibility shims; use `harness.py` for reproducible operations and docs parity.

Generated under `.bench/datasets/`:

1. `small_mixed_tree`
   - hundreds to low thousands of files
   - mixed file sizes
   - empty directories
   - symlinks
   - xattrs when supported by host/filesystem
2. `medium_realistic_tree`
   - tens of thousands of files
   - mixed text/binary content
   - nested project-like trees
3. `large_stress_tree`
   - high file count plus repeated large binaries
   - designed to surface scaling behavior and memory pressure

Determinism controls:

- fixed generation seed
- deterministic payload bytes from digest expansion
- fixed mtime for generated files/directories
- emitted `dataset_manifest.json` with counts/byte totals

## Comparison set and commands

The benchmark harness executes a centralized comparator set from `scripts/benchmark/contract.py`:

Baseline (always):
- `tar + zstd` (`zstd -3`)
- `tar + xz` (`xz -3`)
- `crushr pack --preservation full --level 3`
- `crushr pack --preservation basic --level 3`

Optional experiment comparator (enabled via harness flags):
- `tar + zstd` with a deterministic trained dictionary (`tar_zstd_dict`)

The comparator set, compression level, dataset names, and dictionary experiment model are centralized and used by both run orchestration and benchmark assumptions fingerprinting.

Canonical command forms used by the harness:

- `tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --pax-option=delete=atime,delete=ctime -I 'zstd -3' -cf <archive.tar.zst> <dataset>`
- `tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --pax-option=delete=atime,delete=ctime -I 'xz -3' -cf <archive.tar.xz> <dataset>`
- `crushr pack <dataset> -o <archive.crs> --level 3 --preservation <full|basic> --silent`

Extraction command forms:

- `tar -xf <archive.tar.zst|archive.tar.xz> -C <out_dir>`
- `crushr extract <archive.crs> -o <out_dir> --all --overwrite --silent`

## Metrics

Required:

1. `archive_size_bytes`
2. `pack_time_ms` (wall clock)
3. `extract_time_ms` (wall clock)
4. peak memory (`pack_peak_rss_kb`, `extract_peak_rss_kb`)

Optional (captured when available):

- CPU timings (`*_user_time_ms`, `*_sys_time_ms`)

## Result format

Structured output:

- JSON file produced by `scripts/benchmark/run_benchmarks.py`
- schema: `schemas/crushr-benchmark-run.v1.schema.json`

Top-level benchmark output also includes:

- `dataset_manifest` (embedded dataset identity, generation controls, and counts)
- `assumptions` (level, comparator set, deterministic command-set fingerprint, and dictionary experiment config)
- `dictionary_artifacts` (deterministic dictionary identity/provenance records for experiment cohorts)

Each run record includes:

- dataset
- tool
- profile (`full`/`basic` for crushr, `null` for tar baselines)
- `comparator_label` (explicit comparator identity in comparisons)
- exact pack/extract command strings
- archive path + size
- timing + peak RSS fields
- `dictionary` metadata (enabled/disabled, dictionary identity hash, cohort label, training provenance summary, dependency marker)

## Reproducibility steps

From repo root:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/harness.py full \
  --clean \
  --datasets .bench/datasets \
  --crushr-bin target/release/crushr \
  --output .bench/results/benchmark_results.json
```

Environment assumptions:

- Linux host
- GNU tar with `--sort`/`--pax-option`
- `zstd`, `xz`, and `time` available in `PATH`
- filesystem with symlink support
- xattrs are disabled by default (`--xattrs off`) for host-independent dataset identity
- optional xattr-inclusive runs must set `--xattrs on`, which changes dataset identity and should not be mixed with default results

## Pack phase attribution (v0.4.17+)

`crushr pack` supports explicit pack-phase timing output with `--profile-pack`.

This is attribution-only instrumentation for local investigation; it is not a benchmark-score mode and is never enabled by default.

### Commands for local operator runs (Rich)

From repo root:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/generate_datasets.py --clean --output .bench/datasets
```

1) Medium dataset (`full` profile):

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_full.txt
```

2) Medium dataset (`basic` profile):

```bash
target/release/crushr pack .bench/datasets/medium_realistic_tree \
  -o .bench/results/medium_realistic_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_medium_basic.txt
```

3) Large dataset (`full` profile):

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.full.profiled.crs \
  --level 3 \
  --preservation full \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_full.txt
```

4) Large dataset (`basic` profile):

```bash
target/release/crushr pack .bench/datasets/large_stress_tree \
  -o .bench/results/large_stress_tree.basic.profiled.crs \
  --level 3 \
  --preservation basic \
  --profile-pack \
  --silent | tee .bench/results/pack_phases_large_basic.txt
```

### Expected output shape

`--profile-pack` appends a deterministic phase table after normal pack completion:

```text
Pack phases
  discovery         <ms>
  metadata          <ms>
  hashing           <ms>
  compression       <ms>
  emission          <ms>
  finalization      <ms>
```

Capture the full command stdout in `.bench/results/pack_phases_*.txt`.

### Interpretation hints

- `compression` dominant: codec work is primary pack bottleneck.
- `hashing` dominant: digest work is disproportionate vs compression.
- `metadata` dominant: input walk/planning and metadata capture are likely scaling poorly.
- `finalization` dominant: tail/index closeout may be doing too much late work.
- Large `full` vs `basic` attribution gaps suggest metadata-envelope cost concentration.

## Limitations

- Peak RSS and CPU fields depend on `time` implementation on host.
- xattr coverage is best-effort and may be partial/non-zero only on supporting filesystems.
- Raw benchmark outputs are not performance claims until reviewed comparatively.

## Dictionary experiment mode (benchmark-only)

Dictionary mode is a controlled benchmark experiment path. It does **not** change archive format/runtime behavior.

Enable it from the canonical harness command surface:

```bash
python3 scripts/benchmark/harness.py run \
  --datasets .bench/datasets \
  --crushr-bin target/release/crushr \
  --output .bench/results/benchmark_results.dict.json \
  --dictionary-experiment on \
  --dictionary-scope per_dataset
```

Determinism and provenance model:

- training input selection is explicit and deterministic (`lexicographic_relative_path`, then capped by `--dictionary-max-samples`)
- per-sample read size is explicit (`--dictionary-sample-bytes`)
- dictionary size target is explicit (`--dictionary-size-bytes`)
- each trained dictionary has:
  - `dictionary_content_hash` (content digest)
  - `dictionary_id` (stable identity derived from scope/cohort/training manifest + content hash)
  - `cohort_label` and `cohort_datasets`
  - `training_manifest_id` and per-sample digest list

Non-goals for this packet:

- no archive-format dictionary dependency semantics
- no runtime/archive pack/extract behavior changes
- no silent fallback; dictionary experiment runs carry explicit dependency metadata (`required_dictionary`)

Design framing for follow-up runtime/archive work (pending benchmark evidence):

- dictionary identity must stay explicit
- dependency edges must stay explicit and auditable
- survivability should favor payload-adjacent or checkpoint-local placements over hidden centralized dictionary dependencies
