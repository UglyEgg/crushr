<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Applicability and roadmap

crushr is best understood as a serious archival utility for workflows that value verifiable post-damage recovery. It is not trying to be the default answer for every routine packaging task.

## Fit

crushr aligns well with archival and preservation workflows, corruption research, long-lived storage experiments, and any environment where the difference between clean extraction failure and still-provable payload recovery matters operationally.

## Current limits

The architecture is now strong enough to justify formal documentation and productization, but some features remain intentionally out of the core until the architecture is fully hardened. These include the broader POSIX metadata envelope, extended attributes, and later compression-dictionary refinements.

## Roadmap logic

The sequencing is deliberate.

1. Finish architecture hardening and namespace factoring decisions.
2. De-cruft and separate canonical runtime paths from lab infrastructure.
3. Complete the bounded metadata envelope required for a real archival utility.
4. Improve compression intelligence without weakening the recovery model.


<strong>Legitimacy does not require total completeness.</strong> A format becomes legitimate when it has a defensible purpose, a coherent architecture, a documented trust boundary, and an evaluation story that can survive professional scrutiny.

