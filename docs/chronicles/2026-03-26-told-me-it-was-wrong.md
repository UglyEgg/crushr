---
title: "The Tool Told Me It Was Wrong"
description: "A benchmark run exposed a contradiction between archive contract and extraction behavior, validating crushr’s design philosophy."
date: 2026-03-26
status: Historical
series: Chronicles
project: crushr
tags:
    - crushr
    - chronicles
    - debugging
    - benchmarking
    - engineering
source: project site
original_date: 2026-03-26
summary: "A benchmark run surfaced a contradiction between declared archive semantics and runtime behavior, demonstrating the value of explicit contracts and observability in crushr."
---

# The Tool Told Me It Was Wrong

> _Chronicles entry — originally published externally and preserved here as a historical milestone._

## Context

- **Project:** crushr
- **Series:** Chronicles
- **Status:** Historical snapshot
- **Original venue:** LinkedIn

---

It wasn’t supposed to be a big moment.

I was just running the first real benchmark pass on crushr.  
Nothing fancy—generate datasets, run the matrix, collect numbers.

And then I saw it.

A stream of warnings:

```
WARNING[ownership-restore]: could not restore '0:0' ...
```

Except… this was a `basic` archive.

And `basic` explicitly does **not** preserve ownership.

So why was extraction even trying?

---

### That feeling

It wasn’t panic. It wasn’t even frustration.

It was that very specific feeling:

> “Something doesn’t line up.”

Not “this is broken.”  
Not “this failed.”

Just:

- the archive says one thing
- the runtime behavior says another

And they shouldn’t disagree.

---

### The interesting part

This is where things usually get messy.

You start digging:

- is it the dataset?
- is it permissions?
- is it Linux being Linux?
- did I misunderstand something?

Instead, this happened:

1. `crushr info` clearly said:

    ```
    ownership: omitted by profile
    ```

2. extraction clearly said:

    ```
    trying to restore ownership
    ```

That’s not ambiguity.

That’s a contradiction.

---

### And the system made it obvious

That’s the part that surprised me.

I didn’t have to:

- instrument half the codebase
- sprinkle debug logs everywhere
- reproduce it ten different ways

The system already had:

- a declared contract (`info`)
- observable behavior (`extract`)
- explicit warnings

So the problem reduced to:

> “Why does restore ignore the profile?”

That’s it.

---

### The fix

The bug wasn’t complicated.

Extraction was:

- attempting metadata restore first
- then deciding what to do about failures

But the preservation profile is the contract.

So the correct behavior is:

> **If the profile omits a metadata class, never attempt to restore it.**

Move the check earlier.  
Make the profile authoritative at restore time.

Bug gone.

---

### Why this mattered more than the bug

The bug itself was minor.

The important part was what it revealed.

crushr has been picking up features that felt… extra:

- preservation profiles
- metadata visibility (`present / omitted by profile`)
- trust-classed extraction
- explicit warnings instead of silent fallback
- deterministic benchmark datasets

Individually, they’re just features.

Together, they did something I didn’t fully appreciate until now:

> they made the system self-diagnosing

---

### Most tools would not have shown this

A typical archive tool would have:

- silently skipped ownership restore
- or partially applied it
- or ignored the mismatch entirely

And I would have never noticed.

The archive would “work.”

Just incorrectly.

---

### This is the moment it stopped being a toy

Up until now, crushr has been:

- a good idea
- a lot of engineering
- something that _felt_ solid

This was different.

This was:

> I trusted the tool, and it helped me fix itself.

No guessing.  
No archaeology.  
No “probably fine.”

Just:

- stated truth
- observed behavior
- contradiction
- fix

---

### Benchmarking didn’t just measure performance

It did something more important.

It forced the system to operate at scale, repeatedly, under real conditions.

And because the system was built to expose its own contract:

> the bug couldn’t hide

---

### The takeaway

The features that felt like over-engineering early on are the reason this was easy.

Not fast.

Easy.

There’s a difference.

---

### Where this leaves things

crushr is still slower than tar+zstd.  
There’s real work to do there.

But this was the moment I stopped thinking of it as:

> a weekend project that got out of hand

and started thinking of it as:

> a tool that can be trusted to tell the truth, even when it’s inconvenient

---

And that’s a much better place to be.
