<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Whitepaper

This section presents crushr as a technical format rather than an experiment diary. It explains what the format is for, how the archive is structured, what recovery guarantees it does and does not make, and why the current architecture emerged from the evaluation program.

| Dimension | Current answer |
|---|---|
| Audience | Storage engineers, systems programmers, archival practitioners, and technical reviewers |
| Claim | crushr is a compression-oriented archive with deterministic salvage behavior under corruption |
| Core mechanism | Extent identity near the payload plus mirrored naming dictionaries |
| Evaluation basis | Deterministic destructive testing across competing design branches |

## Executive summary

crushr isolates the minimum architecture required to keep verified data useful after damage. It treats structural identity, naming, and later metadata policy as distinct concerns instead of collapsing them into one fragile metadata surface.

!!! note "Three claims that matter"
    - **Payload truth**: local extent identity remains the primary recovery anchor.
    - **Naming truth**: mirrored dictionaries preserve names when one trusted copy survives.
    - **Trust boundary**: naming fails closed and falls back anonymously when proof is unavailable.

## Whitepaper contents

1. [Problem statement](problem-statement.md)
2. [Format architecture](architecture.md)
3. [Recovery and integrity model](recovery-model.md)
4. [Experimental evaluation](evaluation.md)
5. [Applicability and roadmap](applicability.md)
