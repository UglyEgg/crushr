<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

## Current state (authoritative)

- **Current Phase:** Phase 16 — 0.x Benchmarking and Compression Evidence.
- **Current Step:** **CRUSHR_PHASE16_08 complete** (deterministic lightweight content-class clustering experiments are now integrated into benchmark tar comparators with explicit schema metadata).
- **Phase 16 benchmark/tooling status:** deterministic dataset identity, assumptions, dictionary experiment metadata, zstd level/strategy experiment metadata, deterministic ordering/locality experiment metadata, deterministic content-class clustering experiment metadata, zstd strategy-capability-safe command construction, and ordering input-list path validation/rooting are centralized and schema-backed.
- **Runtime/code status in this packet:** benchmark tooling and docs updated; no archive format or runtime extraction/pack semantics changed.

## What is now true in code (benchmark/tooling truth)

- Benchmark dataset and comparator assumptions are centralized in `scripts/benchmark/contract.py` and consumed by both dataset generation and benchmark execution.
- Dataset generation now has explicit xattr mode control (`--xattrs off|on`, default `off`) and emits a deterministic `dataset_identity` digest in `dataset_manifest.json`.
- Benchmark execution now requires/embeds `dataset_manifest.json`, records normalized `assumptions` metadata (`level`, comparator set, command-set fingerprint, dictionary experiment config, zstd level/strategy experiment config, ordering experiment config, content-class experiment config), and emits dictionary artifact provenance plus per-run dictionary dependency/zstd/ordering/content-class metadata in a consistent schema envelope.
- zstd tar comparator command construction is now centralized and capability-safe: default strategy omits explicit `--strategy` flags, and non-default strategy requests fail early with a clear host-capability diagnostic when `zstd --strategy=<name>` is unsupported.
- Deterministic ordering/locality strategies for tar comparators (`lexical`, `size_ascending`, `size_descending`, `extension_grouped`, `kind_then_extension`) are centralized and executed through generated explicit tar input-order files.
- Ordering input files now write deterministic benchmark-execution-root-relative paths (for example `datasets/<dataset>/...`), tar invocations use `--verbatim-files-from`, and the harness fails early with explicit diagnostics for empty/malformed/unresolvable ordering files before tar execution.
- Ordering strategy matrices now expand to independent tar comparator variants for each requested strategy (across `tar_zstd`, `tar_xz`, and dictionary tar-zstd when enabled), with strategy-distinct comparator labels and archive filenames to prevent silent lexical-only collapse.
- Benchmark execution now enforces ordering-matrix sanity: if more than one strategy is requested, comparator expansion and final run output must contain more than one tar ordering strategy or the harness fails early.
- Lightweight deterministic content-class clustering (`off|lightweight_v1`) is now available for tar comparators only, and comparator labels include explicit clustering mode (`_cc<strategy>`) for auditability.
- Canonical benchmark command surface is `scripts/benchmark/harness.py` (`datasets`, `run`, `full`) including dictionary, zstd, ordering, and content-class experiment flags to reduce invocation drift between docs and operations.

## Open debt (intentional / deferred)

1. **Benchmark environment portability debt:** benchmark runner still requires `tar`, `xz`, and `zstd` binaries in PATH.
2. **Experimental metadata pruning direction:** FORMAT-10/11/12/13/14A/15 evidence review remains planning input, not product-surface runtime work.
3. **Long-range platform work:** Phase 17+ roadmap items (1.x stabilization, evidence/custody layer) remain future work.

## Next permitted workstream

- **Permitted next action:** execute baseline + dictionary + zstd level/strategy + ordering/locality + content-class clustering benchmark matrices in an environment with full comparator dependencies present and publish updated evidence artifacts from normalized schema.
- Future packets may assume:
  - benchmark dataset/comparator assumptions are centralized in `scripts/benchmark/contract.py`,
  - `scripts/benchmark/harness.py` is the canonical benchmark command surface,
  - benchmark output includes embedded dataset identity + assumptions metadata + dictionary provenance metadata + zstd level/strategy metadata + ordering strategy metadata + content-class strategy/classification metadata,
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
