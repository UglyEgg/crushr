# Review Checklist

## Architecture
- Does the patch respect crate boundaries?
- Does it preserve the FDD thesis?
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
- does any code contradict `SPEC.md` or `docs/CONTRACTS/*`?
