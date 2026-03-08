# Format Stability Contract

crushr v1.0 format is defined by `SPEC.md` and the `crushr-format` crate.

Canonical components:
- BLK3
- DCT1
- IDX3
- LDG1
- FTR4

## Stability rules

- Any byte-level change requires a new format version or explicit compatibility section.
- Golden vectors are required for new format components.
- Parser behavior for malformed inputs must be deterministic.
- Prototype-era archives are not guaranteed to be readable unless compatibility is explicitly added.
