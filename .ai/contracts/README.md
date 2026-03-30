<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Contracts Index

These documents define non-negotiable implementation and reporting boundaries for crushr.

## Core contracts

- `PROJECT_SCOPE.md` — what crushr is and is not
- `FORMAT_STABILITY.md` — on-disk format compatibility and change rules
- `SNAPSHOT_STABILITY.md` — JSON snapshot compatibility and merge rules
- `ERROR_MODEL.md` — error classes, exit codes, and reporting semantics
- `SECURITY_MODEL.md` — security posture and extraction-safety rules
- `PERFORMANCE_BUDGETS.md` — performance/regression posture
- `QUALITY_GATES.md` — required local, CI, and release gates

## Behavioral/reporting contracts

- `EXTRACTION_RESULT_V1.md` — deterministic machine-readable extraction result contract
- `PROPAGATION_GRAPH_V1.md` — deterministic explanatory propagation graph contract
- `CLI_VISUAL_SEMANTICS.md` — shared CLI semantic token contract
- `CLI_MOTION_POLICY.md` — restrained motion contract for active CLI phases

## Contract vocabulary rule

Contracts must use current canonical product vocabulary.

Use:

- validate = structural correctness
- verify = integrity correctness
- recover = explicit bounded non-canonical extraction path
- trust classes:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

Historical terms may appear only when a contract explicitly documents legacy naming retained for continuity.

