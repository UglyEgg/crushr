# TASK_PACKET

## Task ID
CRUSHR-0.8-A

## Title
Implement tail frame assembly helpers in crushr-format

## Objective
Compose and parse a self-contained tail frame containing optional DCT1, required IDX3, optional LDG1, and required FTR4.

## Canonical inputs
- `AGENTS.md`
- `.ai/STATUS.md`
- `.ai/DECISION_LOG.md`
- `SPEC.md`
- `docs/CONTRACTS/*`

## Scope
- add a `tailframe` module to `crushr-format`
- implement deterministic assembly helpers
- implement strict parse helpers
- add round-trip tests and malformed-component rejection tests

## Out of scope
- core archive scanning
- pack/extract logic
- dictionary training

## Acceptance criteria
- self-contained tail frame round-trips
- offsets in FTR4 match assembled component layout
- malformed or hash-mismatched components reject deterministically

## Required tests
- round-trip with DCT1 + IDX3 + LDG1 + FTR4
- round-trip without DCT1 and without LDG1
- footer or ledger corruption rejects
