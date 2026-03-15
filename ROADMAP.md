# crushr Roadmap

## Baseline Implementation (White-Paper Phase)

Current evaluation focuses on the baseline crushr format implementation.

Goals:

- corruption behavior analysis
- compression comparison vs common formats
- deterministic experimental methodology

Future capabilities are intentionally deferred until after the baseline evaluation.

---

# Experimental Resilience Direction (active)

The active post-baseline research direction is now:

- metadata-independent reconstruction
- payload-adjacent file identity
- file manifest truth
- repeated path checkpoints
- a gradual move toward a **content-addressed recovery graph**

This direction is governed by the locked inversion principle:

> prefer architectures where verified payload-adjacent structures carry reconstructive truth, and treat centralized metadata as an accelerator rather than the sole authority.

Active near-term layering:

1. payload truth
2. extent/block identity truth
3. file manifest truth
4. path truth

Recovery should degrade in reverse order:

1. full named recovery
2. full anonymous recovery
3. partial ordered recovery
4. orphan evidence

---

# Next Active Research Steps

## FORMAT-06 — File manifest checkpoints

Add verified file manifest checkpoints to complete the first practical file-truth layer above payload identity.

Intended benefits:

- better completeness validation
- stronger anonymous verified recovery
- better header/index/tail resilience than payload identity alone

## Later — Graph-aware salvage reasoning

After payload identity + manifest truth stabilize, teach salvage to choose the best surviving verified recovery path across graph layers.

---

# Deferred Research (not current priority)

These remain interesting but deferred until the active recovery-graph direction matures:

- deterministic distributed-hash checkpoint placement
- deterministic low-discrepancy / golden-ratio checkpoint placement
- larger placement-strategy bakeoffs
- broader generalized graph-engine abstractions

The current evidence says placement optimization is secondary to metadata-independent reconstruction.

---

# Longer-Term Vision

The long-term direction for crushr is no longer just “a resilient archive container.”
It is trending toward a **structured archive with a recoverable, content-addressed relationship graph**.

Potential long-term capabilities once the current graph layers are proven:

- stronger recoverable archives
- true random-access extraction
- content-aware deduplication
- graceful degradation from named recovery -> anonymous recovery -> ordered partial recovery -> orphan evidence
