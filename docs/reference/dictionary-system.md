<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Dictionary system

## Intent

This page defines how crushr stores and validates identity-bearing metadata such as names and paths without making that metadata a single point of total archive truth.

The dictionary system exists to preserve metadata completeness while remaining independent from payload verification.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

crushr uses mirrored dictionaries so that metadata survival does not depend on one central authoritative copy.

### Structure

| Field | Description |
|------|-------------|
| dict_id | unique dictionary identifier |
| entries | mapping of extent to filename or path metadata |
| checksum | BLAKE3 over dictionary content |

### Mirroring model

- dictionaries are duplicated across archive segments
- no single primary dictionary exists
- any valid dictionary may contribute metadata completeness

### Validation

```text
if blake3(dict_bytes) != checksum:
    reject dictionary
```

Dictionary validation affects whether metadata can be trusted. It does not redefine payload integrity.

### Failure behavior

| Condition | Result |
|----------|--------|
| one valid dictionary | metadata completeness may be preserved |
| multiple valid dictionaries | consistency must be established before metadata is trusted |
| no valid dictionary | recovery may degrade to named or anonymous output depending on surviving evidence |

### Architectural consequence

Payload recovery must not depend on dictionary survival. Dictionary loss may degrade identity, but it must not be treated as equivalent to payload corruption.

## Boundaries / Non-goals

This page does not authorize guessed names, repair behavior, or heuristic metadata synthesis.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Constraints

- Dictionaries must remain small relative to payload
- No cross-dependency between mirrored dictionaries may create a hidden central authority
