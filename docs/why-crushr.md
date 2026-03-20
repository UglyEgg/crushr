<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Why crushr

crushr exists because conventional archive assumptions become brittle in the damage case. It is a serious attempt to make **what can still be proven** more useful than **the container is no longer coherent, good luck**.

| Dimension | Meaning |
|---|---|
| Primary audience | Storage engineers, systems programmers, archival practitioners, and technical reviewers |
| Problem class | Damaged containers where meaningful payload survives more readily than the metadata that once explained it |
| Position | Not universal packaging; a legitimate archival utility for workflows where recovery honesty matters |

Conventional archive formats optimize for intact containers. They are good at packaging, transport, and clean extraction when metadata remains authoritative. They are much less good at the awkward middle state where metadata is truncated, overwritten, or internally inconsistent but a large portion of payload still survives.

!!! note "crushr rejects both bad extremes"
    It does not promise miracle reconstruction, and it does not treat every damaged container as unrecoverable. Its promise is narrower and more credible: recover exactly what can still be proven and classify the result in an operationally honest way.

## Why the architecture is credible

The strongest architectural choices in crushr were not selected because they sounded elegant. They survived because the deterministic corruption harness repeatedly showed that they remained useful while competing branches did not.

| What survived testing | What did not survive testing |
|---|---|
| Extent identity attached to payload | Metadata-heavy branches |
| Mirrored naming dictionaries | Placement gimmicks |
| Anonymous fallback | Single-dictionary leadership |
| Fail-closed naming rules | FORMAT-15 as a lead replacement |

!!! warning "The central result"
    Payload-adjacent structural truth matters much more than heavy centralized metadata duplication. Naming is still worth preserving, but it is best preserved through a mirrored dictionary subsystem rather than a bloated manifest-heavy control surface.

## Where crushr fits

crushr is a good fit wherever the cost of structural failure is higher than the cost of carrying a little more architectural discipline. That includes long-lived storage, archival research, digital preservation, corruption testing, and workflows that need deterministic post-damage reasoning rather than opaque extraction failure.

It is not intended to replace commodity archives for every routine workflow. The point is not ubiquity. The point is legitimacy: crushr addresses a real problem class with a format whose behavior is explicit, bounded, and evidence-backed.
