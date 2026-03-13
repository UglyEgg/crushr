# crushr Roadmap

## Baseline Implementation (White-Paper Phase)

Current evaluation focuses on the baseline crushr format implementation.

Goals:

- corruption behavior analysis
- compression comparison vs common formats
- deterministic experimental methodology

Future capabilities are intentionally deferred until after the baseline evaluation.

---

# Version 2 Feature Direction

The crushr format is intended to evolve toward a structured archive container with stronger integrity and storage capabilities.

Planned capabilities include:

## Recoverable archives

Allow intact data to be extracted even when portions of the archive are corrupted.

Key design ideas:

- corruption isolation
- deterministic recovery reporting
- partial extraction of verified blocks

## True random-access extraction

Allow direct extraction of specific files or byte ranges without scanning the full archive.

Expected capabilities:

- file index
- block independence
- seekable decode units

## Built-in deduplication

Reduce storage overhead for repeated data.

Planned rollout:

1. whole-file deduplication
2. fixed-size block deduplication
3. content-defined chunking (only if justified)

Deduplication will be designed to preserve archive integrity guarantees.

---

## Long-Term Vision

The long-term direction for crushr combines:

- compression
- corruption isolation
- deterministic structure
- content reuse

This positions the format closer to a resilient structured container rather than a simple linear archive.
