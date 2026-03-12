# crushr Archive Format v1.0

This is the canonical on-disk archive contract for active implementation.

## Scope

Defines:

- byte-level layout
- verification-critical invariants
- append/update behavior for tail frames

Does not define:

- recovery/salvage/reconstruction behavior
- UI/UX workflows

## Compatibility

- v1 uses `BLK3`, optional `DCT1`, `IDX3`, and `FTR4`
- pre-v1/prototype archives are not compatibility targets

## Layout model

An archive is block data followed by one or more tail frames.
The last valid tail frame is authoritative.

`BLK3...BLK3 -> (optional DCT1) -> IDX3 -> FTR4`

## Encoding basics

- little-endian integer encoding
- absolute archive offsets
- BLAKE3 hashes where hash fields are present

## BLK3

Block header carries codec/length/hash metadata and is followed by compressed payload bytes.

Required invariants:

- header length must cover all present fields
- compressed length must match payload byte count
- dictionary reference must be valid when dictionary flag is set
- payload hash verification is required when present and verification mode requires it

## DCT1 (optional)

Dictionary table appears only when referenced by blocks.

Required invariants:

- unique non-zero dictionary IDs
- dictionary hash verification before use

## IDX3

Canonical file/block mapping index.

Required invariants:

- fully parsed without trailing garbage
- references must be in-bounds
- deterministic interpretation for impact and extraction mapping

## FTR4

Tail footer anchors offsets and hashes for the tail frame.

Required invariants:

- offsets and lengths are in-bounds
- index hash must verify
- footer hash must verify

## Verification semantics

- fast verification: structural/footer/index validation
- deep verification: includes block-level payload/hash validation

## Extraction semantics boundary

Extraction is strict-only: tools extract verified-safe files and refuse unsafe files deterministically.
No recovery/salvage/reconstruction semantics are part of this format contract.

## Append semantics

Append/update writes a new authoritative tail frame:

1. validate current authoritative tail frame
2. truncate to `blocks_end_offset` when replacing tail region
3. append new blocks (if any)
4. write optional DCT1, then IDX3, then FTR4
