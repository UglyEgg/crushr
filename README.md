# crushr

**crushr** is a salvage-oriented archival format built for the failure case, not merely the happy path.

Its core design question is simple:

> When an archive is damaged, what can still be proven and recovered without guessing?

crushr’s current architecture is built around:

- distributed extent identity
- mirrored naming dictionaries
- deterministic recovery classification
- fail-closed naming semantics
- anonymous fallback when naming proof is unavailable

## Documentation

The public product and whitepaper documentation now lives under `docs/`.

Start here:

- `docs/index.md` — site landing page
- `docs/ARCHITECTURE.md` — canonical runtime and lab boundary
- `docs/SNAPSHOT_FORMAT.md` — salvage snapshot and classification guarantees
- `docs/testing-harness.md` — runtime vs lab test execution
- `docs/why-crushr.md` — positioning and legitimacy
- `docs/whitepaper/index.md` — technical whitepaper
- `docs/foundational_docs/index.md` — lower-level format references

## Internal project control

The repository also contains internal planning and control material under:

- `.ai/` — active project-control documents
- `.ai/contracts/` — policy and interface contracts used during development

These files are not part of the website and should be treated as internal engineering/project-control material.

## Canonical runtime boundary

The canonical tool boundary remains:

- `crushr-pack`
- `crushr-info`
- `crushr-extract --verify`
- `crushr-extract`
- `crushr-salvage` (experimental, separate from canonical extraction)

`crushr-extract` remains strict and deterministic.
`crushr-fsck` is retained only as a temporary deprecated compatibility shim that directs to `crushr-extract --verify`.
