<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Dictionary System

crushr uses **mirrored dictionaries** to preserve naming without creating a single point of failure.

## Structure

| Field | Description |
|------|-------------|
| dict_id | Unique dictionary identifier |
| entries | Mapping of extent → filename/path |
| checksum | BLAKE3 over dictionary content |

## Mirroring

- Dictionaries are duplicated across archive segments
- No primary dictionary exists
- Any valid dictionary can restore naming

## Validation

```
if blake3(dict_bytes) != checksum:
    reject dictionary
```

## Failure behavior

| Condition | Result |
|----------|--------|
| one valid dictionary | names preserved |
| multiple valid | first consistent wins |
| none valid | anonymous recovery |

## Design decision

FORMAT-15 attempted central leadership. It failed under corruption.

Mirroring is strictly more resilient.

## Constraints

- Dictionaries must be small relative to payload
- No cross-dependency between dictionaries
