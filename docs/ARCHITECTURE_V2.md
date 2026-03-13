# Architecture V2 Direction

This document records the locked long-term architectural direction for crushr v2.

It does **not** replace the current baseline implementation, Phase-2 trial plan, or canonical baseline product thesis. It exists to preserve the intended next-step design without creating ambiguity in current work.

## Purpose

The baseline white-paper trials evaluate the current crushr implementation.

Several future capabilities remain intentionally deferred until after that evaluation:

- recoverable archive extraction
- true random-access extraction
- built-in deduplication

This document defines the architectural direction those features should follow when v2 work begins.

## Locked v2 direction

crushr v2 should move toward **content-addressed block identity with deterministic on-disk indexing**.

That means:

- blocks are identified by verified content identity rather than positional location alone
- on-disk layout remains deterministic and inspectable
- file records ultimately reference verified block identities
- physical placement is an implementation detail, not the canonical identity model

## Why this direction is locked

These deferred features all depend on the same structural choice.

### Recoverable archives

Recoverable extraction is cleaner when the system can reason about:

- which referenced objects are present and verified
- which objects are corrupt
- which objects are missing or unreachable

rather than only reasoning about damaged archive offsets.

### True random-access extraction

Random-access extraction becomes more coherent when files reference stable block identities and the archive maintains deterministic lookup/index structures for those identities.

### Deduplication

Deduplication becomes a natural extension of the model when identical content shares the same stable identity.

This is especially important for the planned rollout:

1. whole-file deduplication
2. fixed-size block deduplication
3. content-defined chunking only if later justified

## Explicit non-direction

v2 should **not** be built as:

- a purely positional archive
- plus ad hoc recovery tables
- plus ad hoc dedup references
- plus ad hoc random-access metadata

That approach would create feature layering without architectural coherence.

## Expected v2 design spine

When v2 work begins, the recommended sequence is:

1. deterministic object/block identity
2. object/block table design with deterministic on-disk indexing
3. whole-file deduplication
4. random-access extraction over stable block references
5. recoverable extraction over surviving verified object graph
6. block-level deduplication only if justified by evidence

## Constraints inherited from current thesis

Even in v2, the design should preserve current architectural priorities:

- integrity first
- determinism
- explicit failure semantics
- inspectable structure

## Scope boundary

This document is roadmap guidance only.

It must not be interpreted as:

- changing the current archive format
- changing the current white-paper trial scope
- reopening settled baseline decisions
- authorizing speculative v2 implementation during the current trial phase
