# .ai/PHASE_PLAN.md

## Active Phase

- Phase: 0
- Goal: Complete the canonical format and the first read/write/verify path while preserving the integrity-first thesis.

## Steps

- [x] Step 0.1: Migrate halted `crushr` project into prime scaffold
- [x] Step 0.2: Lock v1.0 spec + architecture; introduce workspace crate boundaries
- [x] Step 0.3: Add live+snapshot TUI skeleton + normative snapshot docs/schema
- [x] Step 0.4: Implement Ledger v1 framing (LDG1) + snapshot fingerprint plumbing
- [x] Step 0.5: Define BLK3 parser/writer in `crushr-format`
- [x] Step 0.6: Define DCT1 parser/writer in `crushr-format`
- [x] Step 0.7: Define FTR4 parser/writer in `crushr-format`
- [x] Step 0.8: Tail frame assembly helpers (DCT1 + IDX3 + LDG1 + FTR4)
- [x] Step 0.9: `crushr-core` open path (read-only): locate last valid tail frame
- [x] Step 0.10: Minimal `crushr-info` snapshot emission (read-only)
- [x] Step 0.11: Minimal pack path with BLK3 (no dicts yet)
- [x] Step 0.12: `crushr-fsck` verify (detect+isolate baseline)
- [ ] Step 0.13: Blast-zone dump implementation

## Phase F — Failure-Domain Validation

- [x] Step F.1: Deterministic corruption harness skeleton
- [x] Step F.2: Decompression-free impact enumeration model
- [x] Step F.3: Controlled datasets (single-file deterministic runner path and artifact refresh complete)
- [x] Step F.4: Comparative baseline experiments (bounded first scaffold: crushr/zip runnable, tar+zstd + 7z deferred with explicit reasons)
- [ ] Step F.5: Recorded results and claim validation

## Later Phases

### Phase 1
- DCT1 support in pack
- decode with dicts
- extract + salvage semantics
- tail repair

### Phase 2
1. Compression ledger enrichment
2. Integrity heatmap / blast simulator
3. Opt-in adaptive planning
4. Policy packs
5. Incremental tail-frame checkpointing
6. Near-duplicate layout optimization
