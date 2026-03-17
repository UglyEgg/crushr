# Snapshot Format

`crushr-salvage` emits deterministic JSON snapshots (plans) for damaged archives.

## Pipeline

`scan -> extent verification -> dictionary resolution -> recovery classification`

## Core guarantees

- Exactly one classification per planned file.
- Dictionary conflicts are fail-closed for naming.
- Naming failure does not block anonymous recovery when payload identity verifies.
- Salvage never fabricates unverified filenames.

## Classification families

Primary classes emitted by salvage planning:

- `FULL_NAMED_VERIFIED`
- `FULL_ANONYMOUS_VERIFIED`
- `PARTIAL_ORDERED_VERIFIED`
- `PARTIAL_UNORDERED_VERIFIED`
- `ORPHAN_EVIDENCE_ONLY`
- `NO_VERIFIED_EVIDENCE`

Historical/legacy labels may still appear in older artifacts and should be normalized during analysis.
