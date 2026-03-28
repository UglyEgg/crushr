<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Development Status

## Current state (authoritative)

- **Current Phase:** Phase 15 — Dictionary Hardening and Namespace Factoring.
- **Current Step:** **CRUSHR_CLEANUP_11 complete** (continuity/control-doc truth reconciliation).
- **Phase 15 cleanup/hardening status:** **complete** through hostile-review follow-on packets (`CRUSHR_CLEANUP_02` through `CRUSHR_CLEANUP_10`) plus this continuity sweep.
- **Runtime/code status in this packet:** **no runtime files changed** (documentation-only reconciliation).

## What is now true in code (post-cleanup truth)

Hostile-review findings were addressed through landed cleanup packets:

- `CRUSHR_CLEANUP_02`: single canonical pack preservation-profile planning authority (`plan_pack_profile`), with discovery policy-free and warning emission centralized.
- `CRUSHR_CLEANUP_03`: single recover metadata-degraded routing + manifest assembly authority (`route_metadata_degraded_entry` + shared manifest helper).
- `CRUSHR_CLEANUP_04`: strict/recover shared restoration core (`restoration_core`) with explicit policy input to preserve trust boundary semantics.
- `CRUSHR_CLEANUP_05`: explicit pack ownership layers (`discovery`, `planning`, `emission`) in-module.
- `CRUSHR_CLEANUP_06`: canonical info/list truth classification authority (`build_info_truth_view`, `build_listing_truth_view`).
- `CRUSHR_CLEANUP_07`: removed dead recover pre-analysis path; recover execution has one authoritative orchestration path.
- `CRUSHR_CLEANUP_08`: restored profile-derived selective metadata capture requirements without reintroducing split profile-policy ownership.
- `CRUSHR_CLEANUP_09`: physical pack decomposition into bounded files (`pack.rs`, `pack/discovery.rs`, `pack/planning.rs`, `pack/emission.rs`).
- `CRUSHR_CLEANUP_10`: shared strict/recover extraction payload/materialization mechanics (`extraction_payload_core`) with explicit policy boundaries preserved.

## Open debt (intentional / deferred)

1. **Planner sequencing debt:** select and lock the next active workstream packet after cleanup closeout (no active implementation packet currently in progress).
2. **Experimental metadata pruning direction:** FORMAT-10/11/12/13/14A/15 evidence review remains planning input, not product-surface runtime work.
3. **Long-range platform work:** Phase 16+ roadmap items (benchmark evidence refresh, 1.x stabilization, evidence/custody layer) remain future work; not active in this packet.

## Next permitted workstream

- **Permitted next action:** start the next planner-issued feature packet with cleanup assumptions locked.
- Future packets may assume:
  - pack profile authority is centralized and must remain single-owner,
  - strict/recover share mechanics where behavior is truly common,
  - strict-vs-recover trust policy boundaries remain explicit and non-collapsed,
  - info/listing truth wording/classification remains centrally owned.

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
