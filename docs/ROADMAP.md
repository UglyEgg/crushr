# crushr Roadmap

## Baseline Implementation (White-Paper Phase)

The current white-paper work evaluates the baseline crushr format implementation.

Current goals:

- corruption behavior analysis
- cross-format comparison against common archive families
- deterministic experimental methodology
- auditable evidence generation for white-paper publication

The baseline implementation is intentionally narrower than the long-term product direction.

## Included before white-paper trials

### Deterministic archive generation

Before trials, crushr may include minimal deterministic archive generation so long as it does not alter archive structure or corruption semantics.

Minimal reproducibility rules:

1. deterministic file ordering
2. normalized timestamps
3. normalized permissions
4. deterministic compression parameters
5. deterministic metadata ordering

## Deferred until after the white paper

The following capabilities are intentionally deferred until after the baseline evaluation:

- recoverable archive extraction
- true random-access extraction
- built-in deduplication

They are roadmap items, not features of the baseline implementation under current evaluation.

## Version 2 feature direction

The long-term direction for crushr is a structured archive/container model built around:

- content-addressed block identity
- deterministic on-disk indexing
- integrity-first verification semantics
- explicit and inspectable failure behavior

### Recoverable archives

Goal:

Allow extraction and verification of intact data even when portions of the archive are corrupted.

Expected design qualities:

- corruption isolation
- deterministic recovery reporting
- partial extraction of verified surviving data
- no speculative reconstruction

### True random-access extraction

Goal:

Allow direct extraction of specific files or byte ranges without scanning the full archive.

Expected design qualities:

- stable file-to-block references
- deterministic lookup/index structures
- independently decodable units where appropriate

### Built-in deduplication

Goal:

Reduce storage overhead for repeated content while preserving integrity and future random-access behavior.

Planned rollout:

1. whole-file deduplication
2. fixed-size block deduplication
3. content-defined chunking only if later justified

## Long-term architectural commitment

The decisive v2 choice is that blocks should be identified by **what they are** rather than only **where they sit**.

That means v2 should be built around content identity first, with physical placement treated as deterministic storage layout rather than canonical object identity.

For the formal architectural statement, see `docs/ARCHITECTURE_V2.md`.
