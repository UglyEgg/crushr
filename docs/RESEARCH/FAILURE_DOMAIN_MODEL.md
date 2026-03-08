# Failure-Domain Model

crushr exists to explore a neglected property of consumer archive formats:

> bounded failure domains with deterministic impact enumeration.

Consumer formats often optimize for ratio and speed while leaving corruption impact opaque until extraction. crushr instead models archives as explicit structural units: blocks, extents, dictionaries, and self-contained tail frames.

See `FAILURE_DOMAIN_FORMALIZATION.md` for the normative property statement.
