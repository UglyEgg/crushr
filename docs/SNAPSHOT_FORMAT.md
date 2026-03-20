<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Snapshot Format

`crushr-salvage` emits deterministic JSON snapshots (plans) for damaged archives.

## Schema contract

Current emitted contract is `crushr-salvage-plan.v3` (`schemas/crushr-salvage-plan.v3.schema.json`).

## Pipeline

`scan -> extent verification -> dictionary resolution -> recovery classification`

## Core guarantees

- Exactly one classification per planned file.
- Dictionary conflicts are fail-closed for naming.
- Naming failure does not block anonymous recovery when payload identity verifies.
- Salvage never fabricates unverified filenames.

## file_plans enums

### mapping_provenance

- `PRIMARY_INDEX_PATH`
- `REDUNDANT_VERIFIED_MAP_PATH`
- `CHECKPOINT_MAP_PATH`
- `SELF_DESCRIBING_EXTENT_PATH`
- `FILE_MANIFEST_PATH`
- `FILE_IDENTITY_EXTENT_PATH`
- `FILE_IDENTITY_EXTENT_PATH_ANONYMOUS`
- `PAYLOAD_BLOCK_IDENTITY_PATH`
- `PAYLOAD_BLOCK_IDENTITY_PATH_ANONYMOUS`

### recovery_classification

- `FULL_VERIFIED`
- `FULL_ANONYMOUS`
- `PARTIAL_ORDERED`
- `PARTIAL_UNORDERED`
- `ORPHAN_BLOCKS`

### reason-code vocabulary

`content_verification_reasons` and `failure_reasons` share a closed reason-code set in v3 schema.
