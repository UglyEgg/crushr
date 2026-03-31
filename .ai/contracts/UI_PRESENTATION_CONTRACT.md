<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# UI Presentation Contract

Status: Active contract  
Scope: User-facing CLI presentation only  
Audience: Builder / Reviewer / Controller / future Rich  

---

## 1. Purpose

This contract locks the user-facing CLI presentation rules for crushr.

It exists to prevent regressions where:

- stale terminology reappears
- removed sections reappear
- internal implementation details leak into operator-facing output
- screen layout drifts from approved structure
- UI changes are made “close enough” instead of exactly

This contract applies to human-readable CLI presentation only.

It does not redefine JSON output contracts.

---

## 2. General presentation rules

Human-readable output must be:

- calm
- explicit
- deterministic
- operator-facing
- free of stale or internal-only terminology

Human-readable output must not:

- leak internal identifiers
- expose implementation-only tokens as user-facing labels
- reintroduce removed concepts
- silently change agreed screen layout
- drift from locked terminology

JSON output may expose machine-oriented identifiers where the machine-readable contract requires them.

Human-readable output may not.

---

## 3. Canonical terminology rules

### Required current product language

Use:

- **validate** = structural correctness
- **verify** = integrity correctness
- **recover** = explicit bounded non-canonical recovery path
- trust classes:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

### Forbidden current product language

Do not use in user-facing output:

- salvage
- fsck
- repair
- fixer
- best-effort reconstruction
- guessed identity
- wrapper binary surfaces

Historical references may exist only in historical documents, never in operator-facing presentation.

---

## 4. About screen contract

### Required structure

The `crushr about` screen must contain only:

- title line
- tagline
- Build section
- Behavior section
- Built with section
- Source line or Source section

### Required tagline

The tagline is locked as:

“When the archive breaks, the truth shouldn’t.”

Rendered in quoted italic Unicode presentation.

### Required exclusions

The about screen must not contain:

- Data Model section
- Support section
- Notices line unless the referenced file actually exists and is maintained
- salvage terminology
- internal architecture explanation
- helpdesk-style instructions

### Behavior section

The about screen behavior section must use:

- pack
- extract
- verify
- recover

Use:

- recover → explicit bounded recovery (non-canonical)

Do not use salvage.

---

## 5. Help screen contract

### Canonical product commands

Human-readable help must present only the actual canonical command surface.

Current canonical user-facing commands are:

- pack
- extract
- verify
- info
- about
- completion
- man

### Bounded non-primary commands

Only `lab` may appear as bounded non-primary command unless explicitly revised by a later decision.

### Forbidden

Help output must not expose:

- salvage
- removed commands
- hidden legacy commands
- experimental concepts as normal user workflow

If a legacy command still exists internally, that does not authorize displaying it in help.

---

## 6. Human-readable propagation contract

### Core rule

`crushr info --propagation` in human mode must use operator-facing language only.

### Forbidden in human output

Do not expose raw internal identifiers such as:

- `structure:*`
- `requires_*`
- raw block/structure implementation tokens
- machine-only reason labels

Examples of forbidden patterns include:

- `structure:ftr4`
- `structure:tail_frame`
- `structure:idx3`
- `requires_footer_reachability`
- `requires_tail_frame`
- `requires_index`

### Required behavior

Human-readable propagation must:

- explain impact
- identify affected entries
- identify whether canonical extraction is blocked
- present trust-class support using canonical terminology
- use mapped operator-safe terms

### JSON boundary

Raw propagation identifiers remain acceptable in JSON if the machine-readable contract requires them.

The human-readable surface must not mirror those raw identifiers.

---

## 7. Human vs JSON boundary

### Human mode

Human-readable mode is for operators.

It must:
- explain
- summarize
- present stable user-facing language
- avoid internal leakage

### JSON mode

JSON mode is for machines.

It may:
- use structured identifiers
- expose internal graph/model tokens
- preserve full machine-readable detail

Builder must never collapse these two presentation layers into one mixed surface.

---

## 8. Determinism rule

Human-readable presentation must remain deterministic.

This includes:

- section order
- row order
- summarization rules
- truncation/remainder rules
- wording of fixed labels

Do not make presentation dependent on:
- terminal width heuristics with unstable results
- random ordering
- opportunistic omission
- hidden environment variation

---

## 9. Summarization rule

If human-readable output summarizes dense data:

- summarization must be deterministic
- omitted remainder must be made explicit
- JSON mode must remain full and unchanged in meaning

Example acceptable pattern:

- show first N items in stable order
- show `+ <n> more`

This is acceptable only in human mode.

---

## 10. Presentation change rule

Builder must not alter locked user-facing presentation surfaces unless:

1. the packet explicitly calls for it
2. the change is documented in this contract or a successor contract
3. tests are updated to lock the new presentation intentionally

“Looked cleaner” is not sufficient justification.

---

## 11. Test enforcement rule

Locked presentation surfaces must be backed by tests where practical.

At minimum, tests should protect:

- about screen structure
- absence of forbidden legacy terms
- help command surface
- propagation human-mode abstraction boundary
- presence/absence of locked sections and labels

If a screen is considered contractually stable, it should have a presentation test or golden test.

---

## 12. Version display rule

Human-readable build/version screens must reflect the current canonical product version.

Builder must not regress the displayed version.

If a packet is accepted and version policy requires a bump, the version shown in user-facing output must be updated as part of the packet integration.

---

## 13. Review rule

Reviewer and Controller should reject or require correction for:

- stale terminology in human output
- reintroduced removed sections
- internal token leakage into operator-facing screens
- user-visible version regression
- undocumented layout drift
- command/help surface drift

---

## 14. Locked summary

The following are locked:

- user-facing output must use current canonical vocabulary
- about/help/propagation human mode must not drift from approved presentation boundaries
- internal identifiers belong in JSON, not in human-readable surfaces
- removed concepts must not reappear in UI
- version display must not regress
- presentation changes require explicit packet scope

---

## 15. Promotion / revision rule

This contract remains active until explicitly revised by a later task packet or design decision.

No silent deviation is allowed.

---

## End of document
