# Archive Format Overview

This document describes the on-disk structure at a conceptual level.

```mermaid
flowchart LR
    A[Compressed Blocks] --> B[Index]
    B --> C[Footer]
```

## Blocks
Blocks are the compressed payload units. File contents map to one or more extents referencing block IDs and byte ranges.

## Index
The index is the authoritative mapping from paths to entries (metadata + extents). It enables listing and random-access extraction.

## Footer
The footer stores offsets (where index begins, where blocks end) and integrity hashes for fast open/verify.
