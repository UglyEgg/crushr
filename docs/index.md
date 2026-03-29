<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr

crushr is a **salvage-oriented archival format** built for the failure case, not merely the happy path.

It addresses a narrow but serious systems question:

> When an archive is damaged, what can still be proven and recovered without guessing?

| Perspective | Current answer |
|---|---|
| Format stance | Integrity-first archive semantics with deterministic salvage classification |
| Architectural core | Distributed extent identity plus mirrored naming dictionaries |
| Trust boundary | Fail-closed naming with anonymous fallback when proof is unavailable |
| Selection method | Deterministic corruption experiments, not speculative design preference |

![crushr format evolution master diagram](assets/diagrams/crushr_master_diagram.svg)

<span class="figure-caption">crushr’s architecture emerged through elimination. Several plausible branches were tested; only the branches that survived repeated corruption experiments remained in the mainline design.</span>

## What makes crushr materially different

Most archive formats assume the container remains structurally intact. crushr assumes that real failure is uglier than that and treats post-damage reasoning as part of the design, not as a separate apology after extraction fails.

!!! note "Key differences"
    - **Distributed identity** keeps the strongest durable truth with the extents themselves rather than in a single central authority.
    - **Mirrored naming dictionaries** preserve names when one verified mirror survives and refuse names when the naming subsystem cannot be trusted.
    - **Fail-closed semantics** preserve verified payload and downgrade honestly to anonymous recovery when naming proof disappears.
    - **Evidence-backed design** means the current architecture was selected through deterministic corruption experiments rather than style preference or metadata folklore.

!!! tip "The real claim"
    crushr is not trying to be a more decorative ZIP file. Its claim is narrower and more serious: preserve what can still be proven after damage, distinguish structural truth from naming truth, and refuse to guess when proof is gone.

## Where crushr fits

crushr is aimed at workflows where recovery honesty matters more than convenience-first extraction behavior.

| Dimension | Meaning |
|---|---|
| Primary audience | Storage engineers, systems programmers, archival practitioners, and technical reviewers |
| Problem class | Damaged containers where meaningful payload survives more readily than the metadata that once explained it |
| Position | Not universal packaging; a legitimate archival utility for workflows where bounded post-damage reasoning matters |

Conventional archive formats are good at packaging, transport, and clean extraction when metadata remains authoritative. They are much less good at the awkward middle state where metadata is truncated, overwritten, or internally inconsistent but a large portion of payload still survives.

!!! note "crushr rejects both bad extremes"
    It does not promise miracle reconstruction, and it does not treat every damaged container as unrecoverable. Its promise is narrower and more credible: recover exactly what can still be proven and classify the result in an operationally honest way.

## Why the architecture is credible

The strongest architectural choices in crushr were not selected because they sounded elegant. They survived because deterministic corruption testing repeatedly showed that they remained useful while competing branches did not.

| What survived testing | What did not survive testing |
|---|---|
| Extent identity attached to payload | Metadata-heavy branches |
| Mirrored naming dictionaries | Placement gimmicks |
| Anonymous fallback | Single-dictionary leadership |
| Fail-closed naming rules | FORMAT-15 as a lead replacement |

!!! warning "The central result"
    Payload-adjacent structural truth matters much more than heavy centralized metadata duplication. Naming is still worth preserving, but it is best preserved through a mirrored dictionary subsystem rather than a bloated manifest-heavy control surface.

## Read the site in this order

1. [Whitepaper](whitepaper/index.md) — the coherent technical narrative covering the problem, architecture, recovery model, and evaluation story.
2. [Format evolution](format-evolution.md) — the selection-and-elimination story that killed weaker branches and produced the current design.
3. [Technical reference](reference/index.md) — concise implementation-facing reference pages for the current architecture.
4. [Security](security/index.md) — formal guarantees, invariants, trust boundaries, and control documents.
5. [Chronicles](chronicles/index.md) — historical public writing preserved as project milestones, not normative specification.
