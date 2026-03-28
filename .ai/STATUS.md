<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

## Current state (authoritative)

- **Current Phase:** Phase 16 — 0.x Benchmarking and Compression Evidence.
- **Current Step:** **CRUSHR_PHASE16_01 complete** (benchmark/profiling harness normalization for deterministic evaluation).
- **Phase 16 benchmark/tooling status:** deterministic dataset identity and benchmark assumptions are now centralized and schema-backed.
- **Runtime/code status in this packet:** benchmark tooling and docs updated; no archive format or runtime extraction/pack semantics changed.

## What is now true in code (benchmark/tooling truth)

- Benchmark dataset and comparator assumptions are centralized in `scripts/benchmark/contract.py` and consumed by both dataset generation and benchmark execution.
- Dataset generation now has explicit xattr mode control (`--xattrs off|on`, default `off`) and emits a deterministic `dataset_identity` digest in `dataset_manifest.json`.
- Benchmark execution now requires/embeds `dataset_manifest.json`, records normalized `assumptions` metadata (`level`, comparator set, command-set fingerprint), and emits a consistent top-level schema envelope.
- Canonical benchmark command surface is `scripts/benchmark/harness.py` (`datasets`, `run`, `full`) to reduce invocation drift between docs and operations.

## Open debt (intentional / deferred)

1. **Benchmark environment portability debt:** benchmark runner still requires `tar`, `xz`, and `zstd` binaries in PATH; this environment lacked `zstd` for full matrix execution.
2. **Experimental metadata pruning direction:** FORMAT-10/11/12/13/14A/15 evidence review remains planning input, not product-surface runtime work.
3. **Long-range platform work:** Phase 17+ roadmap items (1.x stabilization, evidence/custody layer) remain future work.

## Next permitted workstream

- **Permitted next action:** execute benchmark matrix in an environment with full comparator dependencies present and publish updated baseline artifacts from normalized schema.
- Future packets may assume:
  - benchmark dataset/comparator assumptions are centralized in `scripts/benchmark/contract.py`,
  - `scripts/benchmark/harness.py` is the canonical benchmark command surface,
  - benchmark output includes embedded dataset identity + assumptions metadata,
  - xattr-inclusive datasets are opt-in and produce different dataset identities.

## Active constraints

- Workspace crate policy remains locked: resolver `3`, edition `2024`, MSRV `1.88`; publish intent rules remain enforced.
- Policy gates remain active (secrets/audit/MSRV/style/version drift).
- `crushr-extract` remains integrity-first strict canonical extraction; no speculative reconstruction.
- `crushr-extract --recover` remains explicitly trust-segregated and non-canonical.
- `crushr-salvage` remains research-only output.
- Do not rerun or broaden expensive full matrix comparison workloads unless explicitly requested.

## Historical notes

- Full packet-by-packet chronology remains in `.ai/CHANGELOG.md`.
- Architectural/policy decisions remain in `.ai/DECISION_LOG.md`.
