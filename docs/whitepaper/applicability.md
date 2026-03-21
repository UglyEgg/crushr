<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Applicability and roadmap

crushr is best understood as a serious preservation utility for workflows that
value verifiable post-damage handling of imperfect result sets. It is not
trying to be the default answer for every routine packaging task.

## Fit

crushr aligns well with:

- post-recovery preservation of damaged or partial result sets
- archival and preservation workflows
- corruption research
- long-lived storage experiments
- ransomware / DFIR aftermath where recovered outputs are mixed-quality
- any environment where the difference between clean extraction failure and still-provable preserved output matters operationally

## Current limits

The architecture is now strong enough to justify formal documentation and
productization, but the project is still in **0.x**.

That means:

- the product thesis is being proven and stabilized now
- benchmark evidence and compression/performance characterization are next
- metadata envelope completion is still ahead
- evidence/custody/signing features are future extension work, not present-day core-format claims

The current product should therefore be described honestly as:

> a deterministic preservation format for imperfect, derived data

—not as a finished evidence/custody platform and not as a universal archive replacement.

## Roadmap logic

The sequencing is deliberate.

### 0.x — prove and stabilize the product

1. Finish product-surface hardening and unified CLI/operator identity.
2. Implement the benchmark harness and generate quantitative evidence.
3. Use measured benchmark results to guide compression and performance work.
4. Complete the bounded metadata and verify/report envelope needed for a serious preservation product.

### 1.x — stabilize the preservation platform

1. Lock stable workflow-facing contracts.
2. Mature packaging, reproducibility, and documentation.
3. Make crushr predictable enough for external workflows to rely on.

### 2.x — extend toward evidence-grade workflows

1. Add sidecar evidence manifests, signing, and custody-event tracking.
2. Expand internal deterministic classification only where provable.
3. Keep evidentiary ceremony layered on top of the stabilized platform rather than prematurely baking it into the core format.

<strong>Legitimacy does not require total completeness.</strong> A format becomes legitimate when it has a defensible purpose, a coherent architecture, a documented trust boundary, measurable evaluation evidence, and a roadmap that respects what is solved now versus what belongs later.
