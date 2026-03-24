---
title: "We Have Recoverability"
description: "A milestone entry describing how a deterministic corruption harness turned recoverability from theory into measurable evidence."
date: 2026-03-16
status: Historical
series: Chronicles
project: crushr
tags:
    - crushr
    - chronicles
source: LinkedIn
original_date: 2026-03-16
summary: "Explains how the corruption test harness accelerated iteration and produced a jump from zero salvageable scenarios to seventy-five percent."
---

# We Have Recoverability

> _Chronicles entry — originally published externally and preserved here as a historical milestone._

## Context

- **Project:** crushr
- **Series:** Chronicles
- **Status:** Historical snapshot
- **Original venue:** LinkedIn

---

# From 0% to 75% Recoverability in 24 Hours: What a Corruption Test Harness Changes

### March 16, 2026

A few days ago I shared the first corruption-trial results from an archive format I’ve been building called **crushr**.

Those trials produced an uncomfortable but useful observation.

Most corrupted archives still contained valid data blocks. What
disappeared was the metadata that explains which file those blocks
belong to.

When that mapping vanished, the system could prove the data existed but could not safely reconstruct the files.

So the next step migrated from theory craft to experimentation.

---

### Removing Guesswork From Format Design

Early in the project I built a **corruption research harness**.

The harness does three things:

1. Generates deterministic corruption scenarios across an archive
2. Runs extraction and salvage logic
3. Records what can still be _proven_ about the data afterward

Each run produces structured artifacts describing the outcome.
Those artifacts are normalized so different format variants can be
compared directly.

That means design changes can be evaluated using the same corruption scenarios every time.

Instead of arguing about whether a format change _should_ improve resilience, we can measure whether it actually does.

---

### What Happened Next

Once the harness was in place, the development loop changed completely.

Instead of:

```
design → implement → wait weeks to evaluate
```

the loop became:

```
design change → implement format variant → corrupt the archive → run the salvage planner → compare results
```

That loop now runs in **hours**.

In the most recent comparison run, the new experimental layout produced a large shift in outcomes.

Across **24 deterministic corruption scenarios**, the system moved from:

> **0% salvageable scenarios → 75% salvageable scenarios**

Along with:

- **+92 additional verified blocks**
- **+18 salvageable files**
- **16 corruption cases that previously produced only orphan data now reconstruct partial files**

These numbers come directly from the experimental harness output.

They are not synthetic benchmarks or marketing metrics. They are the raw results of controlled corruption trials.

---

### Why This Matters

Archive formats often assume that if data blocks survive, recovery will be possible.

In practice, metadata structures tend to be the real single point of failure.

Indexes, path tables, and extent maps are small but critical. When
those structures disappear, the surviving data can become difficult to
associate with files safely.

The design change being tested allows surviving blocks to carry **verifiable identity information** so that file structure can sometimes be reconstructed even when the primary metadata layer is damaged.

This is still experimental work, but the harness makes it possible to evaluate the idea quickly.

---

### The Real Story

The most interesting part of this project isn’t the compression format.

It’s the development process around it.

This repository is being built with **AI acting inside a structured engineering workflow**:

```
Planner → Builder → Hostile Review → Controller → Project State
```

The AI implements defined work packets, the results are reviewed,
and everything runs through the corruption harness before conclusions
are drawn.

The harness removes the biggest problem in exploratory systems work: **guessing**.

Without it, experiments like this could take weeks. With it, the project can move from:

```
idea → implementation → corruption test → measurable result
```

**All within a single day.**

---

### What Comes Next

The next phase is expanding the corruption matrix and continuing to evolve the metadata survivability mechanisms.

The goal is not to claim that crushr is “better” than existing formats.

The goal is to explore how archive structures behave under
corruption, and to do it in a way that produces measurable evidence
rather than speculation.

The real eye-opener has been discovering how much faster disciplined engineering experimentation becomes when you combine:

- deterministic test harnesses
- structured development workflows
- and AI as a constrained engineering collaborator

That combination turns what would normally be a slow design process into something closer to a **systems resilience laboratory**.

And we’re only getting started.
