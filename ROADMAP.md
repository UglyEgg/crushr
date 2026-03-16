# crushr Roadmap

## Baseline Implementation (white-paper phase)

The baseline implementation and baseline comparative evidence are complete and frozen.

That baseline work established:
- deterministic experimental methodology
- comparative corruption evidence
- a credible foundation for later resilience experiments

---

# Experimental resilience direction (active)

The active post-baseline direction is now:

- metadata-independent reconstruction
- payload-adjacent file identity
- file manifest truth
- repeated path checkpoints
- graph-aware salvage reasoning
- bounded metadata placement strategy experiments

This direction is governed by the locked inversion principle:

> prefer architectures where verified payload-adjacent structures carry reconstructive truth, and treat centralized metadata as an accelerator rather than the sole authority

Active near-term layering:

1. payload truth
2. extent/block identity truth
3. file manifest truth
4. path truth

Recovery degrades in reverse order:

1. full named recovery
2. full anonymous recovery
3. partial ordered recovery
4. orphan evidence

---

# Next active research steps

## FORMAT-09 — curated corruption grid / survivability evaluation harness

The next active step is not another format rewrite.

It is a richer evaluation packet that should answer:

- which truth layers survive most often?
- when does recovery downgrade from named to anonymous?
- when does ordered recovery become unordered?
- which duplicated metadata surfaces are weak enough to prune?

This packet should stress:
- truth-layer loss
- metadata disagreement
- block deletion / reorder
- multi-region failures
- downgrade behavior

## After FORMAT-09 — evidence-based pruning and retention

After the richer grid exists, use evidence to decide:

- which metadata surfaces should remain core
- which should become optional experimental ballast
- which should be removed because they add size without enough survivability gain

---

# Product-completeness track (planned)

Once the current resilience-evaluation arc settles, crushr needs to close a very practical product gap on Unix-like systems.

Planned bounded Unix metadata envelope:
- file type
- mode
- uid/gid
- optional uname/gname policy
- mtime policy
- symlink target
- xattrs

This is not the active packet, but it is a real planned product-completeness track.

---

# Compression optimization track (later)

After the resilience architecture, placement work, and pruning decisions settle, revisit compression efficiency.

Planned future area:
- distributed dictionaries

Dictionary work should compare:
- archive-global dictionary
- clustered/per-region dictionaries
- explicit dictionary-object approaches

And must preserve crushr’s integrity-first rules:
- explicit dictionary identity
- verifiable block -> dictionary dependency
- deterministic degradation when a dictionary is missing

---

# Longer-term vision

The long-term direction for crushr is no longer just “a resilient archive container.”

It is trending toward a structured archive with:
- recoverable, content-addressed relationships
- deterministic survivability evaluation
- stronger Unix/archive product completeness
- later compression optimization that does not weaken verification guarantees
