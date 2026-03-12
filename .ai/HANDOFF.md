# .ai/HANDOFF.md

Start with:
1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `PROJECT_STATE.md`
5. `docs/CONTRACTS/PROJECT_SCOPE.md`
6. `docs/CONTRACTS/ERROR_MODEL.md`

Handoff state:

- CRUSHR-CLEANUP-2.0-A is complete: legacy recovery/salvage code/API/CLI/model surfaces were deleted.
- Step 1.1 hostile-review hardening (CRUSHR-1.1-B) is complete.
- Propagation report now emits bounded structural-current-state output via fallback inspection when open fails.
- Schema/tests were hardened; propagation now includes structural-current-state fallback coverage and extract is strict-only.
- Control docs now align on authority: `AGENTS.md` then `.ai/STATUS.md`.
- Phase 2 is active; next packet remains Step 2.1 controlled corruption matrix manifest/schema.
