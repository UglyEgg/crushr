<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Extraction Result Contract v1 (`crushr extract --json`)

## Intent

This document defines the deterministic machine-readable extraction result contract emitted by `crushr extract --json`.

## Scope

This contract applies to the current extraction reporting surface.

No speculative recovery, repair, or reconstruction behavior is part of this contract.

## Result categories

Machine-readable extraction output must distinguish:

- successful canonical extraction
- explicit non-canonical or partially refused extraction outcomes where policy permits them
- structural/open/parse/verification error conditions

## Reporting rule

Any non-canonical or refused outcome must remain explicit in machine-readable output.

Strict extraction and recover-capable extraction must not collapse into the same undifferentiated success surface.

## Determinism

For identical archive bytes, flags, and requested operation:

- result category is deterministic
- ordered file/report lists are deterministic
- refusal or classification reasons are deterministic

## Vocabulary rule

Use current canonical product vocabulary in future revisions of this contract.

Historical field names may remain only where compatibility requires them.
