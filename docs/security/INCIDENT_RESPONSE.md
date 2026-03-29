# Incident Response

## Definition
An incident is any condition where:
- archive integrity is compromised
- verification fails unexpectedly
- recovery behavior violates documented guarantees
- system behavior contradicts published invariants

## Response Process

1. Detect
- failure surfaced via verification, inspection, salvage, or user report

2. Contain
- prevent further processing of unverified data
- fail operation immediately where required

3. Analyze
- use structured outputs to determine failure scope
- identify affected regions or behaviors

4. Report
- provide explicit machine-readable output describing failure
- no silent handling

5. Resolve
- fix underlying issue if caused by implementation defect
- add regression tests or documentation updates as needed

## Non-Goals
- no automatic repair of corrupted data
- no heuristic reconstruction

## Principle
All incidents must result in increased determinism, better reporting, or clearer system boundaries.
