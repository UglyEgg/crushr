<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

## Current state (authoritative)

- **Current Phase:** Phase 16 — 0.x Benchmarking and Compression Evidence.
- **Current Step:** **CRUSHR_PHASE16_04 complete** (zstd strategy execution hardening for host CLI capability differences in benchmark harness).
- **Phase 16 benchmark/tooling status:** deterministic dataset identity, assumptions, dictionary experiment metadata, zstd level/strategy experiment metadata, and zstd strategy-capability-safe command construction are centralized and schema-backed.
- **Runtime/code status in this packet:** benchmark tooling and docs updated; no archive format or runtime extraction/pack semantics changed.

## What is now true in code (benchmark/tooling truth)

- Benchmark dataset and comparator assumptions are centralized in `scripts/benchmark/contract.py` and consumed by both dataset generation and benchmark execution.
- Dataset generation now has explicit xattr mode control (`--xattrs off|on`, default `off`) and emits a deterministic `dataset_identity` digest in `dataset_manifest.json`.
- Benchmark execution now requires/embeds `dataset_manifest.json`, records normalized `assumptions` metadata (`level`, comparator set, command-set fingerprint, dictionary experiment config, zstd level/strategy experiment config), and emits dictionary artifact provenance plus per-run dictionary dependency/zstd metadata in a consistent schema envelope.
- zstd tar comparator command construction is now centralized and capability-safe: default strategy omits explicit `--strategy` flags, and non-default strategy requests fail early with a clear host-capability diagnostic when `zstd --strategy=<name>` is unsupported.
- Canonical benchmark command surface is `scripts/benchmark/harness.py` (`datasets`, `run`, `full`) including dictionary and zstd experiment flags to reduce invocation drift between docs and operations.

## Open debt (intentional / deferred)

1. **Benchmark environment portability debt:** benchmark runner still requires `tar`, `xz`, and `zstd` binaries in PATH; this environment lacked `zstd` for full matrix execution.
2. **Experimental metadata pruning direction:** FORMAT-10/11/12/13/14A/15 evidence review remains planning input, not product-surface runtime work.
3. **Long-range platform work:** Phase 17+ roadmap items (1.x stabilization, evidence/custody layer) remain future work.

## Next permitted workstream

- **Permitted next action:** execute baseline + dictionary + zstd level/strategy experiment benchmark matrices in an environment with full comparator dependencies present and publish updated evidence artifacts from normalized schema.
- Future packets may assume:
  - benchmark dataset/comparator assumptions are centralized in `scripts/benchmark/contract.py`,
  - `scripts/benchmark/harness.py` is the canonical benchmark command surface,
  - benchmark output includes embedded dataset identity + assumptions metadata + dictionary provenance metadata + zstd level/strategy metadata,
  - dictionary experiment results are explicitly distinguishable from non-dictionary runs,
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


Phase 16 operating under integrity-first compression constraint (see DECISION_LOG: CRUSHR_PHASE16_IDENTITY_GUARDRAIL)


Phase 16 dictionary experiments remain benchmark-only; runtime/archive dictionary dependency is out of scope unless future evidence clears the locked evaluation gate.
