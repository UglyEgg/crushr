# Recovery and Integrity Model

crushr separates **data integrity** from **metadata validity**.

## Integrity

- Verified via BLAKE3 hashes
- Applies to each extent independently

## Metadata

- Dictionaries are optional
- Validation required before use

## Recovery flow

1. locate extents
2. verify hashes
3. reconstruct data
4. apply naming if possible

## Key invariant

Data recovery must not depend on metadata survival.
