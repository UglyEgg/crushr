---
title: "Corruption Trials"
description: "The first corruption-trial results and the discovery that metadata survivability, not payload survival, was the real failure point."
date: 2026-03-15
status: Historical
series: Chronicles
project: crushr
tags:
    - crushr
    - chronicles
source: LinkedIn
original_date: 2026-03-15
summary: "Summarizes 540 deterministic corruption trials and explains why orphan evidence emerged as the key archive-design problem."
---

# Corruption Trials

> _Chronicles entry — originally published externally and preserved here as a historical milestone._

## Context

- **Project:** crushr
- **Series:** Chronicles
- **Status:** Historical snapshot
- **Original venue:** LinkedIn

---

# What 540 Corruption Trials Taught Me About Building an Archive Format With AI

### March 15, 2026

I ran 540 corruption trials against an archive format I’m building
with AI acting as an engineering collaborator. The result wasn’t what I
expected.

---

A few days ago I wrote about an experiment: building a compression
format called crushr with AI acting as part of a disciplined
engineering workflow.

The goal wasn’t to see how quickly AI could generate code. The
question was whether AI could work inside a normal engineering process:
planning documents, task packets, hostile review, decision logs, and
measurable outcomes.

We just finished the first corruption trial run.

### The experiment

The research harness generates deterministic corruption scenarios
across different regions of an archive and different magnitudes of
damage.

For the initial run we executed 540 corruption trials against crushr archives.

Each run asks a simple question:

After corruption, what can still be proven about the archive contents?

### Results

Out of 540 trials:

- 531 archives still contained verified data blocks
- 186 archives contained salvageable files
- 345 archives degraded to orphan evidence
- 9 archives had no verified evidence remaining

Those numbers look strange at first. If verified blocks survive in most runs, why are salvageable files much rarer?

### What actually failed

The problem wasn’t the data blocks.

It was the metadata that maps blocks back to files.

Many corrupted archives still contained blocks that verified
correctly. What disappeared was the authoritative mapping that explains
which file those blocks belong to.

Without that mapping the system can prove that valid data exists,
but it cannot safely reconstruct the files without guessing. crushr
intentionally refuses to do that.

Those cases become what the experiment calls orphan evidence:
verified data that no longer has a trustworthy path back to a specific
file.

### Why this matters

Archive formats often assume that if the data blocks survive, recovery will be possible.

These trials suggest something slightly different. The real single
point of failure tends to be the metadata layer: indexes, path tables,
extent maps. When those structures disappear, the remaining data becomes
difficult to reconstruct safely.

The corruption trials made that behavior visible.

### What changes next

The next iteration of crushr will focus on improving the survivability of file-mapping metadata.

The design direction is to introduce redundant, verifiable mapping
structures so extents can still be associated with files when the
primary index path is damaged.

The constraint stays the same: Files are only reconstructed when both the mapping and the data verify. No guessing.

### The process lesson

The project itself is being built through a structured AI-assisted workflow:

```
Planner → Builder → Hostile Review → Controller → Project State
```

Each task is implemented by an AI acting as a builder, reviewed adversarially, and only then accepted into the codebase.

What this run demonstrated is that this style of development can
produce useful engineering results even when the outcome isn’t a clear
success story.

Sadly, it didn't produce a marketing claim. Instead, the system
produced a measurable observation about archive design and corruption
behavior which is far more useful.

---

For an experiment about engineering workflows, that’s a good place to be.
