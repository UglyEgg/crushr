# .ai/BACKLOG.md

Deferred items only (non-active).

## Active deferrals from current resilience direction

- [ ] Compare repeated checkpoint placement strategies after FORMAT-06 stabilizes:
  - deterministic fixed early/mid/late placement
  - deterministic distributed-hash placement
  - deterministic low-discrepancy / golden-ratio placement
- [ ] Revisit distributed metadata placement only after payload identity + file manifest graph layers are validated; placement is not the current bottleneck.
- [ ] Evaluate a later graph-aware salvage reasoning layer once payload identity + manifest truth are stable enough to justify it.
- [ ] Decide whether anonymous verified recovery should graduate from experimental salvage to a broader research baseline after FORMAT-06 evidence exists.

## Phase 2+ deferred

- [ ] Expand corruption matrix dimensions only after the current bounded experimental graph/resilience work settles.
- [ ] Add cross-format fixture curation guide after current resilience architecture direction stabilizes.
- [ ] Consider TUI experiment snapshot overlays after salvage/recovery graph reporting stabilizes.

## Later

- [ ] Packaging/distribution polish once research phases complete.
