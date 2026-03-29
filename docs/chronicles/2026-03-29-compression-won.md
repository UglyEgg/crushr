---
title: "2026-03-29 — We Tried to Beat Compression. Compression Won."
description: "Phase 16 findings: after exhausting realistic optimization paths, compression gains were marginal and not worth the tradeoffs."
date: 2026-03-29
status: Historical
series: Chronicles
project: crushr
tags:
    - crushr
    - chronicles
    - compression
    - engineering
source: project site
original_date: 2026-03-29
summary: "We pushed on every reasonable compression lever and found the same answer each time: the gains weren’t worth the cost."
---

# We Tried to Beat Compression. Compression Won.

> _Chronicles entry — originally published externally and preserved here as a historical milestone._

## Context

- **Project:** crushr
- **Series:** Chronicles
- **Status:** Historical snapshot
- **Original venue:** Project Site

---

For the last stretch of development, I went looking for compression gains.

Not in theory, but in a way that would actually matter. Real datasets, controlled runs, and a harness built to remove guesswork.

The question was straightforward: is there still meaningful compression left to capture without compromising the design?

---

## What we tested

### Dictionaries

We added deterministic dictionary training and ran it across the dataset matrix.

The results were underwhelming. In some cases there was no improvement, and in others it was slightly worse than baseline. More importantly, dictionaries introduce a hard dependency between the archive and external training artifacts.

That tradeoff doesn’t fit this system. A smaller archive is not worth a larger failure surface.

---

### Zstd tuning

Next was level and strategy tuning.

We swept levels, compared timing, and looked for a meaningful shift in compression ratio. The gains were consistently small, often within noise, while the time cost was measurable and inconsistent.

Zstd is already well-optimized. There isn’t much left to extract here without paying for it somewhere else.

---

### Ordering and locality

We then forced deterministic ordering into the pipeline: lexical, size-based, extension grouping, and a few hybrids.

Ordering does have an effect, but it’s marginal. The improvements landed in the range of a few hundredths of a percent. Measurable, but not something that would matter in practice or justify additional complexity in the pack path.

---

### Content-based clustering

This was the last serious attempt.

We introduced a lightweight, deterministic classifier and grouped files by inferred content class before compression. If there was hidden locality to exploit, this is where it should have shown up.

It didn’t.

Small datasets sometimes regressed. Medium datasets showed a slight improvement, on the order of a few hundredths of a percent. Larger datasets effectively flattened out again.

At that point, the pattern was consistent enough to stop looking for exceptions.

---

## What we learned

The experiments all pointed in the same direction.

- Zstd is already doing most of the work
- Locality helps, but only slightly for mixed datasets
- Additional logic adds complexity faster than it adds value

That complexity isn’t neutral. It makes behavior harder to reason about, increases coupling, and erodes the guarantees the system is built around.

---

## The decision

We’re not going to push compression further.

Not because it can’t be improved at all, but because the improvements we can measure aren’t meaningful at the product level.

There’s no justification for adding more surface area, more heuristics, or more failure modes to chase sub-percent gains.

---

## What crushr actually is

crushr is not trying to be the smallest archive on disk.

It is trying to be predictable and trustworthy, especially when things go wrong. That means deterministic behavior, explicit structure, and clear reporting of what is intact versus degraded.

Compression supports that goal, but it isn’t the goal itself.

---

## What comes next

With compression effectively closed as an optimization surface, the focus shifts to areas that actually move the needle:

- deeper introspection
- clearer recovery behavior
- explicit dependency and blast-radius visibility
- better operator-facing truth

That’s where the system becomes more useful, not just more efficient.

---

## Closing

This phase wasn’t about finding a clever trick.

It was about removing uncertainty.

Now that the answer is clear, we can stop circling compression and put the effort where it belongs.

