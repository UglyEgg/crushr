# Salvage Model

The salvage model defines how crushr behaves under corruption.

## States

| State | Meaning |
|------|--------|
| intact | full recovery with names |
| degraded | payload recovered, naming partial |
| anonymous | payload recovered, names lost |
| failed | insufficient data to verify payload |

## Algorithm

1. Scan for valid extent identities
2. Validate hashes
3. Group extents by original file mapping (if dictionary valid)
4. Reconstruct files
5. Assign names if dictionary passes validation

## Key rule

> Naming is optional. Payload integrity is not.

## Failure handling

- Never guess names
- Never reconstruct partial extents
- Never trust invalid metadata

## Output guarantees

- All output files are hash-verified
- No corrupted data is emitted silently

## Example

Corrupt archive:

- 80% extents valid
- dictionary lost

Result:

- 80% data recovered
- files emitted as anonymous blocks

This is correct behavior.


## Schema contract

Machine-readable salvage output is defined by `schemas/crushr-salvage-plan.v3.schema.json`. Classification and provenance values are closed vocabularies and must match schema enums exactly.
