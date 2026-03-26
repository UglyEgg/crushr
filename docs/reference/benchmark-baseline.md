<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Benchmark baseline (v0.4.15)

## Overview

This baseline run executes the CRUSHR_BENCHMARK_01 matrix once across all deterministic benchmark datasets:

- `tar + zstd` (`zstd -3`)
- `tar + xz` (`xz -3`)
- `crushr pack --preservation full --level 3`
- `crushr pack --preservation basic --level 3`

Payload-only profile was not included in this baseline because it is optional in the packet.

Raw data artifact:

- `docs/reference/benchmarks/benchmark_results_v0.4.15.json`

## Environment summary

- Date (UTC): 2026-03-26
- OS/kernel: Linux 6.12.47 (Ubuntu-based container host)
- CPU: 3 vCPU (`Intel(R) Xeon(R) Platinum 8370C @ 2.80GHz`)
- RAM: 17 GiB
- Filesystem: `ext4`

## Size comparison

Archive size in bytes (lower is better):

| Dataset | tar+zstd | tar+xz | crushr full | crushr basic |
|---|---:|---:|---:|---:|
| small_mixed_tree | 2,393,638 | 2,424,020 | 2,781,351 | 2,767,630 |
| medium_realistic_tree | 68,074,204 | 68,970,396 | 76,923,778 | 76,689,026 |
| large_stress_tree | 524,372,661 | 148,617,692 | 545,119,686 | 544,606,073 |

Key size takeaways:

- `crushr` is larger than both tar baselines on every dataset in this run.
- `crushr basic` is consistently slightly smaller than `crushr full` (small but measurable).
- `tar+xz` dominates size on `large_stress_tree` (very large margin), but pays heavily in pack time.

## Pack time comparison

Pack wall time in ms (lower is better):

| Dataset | tar+zstd | tar+xz | crushr full | crushr basic |
|---|---:|---:|---:|---:|
| small_mixed_tree | 184 | 2,356 | 452 | 481 |
| medium_realistic_tree | 1,846 | 70,704 | 8,617 | 9,277 |
| large_stress_tree | 4,623 | 155,041 | 27,447 | 35,200 |

Pack-time takeaways:

- `tar+zstd` is fastest on all datasets.
- `crushr` is much faster than `tar+xz` for packing, especially on medium/large datasets.
- `crushr basic` is slightly slower than `crushr full` in this run.

## Extract time comparison

Extract wall time in ms (lower is better):

| Dataset | tar+zstd | tar+xz | crushr full | crushr basic |
|---|---:|---:|---:|---:|
| small_mixed_tree | 152 | 324 | 349 | 291 |
| medium_realistic_tree | 3,974 | 10,412 | 7,379 | 7,053 |
| large_stress_tree | 14,964 | 23,881 | 33,187 | 36,478 |

Extract-time takeaways:

- `tar+zstd` is fastest on all datasets.
- `crushr` is faster than `tar+xz` on small and medium extraction, but slower than `tar+xz` on large extraction.
- `crushr basic` is slightly better than full on small/medium extract, but worse on large extract.

## Memory behavior

`peak_rss_kb` values are `null` for all runs in this environment.

Reason: benchmark harness falls back to wall-clock-only timing when GNU `/usr/bin/time` is not installed; this container provides shell-keyword `time` only.

Resulting memory conclusion:

- No valid peak RSS comparison can be made from this baseline run.

## Observations

Where crushr is better:

- Pack speed vs `tar+xz` on all datasets.
- Extract speed vs `tar+xz` on small and medium datasets.

Where crushr is worse:

- Archive size vs both tar baselines on all datasets.
- Pack speed vs `tar+zstd` on all datasets.
- Extract speed vs `tar+zstd` on all datasets.
- Extract speed vs `tar+xz` on `large_stress_tree`.

Where results are roughly equivalent:

- `crushr full` vs `crushr basic` archive sizes are close; basic is consistently slightly smaller.
- `crushr full` vs `crushr basic` pack/extract times are in the same broad range, with dataset-dependent lead changes.

Surprising findings:

- `tar+xz` produced dramatically smaller output on `large_stress_tree` (about 3.7x smaller than `tar+zstd` and `crushr`), but with very high pack-time cost.
- On large extraction only, `tar+xz` outperformed both crushr modes despite being slower than crushr on small/medium extraction.

## Known caveats

- This is a single full-suite run; no statistical confidence interval is claimed.
- Peak RSS is missing in this environment (`/usr/bin/time` unavailable).
- CPU time fields are also absent for the same reason.
- Results are tied to this host profile (3 vCPU, ext4, containerized runtime).
- Dataset representativeness is bounded to current deterministic synthetic families (`small_mixed_tree`, `medium_realistic_tree`, `large_stress_tree`).

## Follow-up attribution status

As of `v0.4.17`, pack-phase attribution is available through `crushr pack --profile-pack` (see `docs/reference/benchmarking.md`) so future benchmark investigations can break pack-time cost down by internal phase rather than treating pack as a single undifferentiated bucket.
