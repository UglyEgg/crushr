<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Archive format boundary

## Intent

This page defines the high-level on-disk structure of a crushr archive.

It describes the archive boundary in structural terms only. It does not define recovery policy, extraction workflow, or user-facing command semantics beyond what the format must support.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Layout

```text
[header]
[extent blocks...]
[dictionary blocks...]
[tail frame]
```

### Header

| Field | Description |
|------|-------------|
| magic | format identifier |
| version | format version |
| flags | global flags |

### Extent block

```text
[extent_identity]
[compressed_payload]
```

Extent blocks bind payload data to deterministic extent identity. Their purpose is to preserve payload truth independently from higher-level metadata state.

### Dictionary block

```text
[dict_id]
[entries]
[checksum]
```

Dictionary blocks preserve identity-bearing metadata such as naming and path relationships. Dictionary validity affects metadata completeness, not payload integrity.

### Tail frame

Contains:

- dictionary index
- extent index
- integrity markers

### Structural model

No single structure is treated as the sole authority for all archive truth.

Payload integrity and metadata completeness are separable. Recovery behavior depends on what surviving archive evidence can still be validated or verified.

## Boundaries / Non-goals

This page does not describe repair behavior, guessed reconstruction, or alternate trust vocabularies.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Constraints

- Extents must be independently readable
- No central manifest dependency may become a single point of total truth
