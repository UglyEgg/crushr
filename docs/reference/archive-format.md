<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Archive Format Boundary

Defines the on-disk structure of a crushr archive.

## Layout

```
[header]
[extent blocks...]
[dictionary blocks...]
[tail frame]
```

## Header

| Field | Description |
|------|-------------|
| magic | format identifier |
| version | format version |
| flags | global flags |

## Extent block

```
[extent_identity]
[compressed_payload]
```

## Dictionary block

```
[dict_id]
[entries]
[checksum]
```

## Tail frame

Contains:

- dictionary index
- extent index
- integrity markers

## Design principle

No single structure is required for recovery.

## Constraints

- Extents must be independently readable
- No central manifest dependency
