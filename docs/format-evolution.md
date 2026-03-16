# Format Evolution

crushr is moving quickly, so this page is intended as a living map rather than a frozen history.

The short version is:

> crushr started as an integrity-first archive format and is evolving into a **salvage-first archive design** where the strongest truth lives with the payload rather than in a fragile metadata hub.

This page exists to help readers understand how the format changed, what the harness proved, and why the current direction looks the way it does.

## The original assumption

The early design assumption was familiar to anyone who has worked with archives:

```text
archive integrity depends on metadata integrity
metadata corruption causes recovery failure
```

That assumption is reasonable because it describes how many traditional archive containers behave in practice.

The original goal of crushr was therefore framed around **perfect integrity** in the conventional sense.

## The turning point

Once the corruption harness was in place, the project stopped being driven only by theory.

Instead, crushr began evolving through repeated destructive tests:

1. build deterministic archive variants
2. corrupt them in controlled ways
3. measure what still survives
4. keep what helps
5. cut what does not

That process changed the architecture.

## Evolution timeline

```mermaid
flowchart TD
    A["Initial integrity-first archive idea"] --> B["Corruption harness introduced"]
    B --> C["FORMAT-05 self-identifying payload blocks"]
    C --> D["FORMAT-06 file manifests"]
    D --> E["FORMAT-07 graph-aware salvage reasoning"]
    E --> F["FORMAT-08 metadata placement comparison"]
    F --> G["FORMAT-09 metadata survivability audit"]
    G --> H["FORMAT-10 pruning / simplification direction"]
```

## Phase-by-phase evolution

### Early phase — integrity-first container thinking

At the start, crushr looked more like a traditional archival design problem:

- protect metadata
- duplicate metadata
- make the container robust
- recover files from preserved structural information

This phase was important because it established the baseline assumptions that the harness later challenged.

### FORMAT-05 — self-identifying payload blocks

This was the major architectural shift.

Payload blocks gained enough local truth to be independently identified and verified.

That meant recovery no longer had to start from a surviving index. Salvage could begin with the payload itself.

**What changed conceptually**

```text
old model:
metadata -> file structure -> payload

new model:
payload truth -> reconstruction -> metadata as supporting evidence
```

**What the harness showed**

This phase produced the first major recovery improvement. It was the point where crushr stopped looking like “archive with better metadata” and started looking like “archive where the data can still explain itself after damage.”

### FORMAT-06 — file manifests

This phase added file-level truth so the system could better reason about:

- file completeness
- expected extent count
- stronger recovery confidence

**What changed conceptually**

FORMAT-06 did not replace FORMAT-05. It layered file-level structure on top of block-level truth.

**What the harness showed**

FORMAT-06 improved confidence and verification detail more than it improved top-line recovery counts. That was still useful: not every phase needs to create dramatic jumps if it sharpens what the system can prove.

### FORMAT-07 — graph-aware salvage reasoning

This phase changed the reasoning model.

Instead of flat metadata checks, salvage began reasoning over surviving verified relationships:

- block belongs to extent
- extent belongs to file
- file links to name/path if that truth survives

**What changed conceptually**

Recovery classes became more explicit and defensible. The system could explain not just *what* was recovered, but *why that level of recovery was justified*.

### FORMAT-08 — metadata placement comparison

This phase tested three metadata placement strategies:

- `fixed_spread`
- `hash_spread`
- `golden_spread`

The expectation was that better placement might improve metadata survivability.

**What the harness showed**

All three strategies produced effectively identical results.

That suggested placement was not the real issue.

### FORMAT-09 — metadata survivability and necessity audit

This phase asked the blunt question:

> Do the extra metadata layers actually survive enough to matter?

**What the harness showed**

The answer was stark:

- manifest checkpoint survival stayed at or near zero
- path checkpoint survival stayed at or near zero
- verified metadata node count stayed at or near zero

This strongly suggested that the current metadata layers were not driving resilience.

## What the project learned

The experiments so far point to a simple but important conclusion:

```text
traditional metadata is not the main resilience mechanism in crushr
self-identifying payload truth is
```

That does **not** mean metadata is useless.

It means metadata appears to be:

- optional support
- helpful for confidence and naming when it survives
- not the primary source of recoverability

## Current direction

The format is now moving toward a more minimal, evidence-driven architecture.

The working model looks like this:

```text
minimal container framing
+ self-identifying payload blocks
+ salvage reasoning
+ only the metadata that proves its worth
```

This is a major refinement of the original mission.

The original goal was perfect integrity.

The newer, more realistic formulation is:

> Preserve perfect integrity for whatever survives, and recover only what can still be proven.

## Why this matters

This direction is unusual for archive formats.

Traditional archive design usually assumes that metadata must remain authoritative.

crushr is increasingly exploring a different principle:

> data should carry enough truth to survive structural failure.

That is the core reason the project is interesting.

## What comes next

The next major question is no longer “how do we add more metadata?”

It is:

- which metadata layers actually matter?
- which ones can be pruned?
- how small and elegant can the format become without losing recovery capability?

That is why the post-FORMAT-09 direction is likely to emphasize **simplification** rather than growth.

## Reading this page later

This page will need regular updates. The format is evolving quickly, and some current conclusions may be refined or overturned by later data.

That is expected.

The important thing is that crushr is not being shaped by attachment to earlier assumptions. It is being shaped by what survives the harness.

## Read next

For the mechanics behind these results, continue to [Testing Harness](testing-harness.md).