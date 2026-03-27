<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_HOSTILE_REVIEW_01

## 1. Executive judgment

Overall enterprise-readiness assessment: **functionally strong but structurally patched-over in critical paths**.

The codebase shows real correctness work and extensive tests, but the pack/extract/info surfaces now carry clear iterative-layer residue:

- preservation-profile behavior is implemented in multiple places with overlapping logic
- strict and recover extraction paths duplicate substantial behavior rather than sharing a single authority layer
- pack has grown into a mixed production+experimental orchestration module with high coupling
- introspection output has at least one now-misleading structural statement

Major risk themes:

1. **Multi-source-of-truth risk** for preservation semantics (pack discovery/filtering + strict/recover gating + info projection).
2. **High drift risk** from strict/recover duplication for metadata restoration and profile logic.
3. **Operational complexity risk** from an overgrown pack module where production concerns and lab metadata experiments co-reside.
4. **Trust/communication risk** where user-facing info wording no longer cleanly matches actual layout semantics.

Verdict: the project looks like a serious system under active hardening, but not yet like a cleanly-owned enterprise surface. It is extendable, but extension risk is rising quickly without structural cleanup packets.

## 2. High-severity findings

### H1 — Preservation profile behavior is implemented twice in pack (and not from one authority)

**Where**

- `collect_files(..., profile)` performs profile-aware omission and warning emission during discovery.
- `apply_preservation_profile(...)` then does another profile pass with retention/field-pruning + warning emission.

**Pattern**

Layered patching / dual behavior ownership.

**Why risky**

- Two independently-evolving places decide what gets omitted and how warnings are surfaced.
- Easy to introduce mismatches where discovery captures/skips one thing but post-filtering assumes another.
- Hard for future maintainers to identify the canonical profile authority in pack.

**Cleanup shape**

- Single-source “profile projection” stage for `InputFile` records.
- Discovery should capture only what is needed for policy evaluation; one post-capture policy transform should own omission + warning decisions.

---

### H2 — Strict vs recover extraction duplicate core metadata/profile logic and payload read paths

**Where**

- `metadata_required_by_profile(...)` exists in both strict and recover modules.
- metadata restoration helpers (`restore_xattrs`, `restore_security_metadata`, `restore_ownership`, `restore_mtime`) are duplicated with near-identical flow.
- raw payload read/decompress helper paths are duplicated (`read_entry_bytes_strict` + `block_raw_payload`).

**Pattern**

Parallel implementations that can silently drift.

**Why risky**

- One bugfix in strict can easily miss recover (or vice versa).
- Profile behavior changes become two-module edits and two-module test surface.
- Hard to prove trust-class differences are intentional rather than accidental drift.

**Cleanup shape**

- Extract shared “entry materialization + metadata restore engine” with policy hooks for strict vs recover outcomes.
- Keep trust routing separate; share restoration/profile semantics.

---

### H3 — Recover flow computes recovery analysis in command layer and then discards it

**Where**

- `commands/extract.rs` calls `run_recovery_analysis(...)`, then immediately binds fields to `_` and does not use results.

**Pattern**

Layer residue / dead intermediate computation.

**Why risky**

- Extra analysis pass adds cost/coupling without behavior.
- Creates false impression that pre-analysis meaningfully gates recover extraction.
- Encourages hidden divergence if recover implementation evolves independently.

**Cleanup shape**

- Either remove the pre-pass from command layer or make it authoritative (drive displayed summary/progress contract from it).

## 3. Medium-severity findings

### M1 — `pack.rs` is a monolithic mixed-responsibility boundary

**Where**

Single module owns:

- production CLI parse/help
- lab experimental CLI surface
- discovery policy
- planning
- compression/emission
- tail finalization
- large volume of experimental metadata record builders

**Pattern**

Overgrown abstraction with mixed product/lab concerns.

**Why risky**

- Raises review blast radius for any change in pack.
- Obscures ownership boundaries between canonical product behavior and lab-only experimentation.
- Increases regression risk when optimization packets touch production hot paths.

**Cleanup shape**

Split by responsibility into internal submodules (`cli`, `discovery`, `planning`, `emitter`, `experimental_metadata`). Keep command entry thin.

---

### M2 — Metadata-degraded routing in recover has repeated per-entry-kind branches with cloned manifest assembly

**Where**

`recover_extract_impl` match arms for regular/directory/symlink/special each duplicate:

- failed metadata branch
- move to `metadata_degraded`
- construct almost identical `RecoveryManifestEntry`

