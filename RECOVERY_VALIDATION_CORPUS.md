<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Recovery validation corpus (CRUSHR_RECOVERY_MODEL_04)

This note documents the deterministic corruption corpus and the end-to-end assertions in
`crates/crushr/tests/recovery_validation_corpus.rs`.

## Included deterministic source corpus

The fixture builder creates a compact but mixed tree with:

- canonical structured text (`txt`, `json`, `xml`, `html`)
- strong-signature binary/document types (`png`, `pdf`, `sqlite`, `zip`)
- office/document-container signatures where subtype matters (`docx`, `xlsx`, `odt`)
- nested directory hierarchy with repeated file names in different paths
- an empty directory
- multi-block payload fixtures for anonymous recovery confidence-tier checks (`high`, `medium`, `low`)

## Corruption scenarios and intent

1. `clean` baseline
   - proves strict extraction succeeds and recover mode produces canonical-only output with an empty recovery manifest.
2. `tail truncation`
   - deterministic tail truncation (`truncate_tail(..., 128)`) proves strict and recover both fail closed when archive structure is not openable.
3. `interior metadata/index damage`
   - deterministic index-byte mutation (`corrupt_index_bytes`) proves strict and recover both fail closed on index-hash mismatch.
4. `payload hash mismatch with payload intact`
   - deterministic block-header hash-bit flip (`flip_block_payload_hash_bit`) forces strict refusal while recover mode emits `recovered_named` with untrusted identity.
5. `mixed recovery outcomes in one archive`
   - deterministic block payload clobbering over a split-block archive proves one run can contain:
     - `canonical`
     - `recovered_named`
     - `recovered_anonymous` (high/medium/low confidence naming tiers)
     - `unrecoverable`

## What is asserted

The test validates:

- strict vs recover divergence by scenario
- output tree placement by trust class (`canonical/`, `recovered_named/`, `_crushr_recovery/anonymous/`)
- anonymous naming policy lock:
  - `file_<id>.<ext>`
  - `file_<id>.probable-<type>.bin`
  - `file_<id>.bin`
- manifest truthfulness against on-disk outputs:
  - `recovery_kind`
  - `classification.kind/confidence/basis`
  - `original_identity`
  - `assigned_name`, `size`, `hash` semantics for recoverable vs unrecoverable entries
- no false canonicality and no silent omission for expected recoverable outputs
