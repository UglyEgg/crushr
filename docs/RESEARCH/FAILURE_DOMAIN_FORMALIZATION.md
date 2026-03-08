# Failure-Domain Formalization

Let:
- `B` be the set of archive blocks
- `F` be the set of files
- `E(f)` be the set of extents belonging to file `f`
- `C` be the set of corrupted blocks

## Failure-Domain Determinism (FDD)

A file `f` is fully extractable if and only if `E(f) ∩ C = ∅`.

The set of affected files can be enumerated without decompression from block health and index/extents alone.

## Complexity target

Given block-health state and extents, impact enumeration must run in `O(|B| + |E|)` time.

## Implementation status

- format primitives for BLK3/DCT1/FTR4/LDG1 exist in `crushr-format`
- decompression-free impact enumeration model exists in `crushr-core::impact`
- real archive parsing and end-to-end corruption experiments remain to be wired
