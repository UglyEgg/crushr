# Format Evolution and Decision Record

This document records how the crushr format architecture emerged through **deliberate experimentation and elimination**, not incremental feature accumulation.

It is not a changelog. It is a **design evidence record**.

Each phase documents:

- the hypothesis under test  
- why it was plausible  
- how it was evaluated  
- what the results showed  
- the resulting architectural decision  

---

## Current Outcome (Summary)

The current crushr architecture is defined by:

- **Extent identity as primary truth**
- **Mirrored dictionaries for naming**
- **Fail-closed naming semantics**
- **No central authority required for recovery**

The following were rejected:

- manifest-led designs  
- metadata-heavy recovery strategies  
- placement-based optimizations  
- leadership-based dictionary systems (FORMAT-15)

---

## FORMAT-05 — Self-identifying blocks

### Hypothesis  
Embedding identity directly with payload blocks enables recovery without reliance on central metadata.

### Why this seemed plausible  
Traditional archive failures are dominated by metadata corruption. Moving identity closer to data may preserve recoverability.

### Test  
Compared:
- payload-only recovery
- metadata-indexed recovery
- self-identifying block recovery

Under deterministic corruption (truncation, overwrite, fragmentation).

### Result  
- Metadata-indexed recovery failed early under corruption  
- Self-identifying blocks retained recoverable structure  
- Overhead was acceptable relative to recovery gains  

### Decision  
**Promoted** — established the foundation for extent identity.

---

## FORMAT-07 — Metadata-heavy reinforcement

### Hypothesis  
Increasing metadata redundancy improves recovery reliability.

### Why this seemed plausible  
Redundant metadata is a common resilience strategy in archive formats.

### Test  
Introduced expanded metadata layers and duplication strategies.

### Result  
- Increased archive size significantly  
- Did not improve recovery in proportion to cost  
- Metadata remained a correlated failure domain  

### Decision  
**Rejected** — redundancy at the metadata layer does not solve structural fragility.

---

## FORMAT-08 — Placement optimization

### Hypothesis  
Strategic placement of metadata and payload improves survivability under corruption.

### Why this seemed plausible  
Physical layout can influence which regions are more likely to survive partial damage.

### Test  
Varied:
- metadata placement strategies
- payload clustering patterns

### Result  
- No consistent recovery advantage  
- Outcomes were highly dependent on corruption pattern  
- Added complexity without deterministic benefit  

### Decision  
**Rejected** — placement strategy is not a reliable recovery mechanism.

---

## FORMAT-09/10 — Incremental refinement phase

### Hypothesis  
Iterative tuning of prior designs may yield compound improvements.

### Why this seemed plausible  
Earlier phases established partial success; refinement might converge on optimal behavior.

### Test  
Multiple minor variations across:
- metadata structure
- block organization
- recovery heuristics

### Result  
- No breakthrough improvement  
- Confirmed that structural assumptions, not tuning, were the limiting factor  

### Decision  
**Neutral / transitional** — provided evidence that a structural shift was required.

---

## FORMAT-11 — Extent identity consolidation

### Hypothesis  
Treating extents as independently verifiable units will maximize recovery under corruption.

### Why this seemed plausible  
Earlier phases showed payload-adjacent identity outperformed metadata-centered designs.

### Test  
Implemented:
- per-extent hashing (BLAKE3)
- independent validation
- removal of central dependency for payload reconstruction

### Result  
- High recovery rates under all corruption modes  
- Payload integrity preserved even when metadata was lost  
- Clear separation between structural truth and naming  

### Decision  
**Promoted (core architecture)** — extent identity becomes the primary invariant.

---

## FORMAT-12 — Inline naming

### Hypothesis  
Attaching naming data directly to extents enables named recovery without centralized metadata.

### Why this seemed plausible  
If identity works locally, naming might also survive when colocated with payload.

### Test  
Compared:
- extent_identity_only  
- extent_identity_inline_path  
- manifest-based naming  

Measured:
- recovery rate  
- name retention  
- archive size overhead  

### Result  
- Named recovery matched manifest-based approaches  
- Significant duplication cost for repeated paths  
- Demonstrated feasibility of decentralized naming  

### Decision  
**Promoted (transitional)** — validated decentralized naming, but not efficient enough long-term.

---

## FORMAT-12-STRESS — Inline naming under scale

### Hypothesis  
Inline naming remains viable under large-scale workloads.

### Test  
Applied large datasets with high path repetition.

### Result  
- Path duplication caused measurable archive bloat  
- Performance degraded under repeated string storage  

### Decision  
**Demoted** — naming must be decoupled from per-extent duplication.

---

## FORMAT-13 — Dictionary introduction

### Hypothesis  
Centralizing naming into a dictionary reduces duplication while preserving recovery.

### Why this seemed plausible  
Separating naming from extents may retain benefits while reducing overhead.

### Test  
Introduced dictionary structures mapping extents → paths.

### Result  
- Archive size improved significantly  
- Naming restored efficiently  
- Introduced new dependency risk (dictionary survival)

### Decision  
**Promoted with caution** — effective but introduces a recoverability dependency.

---

## FORMAT-14A — Mirrored dictionaries

### Hypothesis  
Replicating dictionaries removes the single-point-of-failure introduced in FORMAT-13.

### Why this seemed plausible  
Redundant but independent copies may allow naming recovery even when partially corrupted.

### Test  
- multiple dictionary copies  
- no primary designation  
- independent validation via checksums  

### Result  
- Naming preserved if any valid dictionary survives  
- No coordination dependency required  
- Balanced size vs recovery tradeoff  

### Decision  
**Promoted (final naming architecture)** — mirrored dictionaries adopted.

---

## FORMAT-15 — Factored dictionary leadership

### Hypothesis  
Introducing a “leader” dictionary reduces redundancy while preserving recovery.

### Why this seemed plausible  
Reducing duplication could improve efficiency without sacrificing correctness.

### Test  
- designated primary dictionary  
- fallback handling for secondary structures  

### Result  
- Recovery degraded when leader was corrupted  
- Naming collapsed despite surviving data  
- No meaningful size advantage over mirrored model  

### Decision  
**Rejected** — leadership reintroduces a central point of failure.

---

## Branch Outcomes

| Design branch | Status | Reason |
|---|---|---|
| Metadata-heavy / manifest-led | Rejected | Fragile under corruption |
| Placement optimization | Rejected | Non-deterministic benefit |
| Extent identity | Promoted | Strong recovery invariant |
| Inline naming | Transitional | Correct but inefficient |
| Central dictionary | Partial success | Efficient but fragile |
| Mirrored dictionaries | Promoted | Best resilience/size balance |
| Dictionary leadership (FORMAT-15) | Rejected | Reintroduced failure point |

---

## Remaining Open Questions

The current architecture is stable, but not final. Active areas:

- Compression strategy vs identity placement
- Dictionary scaling limits under extreme datasets
- Optimal tail-frame indexing for large archives
- Benchmark-driven validation vs ZIP / 7z under corruption

---

## Key Takeaway

crushr’s architecture is not the result of incremental feature design.

It is the result of repeatedly asking:

> “What survives when the archive is broken?”

and removing every design that failed to answer that question correctly.
