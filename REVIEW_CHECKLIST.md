# Review Checklist

## Architecture
- Does the patch respect crate/tool boundaries?
- Does it preserve the salvage-oriented, fail-closed thesis?
- Does it introduce hidden coupling?

## Code quality
- obvious duplication?
- layered patch smell?
- unbounded parsing or unsafe assumptions?

## Tests
- are tests deterministic?
- do they prove the claimed behavior?

## Docs and contracts
- were relevant docs updated?
- does any code contradict `SPEC.md`, the whitepaper/foundational docs, or `.ai/contracts/*`?
