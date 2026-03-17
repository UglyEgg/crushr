# crushr Archive Format

Status: Draft v0.1  
Intent: Foundational specification

## Overview

crushr is a compression-oriented archive format designed with strong salvage and verification properties. It prioritizes deterministic recovery, structural integrity, and graceful degradation when portions of an archive become corrupted.

Unlike manifest-heavy formats, crushr distributes critical identity information through the archive and stores naming metadata in mirrored dictionaries. When naming proof is unavailable, crushr falls back to anonymous extraction instead of guessing.

## Design goals

| Goal | Meaning |
|---|---|
| Recoverability | Data should remain recoverable even if parts of the archive are damaged |
| Deterministic verification | Recovered bytes must be provably linked to verified extents |
| Fail-closed naming | Untrusted or missing naming metadata must never silently produce filenames |
| Structural simplicity | Core on-disk structures should stay inspectable and bounded |

## High-level layout

![Archive Layout](../assets/diagrams/archive_layout.svg)

A crushr archive is conceptually organized into:

- archive header
- mirrored dictionary copy A
- extent table / frame locator structures
- payload extents carrying identity
- tail frame with mirrored dictionary copy B

## Core ideas

### Extent identity

Each extent carries enough structural identity to support grouping, ordering, and verification even when naming metadata is unavailable.

### Mirrored naming dictionaries

A header copy and a tail copy are used for file naming. One surviving valid dictionary is sufficient for named recovery. If both are lost, recovery continues anonymously.

### Deterministic recovery classification

Every salvage run classifies outcomes into a bounded set of result categories rather than relying on vague success/failure labels.

## Current architectural direction

The experimental sequence established the current shape:

- FORMAT-12 proved that inline path identity restored named recovery but duplicated strings heavily.
- FORMAT-13 showed that dictionary indirection reduced archive size materially.
- FORMAT-14A validated header+tail mirrored dictionaries as the correct resilience tradeoff.

## Tooling surface

Core commands are expected to converge on:

- `crushr pack`
- `crushr unpack`
- `crushr info`
- `crushr fsck`
- `crushr salvage`

Experimental comparison tooling should remain under a lab namespace.

FORMAT-15 tested a factored namespace refinement and generation-aware dictionary identity. In the submitted runs, those refinements preserved recovery behavior but did not reduce size enough to replace the current mirrored-dictionary design.
