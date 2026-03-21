<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Roadmap

The public-facing roadmap narrative now lives in:

- `docs/whitepaper/applicability.md`

The active internal execution plan lives in:

- `.ai/PHASE_PLAN.md`

This file is intentionally kept short to avoid duplicate roadmap maintenance.

## Current roadmap doctrine

crushr is presently in the **0.x product-proof and stabilization era**.

That means near-term work prioritizes:

- deterministic CLI/operator identity
- benchmark harness implementation
- compression and performance characterization
- trustworthy verification/report outputs
- stable declared-truth packaging semantics

It does **not** yet mean:

- baked-in evidence/custody semantics in the core format
- courtroom-oriented signing inside the archive container
- full internal truth derivation for every semantic category

Those are later-stage concerns and belong on the longer-range roadmap, not in the current stabilization sprint.

## Current execution order

1. Finish the unified CLI presentation / operator-surface work.
2. Build the benchmark harness and establish quantitative evidence.
3. Use benchmark results to drive compression and performance refinement.
4. Continue hardening the preservation-format story for 0.x.
5. Carry evidence/custody design work as a future extension path rather than current core-format scope.
