# Project Scope

## What crushr is

crushr is an integrity-first archival compression container.

Core product goals:
- explicitly bounded failure domains
- deterministic corruption impact enumeration without extraction
- strict verification semantics
- transparent, inspectable metadata and forensics output
- a vendorable Rust library with thin tools layered over it

Product thesis:

> A compression format with database-grade integrity and forensic transparency.

And, in plain language:

> It tells you what survived.

## What crushr is not

crushr is not:
- a parity or erasure-coding system
- a speculative data recovery tool
- a replacement for zip/7z in all workflows
- a distributed storage engine
- an excuse for opaque "AI" behavior or hidden heuristics

## Non-negotiables

- Detect and isolate corruption; do not fabricate bytes.
- `fsck` may dump raw compressed blast-zone payload bytes.
- Decompressed blast-zone dumps are emitted only when verification passes.
- Dictionary scope must not create hidden cross-block dependencies.
- Architecture, format contracts, and research claims are controlled by docs first, code second.