**Pattern**

Copy/paste branch scaffolding instead of a common degraded-routing helper.

**Why risky**

- Increases chance of inconsistent manifest fields across entry kinds.
- Makes future metadata class additions high-touch and error-prone.

**Cleanup shape**

Extract helper: `route_metadata_degraded(entry, destination, degraded_destination, failed_classes, ...) -> RecoveryManifestEntry`.

---

### M3 — `info` structural statement drifts from behavior for hard links

**Where**

`info` prints fixed string: `block model = file-level (1:1 file → unit)`.

**Pattern**

Stale/misleading product statement.

**Why risky**

- Pack planner supports hard-link payload sharing (not strict 1:1 file→unit in those cases).
- Operator-facing truth can become misleading in preservation-heavy workloads.

**Cleanup shape**

Render model statement from actual index/block relationships or use non-absolute phrasing.

---

### M4 — Benchmark harness is deterministic but hard-coded, not manifest-driven

**Where**

`run_benchmarks.py` hardcodes dataset names and comparator variants.

**Pattern**

Tooling contract drift risk as datasets/variants evolve.

**Why risky**

- Adding new dataset classes requires code edits rather than manifest-driven expansion.
- Increases chance docs/schema/harness diverge.

**Cleanup shape**

Read `dataset_manifest.json` and run a declared matrix from one benchmark config object.

## 4. Low-severity findings

### L1 — Stale inline codec comment in `index_codec`

Comment still documents entry-kind domain as only `regular/symlink/directory` while code handles FIFO/char/block kinds.

### L2 — Repeated profile-name warning text formatting in pack discovery

Profile-name mapping and warning strings are repeated in multiple branches (`payload-only/basic/full` mapping + omit warning shape).

### L3 — `info` profile fallback defaults to `full` in multiple degraded decode paths

Fallback appears in several listing/info branches. Behavior is intentional for compatibility, but implementation duplication makes policy intent harder to audit.

## 5. Suspected vibe-code residue

1. **Pack policy layering residue**: discovery-side profile filtering plus post-discovery `apply_preservation_profile` indicates iterative patch stacking instead of one policy owner.
2. **Strict/recover twin implementations**: large mirrored helper sets suggest feature pressure outpaced consolidation.
3. **Recover metadata-degraded branch cloning**: multiple nearly identical arms are classic “ship first, unify later” residue.
4. **Command-layer pre-analysis noop**: `run_recovery_analysis` result discarded is a strong marker of transitional scaffolding left in production flow.
5. **Pack mixed product/lab surface in one module**: practical during acceleration, but now a maintenance hotspot.

## 6. Recommended cleanup packets

### Packet A — CRUSHR_CLEANUP_01: Preservation profile authority unification (pack)

- **Scope**: `collect_files` + `apply_preservation_profile` interaction, warning emission ownership, profile projection flow.
- **Why it matters**: removes multi-source-of-truth risk in canonical pack semantics.
- **Suggested order**: **first**.

### Packet B — CRUSHR_CLEANUP_02: Shared metadata/profile restoration core for strict+recover

- **Scope**: unify `metadata_required_by_profile`, metadata restore helpers, and shared payload-read primitives.
- **Why it matters**: biggest drift-reduction win for extraction correctness transparency.
- **Suggested order**: **second**.

### Packet C — CRUSHR_CLEANUP_03: Recover canonical-routing deduplication

- **Scope**: factor metadata-degraded move + manifest-entry assembly into shared helper paths.
- **Why it matters**: reduces branch-level inconsistency risk and future metadata-class change cost.
- **Suggested order**: **third** (after Packet B).

### Packet D — CRUSHR_CLEANUP_04: Pack module decomposition (product vs experimental boundaries)

- **Scope**: split `commands/pack.rs` into bounded internal modules with clear ownership.
- **Why it matters**: reduces review blast radius and hidden coupling for future optimization packets.
- **Suggested order**: **fourth**.

### Packet E — CRUSHR_CLEANUP_05: Info contract truth pass

- **Scope**: fix block-model wording, centralize profile fallback semantics, audit list/info consistency.
- **Why it matters**: improves operator trust and contract clarity.
- **Suggested order**: **fifth**.

### Packet F — CRUSHR_CLEANUP_06: Benchmark harness matrix/config centralization

- **Scope**: replace hardcoded dataset/variant arrays with manifest/config-driven matrix.
- **Why it matters**: prevents benchmark tooling drift as the suite evolves.
- **Suggested order**: **sixth**.
