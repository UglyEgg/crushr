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
- `crushr info --entry`
- `crushr info --find`
- `crushr info --propagation`
- `crushr extract`
- `crushr extract --recover`
- `crushr about`
- `crushr completion <bash|zsh|fish>`
- `crushr man [--out-dir <path>]`

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

- `P17S02f0` complete: `crushr info --entry` + `crushr info --find` entry-level introspection expansion with deterministic human/JSON behavior.

- `P17S02f1` complete: `crushr info --entry` enriched with deterministic `payload_blake3`, logical range (`start`/`end`), and `identity_source` in both human and JSON output without trust-model drift.
- `P17S03f0` complete: `crushr info --report propagation` now provides explicit dependency, detected-corruption, activated-impact, and per-entry trust-class explanatory sections with deterministic bounded reasons.
- `P17S03f1` complete: propagation surface renamed to `crushr info --propagation`, with default operator-facing human output and explicit `--propagation --json` machine output while preserving deterministic propagation semantics.
- `P17S03f2` complete: propagation human output now renders dependency-dense entry impacts as deterministic multi-line rows with explicit remainder counts (`+ <n> more`) while JSON semantics remain unchanged.
- `P17S02f2` complete: added clap-generated shell completion output for `bash`/`zsh`/`fish` via `crushr completion <shell>` with stdout-only behavior and deterministic CLI token coverage.
- `P17S02f3` complete: added clap-derived man-page generation via `crushr man [--out-dir <path>]` for root + canonical subcommand pages with deterministic output and contract tests.
- `P17S02f4` complete: aligned README command-surface/product-boundary summaries to the canonical CLI, including `completion`, `man`, and `info --propagation` with concise behavior-accurate descriptions.
- `P17S02f5` complete: restored locked presentation contract for `about` and root `help`, removed `salvage` from user-facing command surfaces, and mapped propagation human output away from internal structure/reason identifiers while preserving JSON semantics.
- `P17S03f3` complete: rebuilt `info --entry` correctness to guarantee deterministic dual-order parsing, non-regular archive rejection without hangs (including FIFO), symlink-to-regular-file acceptance, and explicit malformed-usage errors.
- `P17S02f6` complete: removed legacy `crushr salvage` runtime dispatch + implementation modules, deleted salvage-root tests/golden artifacts, and locked CLI/tests/docs so salvage is rejected and absent from help/completion/man surfaces.
