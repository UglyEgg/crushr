<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_HOSTILE_REVIEW_01

## 1. Executive judgment

Overall enterprise-readiness assessment: **correctness-focused but structurally overlayered in core product paths**.

The code is not toy-quality; it has serious validation coverage and deterministic contracts. But the structure still shows classic iterative residue:

- preservation-profile semantics are owned in multiple places (`pack` discovery, `pack` post-filter, strict extract, recover extract, info rendering)
- strict and recover extraction duplicate behavior that should be single-authority policy
- pack owns too many concerns (CLI parsing, production pipeline, and experimental metadata machinery)
- benchmark tooling is deterministic but not centrally configured, creating ongoing drift pressure

Major risk themes:

1. **Multiple sources of truth for one concept** (preservation profile + metadata obligations).
2. **Layered patch architecture** where old and new decision points coexist.
3. **High-coupling maintenance zones** (`commands/pack.rs`, extraction metadata logic).
4. **Contract communication drift** where user-facing statements can diverge from real behavior.

Verdict: the system is defensible for functionality and tests, but **not yet enterprise-clean for long-horizon extension** without cleanup packets.

## 2. High-severity findings

### H1 — Pack preservation-profile ownership is split across discovery and post-discovery filtering

**Where**

- `collect_files(inputs, profile)` applies profile-based omission/warnings during input walk.
- `apply_preservation_profile(&mut files, profile)` applies another profile stage after discovery.

**Pattern**

Dual ownership of one policy.

**Why risky**

- Two places can drift in omission semantics and warning behavior.
- Makes profile behavior audit harder because the true authority is not singular.
- Future profile additions must modify multiple gates correctly.

**Cleanup shape**

- One explicit profile projection authority (either fully in discovery or fully in post-capture transform).
- Keep a single warning-emission owner.

---

### H2 — Strict and recover extraction duplicate metadata obligation + restore behavior

**Where**

- `metadata_required_by_profile(...)` exists in both `strict_extract_impl.rs` and `recover_extract_impl.rs`.
- Both modules carry near-parallel restore helpers (`restore_mtime`, `restore_xattrs`, `restore_ownership`, `restore_security_metadata`).

**Pattern**

Parallel implementations for core policy.

**Why risky**

- Any preservation semantic update is now at least two edits + two review surfaces.
- Bug fixes can land in one path and miss the other.
- Trust-class split (strict/refuse vs recover/route) is mixed with duplicated restore mechanics.

**Cleanup shape**

- Shared restoration engine + profile obligation authority.
- Thin strict/recover policy adapters for outcome routing only.

---

### H3 — Recover command computes a pre-analysis and discards it

**Where**

- `commands/extract.rs` runs `run_recovery_analysis(&opts.archive)` then immediately binds outputs to `_`.

**Pattern**

Dead/placeholder orchestration residue.

**Why risky**

- Adds cost and cognitive load with no behavioral authority.
- Suggests a staged design that no longer exists, but code still implies it.

**Cleanup shape**

- Remove pre-analysis from command path, or make it authoritative and consumed by rendering/output logic.

## 3. Medium-severity findings

### M1 — `commands/pack.rs` is an overgrown mixed-responsibility module

**Where**

Single module owns CLI grammar, discovery, planning, compression/emission, finalization, and extensive experimental metadata record writers.

**Pattern**

Monolithic command boundary.

**Why risky**

- High blast radius for any change.
- Production/lab boundaries are harder to reason about.
- Review quality suffers when one file includes many unrelated responsibility layers.

**Cleanup shape**

Split into internal modules by responsibility (`cli`, `discovery`, `planning`, `emit`, `experimental_metadata`).

---

### M2 — Recover metadata-degraded routing clones logic across entry kinds

**Where**

`recover_extract_impl.rs` has repeated “failed metadata → metadata_degraded placement → manifest entry assembly” flow across regular/dir/symlink/special branches.

**Pattern**

Branch-level duplication.

**Why risky**

- Easy to drift in manifest fields or trust-class accounting.
- Every metadata-class adjustment becomes high-touch.

**Cleanup shape**

Shared helper for degraded routing + manifest row assembly.

---

### M3 — Benchmark runner matrix is hard-coded despite manifest output existing

**Where**

- `run_benchmarks.py` hard-codes dataset names and variant matrix.
- `generate_datasets.py` emits `dataset_manifest.json` but the runner does not consume it.

**Pattern**

Config declared in two places.

**Why risky**

- Adds drift channel between generator, docs, and runner.
- Expanding benchmark coverage requires code edits instead of matrix config changes.

