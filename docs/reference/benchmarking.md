<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Benchmark contract (v0.4.17)

## Intent

This page defines the locked benchmark methodology for crushr.

It describes the measurement contract used to evaluate compression, packing, and extraction behavior. It does not redefine product guarantees, recovery vocabulary, or operator-facing trust classes.

## Guarantees

- Benchmark runs are attributable to dataset, command, comparator, and environment
- Preservation profile is explicit for every crushr benchmark run
- Results are recorded exactly as observed
- Missing metrics are reported explicitly rather than silently omitted
- Benchmark methodology remains reproducible and reviewable

## Behavior

### Principles

1. Benchmarks must be reproducible.
2. Benchmark runs must be attributable to dataset, tool/profile, command, and environment.
3. Preservation profile must be explicit for every crushr run.
4. Results are recorded exactly as observed; no silent exclusions.

### Dataset classes

Deterministic datasets are generated through the canonical harness entrypoint:

- `scripts/benchmark/harness.py datasets`

Legacy direct script paths (`generate_datasets.py`, `run_benchmarks.py`) are compatibility shims; use `harness.py` for reproducible operations and documentation parity.

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
- emitted `dataset_manifest.json` with counts and byte totals

### Comparison set and commands

The benchmark harness executes a centralized comparator set from `scripts/benchmark/contract.py`.

Baseline (always):
- `tar + zstd` (`zstd -3`)
- `tar + xz` (`xz -3`)
- `crushr pack --preservation full --level 3`
- `crushr pack --preservation basic --level 3`

Optional experiment comparators (enabled via harness flags):
- `tar + zstd` with a deterministic trained dictionary (`tar_zstd_dict`)
- `tar + zstd` level variants (controlled by `--zstd-levels`)
- `tar + zstd` strategy variants (controlled by `--zstd-strategies`)
- deterministic file-ordering/locality variants for tar comparators (controlled by `--ordering-strategies`)
- deterministic lightweight content-class clustering for tar comparators (controlled by `--content-class-strategy`)

The comparator set, zstd level and strategy experiment model, dataset names, dictionary experiment model, and content-class experiment model are centralized and used by both run orchestration and benchmark assumptions fingerprinting.

Ordering experiments are centralized in the same model (`lexical`, `size_ascending`, `size_descending`, `extension_grouped`, `kind_then_extension`) and are applied only to tar comparators.

Content-class clustering is benchmark-only and applied only to tar comparators. `lightweight_v1` uses file extension plus a small leading-byte sample to classify files into `structured_text_like`, `text_like`, `binary_like`, or `unknown_mixed`, then keeps deterministic ordering inside each class.

Canonical command forms used by the harness:

- `tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --pax-option=delete=atime,delete=ctime --no-recursion --verbatim-files-from -T <ordered_inputs.txt> -I 'zstd -3' -cf <archive.tar.zst>`
- `tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --pax-option=delete=atime,delete=ctime --no-recursion --verbatim-files-from -T <ordered_inputs.txt> -I 'xz -3' -cf <archive.tar.xz>`
- `crushr pack <dataset> -o <archive.crs> --level 3 --preservation <full|basic> --silent`

Extraction command forms:

- `tar -xf <archive.tar.zst|archive.tar.xz> -C <out_dir>`
- `crushr extract <archive.crs> -o <out_dir> --all --overwrite --silent`

### Metrics

Required:

1. `archive_size_bytes`
2. `pack_time_ms` (wall clock)
3. `extract_time_ms` (wall clock)
4. peak memory (`pack_peak_rss_kb`, `extract_peak_rss_kb`)

Optional (captured when available):

- CPU timings (`*_user_time_ms`, `*_sys_time_ms`)

### Reproducibility steps

From repo root:

```bash
cargo build --release -p crushr
python3 scripts/benchmark/harness.py full \
  --clean \
  --datasets .bench/datasets \
  --crushr-bin target/release/crushr \
  --content-class-strategy lightweight_v1 \
  --output .bench/results/benchmark_results.json
```

Environment assumptions:

- Linux host
- GNU tar with `--sort` / `--pax-option`
- `zstd`, `xz`, and `time` available in `PATH`
- filesystem with symlink support
- xattrs are disabled by default (`--xattrs off`) for host-independent dataset identity
- optional xattr-inclusive runs must set `--xattrs on`, which changes dataset identity and must not be mixed with default results

### Pack phase attribution (v0.4.17+)

`crushr pack` supports explicit pack-phase timing output with `--profile-pack`.

This is attribution-only instrumentation for local investigation. It is not a benchmark-score mode and is never enabled by default.

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

### Limitations

- Peak RSS and CPU fields depend on the `time` implementation on the host
- xattr coverage is best-effort and may be partial on supporting filesystems only
- Raw benchmark outputs are not product claims until reviewed comparatively

## Boundaries / Non-goals

This page does not define recovery semantics, extraction trust classes, or product identity.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies
