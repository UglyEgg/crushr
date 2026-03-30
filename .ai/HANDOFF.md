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
   - `.ai/contracts/README.md`
   - `.ai/DECISION_LOG.md`
2. Treat `.ai/STATUS.md` as authoritative for current truth.
3. Treat `.ai/DECISION_LOG.md` and `.ai/CHANGELOG.md` as historical evidence, not active state.

## Where the repository stands

- Phase 15 and Phase 16 are complete.
- Current work is documentation/control-surface alignment before Phase 17 feature work.
- Public and builder-facing docs are being normalized to one vocabulary and one command surface.

## Canonical product surface

- `crushr pack`
- `crushr verify`
- `crushr info`
- `crushr info --list`
- `crushr extract`
- `crushr extract --recover`
- `crushr about`

## Non-canonical surface

- `crushr lab` is internal development harness only.
- Wrapper binaries are not canonical product surface.
- `fsck` is not a live product concept.

## Builder guardrails

- Do not reintroduce `salvage` as current product vocabulary.
- Do not describe crushr as a repair or fixer tool.
- Do not collapse validation and verification into one concept.
- Do not document wrapper binaries as public command surface.
- Do not let historical notes override current canonical docs.

## Trust model you can rely on

- payload integrity is independent from metadata completeness
- strict extraction is fail-closed
- recovery is explicit through `extract --recover`
- trust-bearing non-canonical results are classified explicitly as:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

## Evidence map

- current truth: `.ai/STATUS.md`
- sequencing: `.ai/PHASE_PLAN.md`
- non-negotiable boundaries: `.ai/contracts/README.md`
- historical rationale: `.ai/DECISION_LOG.md`
- chronology: `.ai/CHANGELOG.md`

## Latest completed step

- `P17S01f0` complete: `crushr info` truth-surface expansion + wrapper binary removal to single `crushr` binary access.