**Cleanup shape**

- One benchmark matrix config source (manifest or dedicated benchmark matrix file).

---

### M4 — `info` structural language can overstate 1:1 mapping truth

**Where**

`commands/info.rs` renders `block model = file-level (1:1 file → unit)`.

**Pattern**

Absolute wording for non-absolute behavior.

**Why risky**

Hard-link sharing and mapping semantics can violate naive 1:1 interpretation.

**Cleanup shape**

Compute/report structure from actual mapping facts or soften wording to avoid false absolutes.

## 4. Low-severity findings

### L1 — Stale/legacy comment footprint in index/decode history paths

Codec comments and compatibility branches carry older shape descriptions while entry-kind space has expanded. The behavior is correct, but commentary no longer helps ownership clarity.

### L2 — Repeated profile-name string mapping/warning composition in pack discovery branches

Warning formatting is repeated across special/symlink omission branches; low risk but noisy and drift-prone.

### L3 — Test surface strongly locks presentation strings, sometimes masking structure debt

Golden/help tests are valuable, but some coverage asserts output form heavily while shared implementation seams (e.g., strict/recover restore policy duplication) remain structurally untested as single-authority behavior.

## 5. Suspected vibe-code residue

1. **Layered profile filtering in pack** (`collect_files` + `apply_preservation_profile`) feels like iterative patching rather than clean policy ownership.
2. **Strict/recover twin restore stacks** are “works now, unify later” residue.
3. **Recover pre-analysis call with discarded output** is classic transitional scaffolding left live.
4. **Single-file pack command complexity** is an “accretion zone” where optimization packets piled onto an already broad module.
5. **Benchmark matrix hard-coding despite emitted dataset manifest** indicates tooling evolved in phases without final consolidation.

## 6. Recommended cleanup packets

### CRUSHR_CLEANUP_01 — Unify pack preservation-profile authority

- **Scope**: collapse dual ownership between `collect_files` and `apply_preservation_profile`.
- **Why**: eliminate profile multi-source truth in canonical pack pipeline.
- **Order**: 1.

### CRUSHR_CLEANUP_02 — Shared strict/recover metadata restore core

- **Scope**: consolidate profile obligation and restore primitives used by strict/recover.
- **Why**: largest drift-reduction win in extraction correctness transparency.
- **Order**: 2.

### CRUSHR_CLEANUP_03 — Recover metadata-degraded routing dedup

- **Scope**: single helper path for metadata-degraded placement + manifest entry assembly.
- **Why**: reduce per-entry-kind drift risk and change cost.
- **Order**: 3.

### CRUSHR_CLEANUP_04 — Decompose `commands/pack.rs`

- **Scope**: internal module split by responsibility while preserving current external behavior.
- **Why**: cut review blast radius and hidden coupling.
- **Order**: 4.

### CRUSHR_CLEANUP_05 — Info wording + structure truth pass

- **Scope**: audit hard-link/structure messaging and profile/metadata wording consistency.
- **Why**: operator trust depends on precise contract language.
- **Order**: 5.

### CRUSHR_CLEANUP_06 — Benchmark matrix/config centralization

- **Scope**: make runner consume manifest/config rather than hard-coded arrays.
- **Why**: prevent tooling/docs/schema drift as benchmark suite grows.
- **Order**: 6.

## Review questions (explicit answers)

1. **Where is logic duplicated?** Pack profile filtering, strict/recover metadata-restore and profile checks, recover metadata-degraded branch handling.
2. **Where are layered fixes instead of clean ownership?** Pack profile gates split across discovery and post-filter.
3. **Where do profile/recovery semantics exist in more than one place?** Pack discovery/filter, strict extract, recover extract, info rendering.
4. **Where are comments stale/vague?** Compatibility/commentary paths in index/decode and some absolute wording in info output.
5. **Where are abstractions doing too much/too little?** `commands/pack.rs` does too much; recover pre-analysis call does too little (no authority).
6. **Where do pack/extract/info share concepts inconsistently?** Preservation-profile obligations and metadata requirement semantics are implemented independently.
7. **Where are tests compensating for awkward structure?** Strong golden-output locking without equivalent shared-authority structural checks in duplicated strict/recover internals.
8. **Where are dead/stale helpers or leftovers?** Recover pre-analysis call-and-discard path.
9. **Where does naming drift from responsibility?** `block model` wording in info can imply tighter 1:1 semantics than implementation reality.
10. **Where could future bugs come from multi-source truth?** Profile obligation logic and metadata restoration expectations across pack/info/strict/recover.
