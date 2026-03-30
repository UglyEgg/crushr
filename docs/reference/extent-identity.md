<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Extent identity

## Intent

An extent identity is the atomic unit of payload truth in crushr.

It binds payload bytes to a deterministic, verifiable identity independent of metadata completeness or broader container state.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Structure

| Field | Size | Description |
|------|------|-------------|
| hash | 32 bytes | BLAKE3 hash of raw extent payload |
| length | 8 bytes | exact byte length of extent |
| offset | 8 bytes | logical offset within original file |
| flags | 4 bytes | bitfield describing extent properties |

### Hash derivation

```text
hash = blake3(payload_bytes)
```

No normalization. No framing. Raw payload only.

### Semantics

Extent identity is intentionally independent from naming, path, and surrounding metadata.

This separation is foundational to crushr's recovery model:

- payload truth can survive metadata loss
- metadata loss does not imply payload corruption
- recovery classification can preserve verified data even when identity context is incomplete

### Constraints

- Any mutation of payload invalidates the extent identity
- Offset is descriptive, not sufficient on its own to prove identity
- No dual identity systems are permitted within the archive model

## Boundaries / Non-goals

This page does not define file naming, recovery output classes, or metadata restoration policy.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Example

```text
payload: 0x48656c6c6f
hash:    blake3(payload)
length:  5
offset:  1024
flags:   0x01
```

Extent identity is a root invariant of the format. Higher-level recovery behavior depends on it but does not replace it.
