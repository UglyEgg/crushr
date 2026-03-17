# Extent Identity

An extent identity is the **atomic unit of truth** in crushr. It binds payload bytes to a deterministic, verifiable identity independent of container structure.

## Structure

| Field | Size | Description |
|------|------|-------------|
| hash | 32 bytes | BLAKE3 hash of raw extent payload |
| length | 8 bytes | Exact byte length of extent |
| offset | 8 bytes | Logical offset within original file |
| flags | 4 bytes | Bitfield (compression, dictionary use, etc.) |

## Hash derivation

```
hash = blake3(payload_bytes)
```

No normalization. No framing. Raw payload only.

## Guarantees

- Identifies payload independent of container damage
- Enables deduplication and reconstruction
- Survives metadata loss completely

## Constraints

- Any mutation of payload invalidates identity
- Offset is advisory, not authoritative in recovery

## Example

```
payload: 0x48656c6c6f
hash:    blake3(payload)
length:  5
offset:  1024
flags:   0x01
```

## Implementation notes

- Hash MUST be computed before compression if identity is pre-compression
- If post-compression identity is used, it must be consistent across archive
- No dual identity systems allowed

This is the root invariant of crushr. Everything else is secondary.
