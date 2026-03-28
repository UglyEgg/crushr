<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/HANDOFF.md

## Fresh-instance takeover (current truth)

1. Read, in order:
   - `AGENTS.md`
   - `AI_BOOTSTRAP.md`
   - `REPO_GUARDRAILS.md`
   - `PROJECT_STATE.md`
   - `.ai/INDEX.md`
   - `.ai/STATUS.md`
   - `.ai/PHASE_PLAN.md`
   - `.ai/DECISION_LOG.md`
2. Treat `.ai/STATUS.md` as authoritative current state.
3. Do not infer active work from old historical sections; use `Current Step` in `STATUS.md`.

## Where the repository stands

- Cleanup/hardening arc after hostile review is complete through `CRUSHR_CLEANUP_10`.
- `CRUSHR_CLEANUP_11` reconciled continuity documents with landed code truth and removed stale/contradictory control-surface guidance.
- There is **no active runtime implementation packet in progress** in this handoff.

## Code assumptions you can rely on

- Pack preservation-profile authority is centralized (plan-owned, not discovery-owned).
- Recover metadata-degraded routing is centralized.
- Strict/recover restoration and payload/materialization mechanics are shared where truly common.
- Strict vs recover trust-policy boundaries remain explicit and must not be collapsed.
- Info/listing truth/report wording is centrally classified before rendering.

## Open debt to keep explicit

- Planner must choose next active packet/workstream.
- Experimental FORMAT metadata-pruning results are still planning input, not product-runtime commitments.
- Long-range phases (16+) remain roadmap, not active implementation.

## Guardrails for the next builder

- Do not modify runtime behavior unless packet scope explicitly requires it.
- Do not reintroduce split authorities that cleanup packets removed.
- Preserve trust-model boundaries (`canonical` strict vs `recover` outputs).
- If any architectural/contract conflict appears, stop and escalate per `AGENTS.md` decision protocol.

## Evidence map

- Detailed packet history: `.ai/CHANGELOG.md`
- Decision rationale and blast radius: `.ai/DECISION_LOG.md`
- Hostile review findings/report: `.ai/COMPLETION_NOTES_CRUSHR_HOSTILE_REVIEW_01.md`
