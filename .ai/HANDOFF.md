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

- Phase 16 benchmark packets `CRUSHR_PHASE16_01`, `CRUSHR_PHASE16_02`, `CRUSHR_PHASE16_03`, `CRUSHR_PHASE16_04`, `CRUSHR_PHASE16_05`, and `CRUSHR_PHASE16_06` are complete.
- Benchmark command surface is canonically `python3 scripts/benchmark/harness.py <datasets|run|full>` with dictionary, zstd level/strategy, and ordering/locality experiment flags on `run/full`.
- Dataset generation defaults to `--xattrs off` and emits stable `dataset_identity` in `dataset_manifest.json`.
- Full benchmark matrix execution still depends on host comparator tools (`tar`, `xz`, `zstd`), and non-default zstd strategy experiments now require host `--strategy=<name>` support with early capability failure when unavailable.

## Code assumptions you can rely on

- Pack preservation-profile authority is centralized (plan-owned, not discovery-owned).
- Recover metadata-degraded routing is centralized.
- Strict/recover restoration and payload/materialization mechanics are shared where truly common.
- Strict vs recover trust-policy boundaries remain explicit and must not be collapsed.
- Info/listing truth/report wording is centrally classified before rendering.
- Benchmark assumptions + dictionary + zstd + ordering experiment models are centralized in `scripts/benchmark/contract.py` and embedded into run output (`assumptions` + `dataset_manifest` + `dictionary_artifacts` + per-run `dictionary` + per-run `ordering_strategy` + per-run `zstd_level`/`zstd_strategy`).
- Ordering tar input-list generation is now deterministic and validated before execution (non-empty, well-formed, and filesystem-resolvable), with paths rooted to the benchmark execution context (for example `datasets/<dataset>/...`) for stable tar `-T` behavior.

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


Guardrail: Do not introduce compression features that increase dependency coupling or reduce failure transparency.


Guardrail: Do not propose runtime/archive dictionary support unless results clear the locked dictionary evaluation gate and preserve integrity-first behavior.
