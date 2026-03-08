# Engineering Charter

## Scope
This repository is engineered for long-term maintainability. Avoid shortcuts that create hidden coupling, non-determinism, or fragile behavior.

## Artifact Naming Convention (Locked)
Release artifacts MUST follow:

```
<product-name>p<phase>s<step>f<fix>.zip
```

Examples:
- `crushrp1s3f0.zip` — Phase 1, Step 3, initial release
- `crushrp1s3f3.zip` — Phase 1, Step 3, third fix/debug artifact

Fix counter resets when a new step starts.

## Versioning Policy (Locked)
- New Phase start: bump **minor** by 0.1.0
- Fix artifact (f increments): bump **patch** by 0.0.1
- Step transition: bump patch only if behavior/CLI/format changed
- Phase completion: bump minor by 0.1.0 (mandatory)

## Documentation Standards (Locked)
Audience: mid-level engineer.

Voice:
- Clear, technical, operational (SRE-style)
- Explain assumptions
- Avoid “reader already knows” dependencies
- Don’t over-document; don’t devolve into bullet-only ambiguity

Each non-trivial document should answer:
- What is this?
- Why does it exist?
- How does it work?
- Where does it fit?

## Diagram Directive (Locked)
Any document that benefits from structure visualization MUST include at least one Mermaid diagram. This includes (non-exhaustive):
- data flow
- format structure
- lifecycle / state machines
- storage layout
- pack/extract pipeline

Mermaid blocks should be minimal and explanatory.
