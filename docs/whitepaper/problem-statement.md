# Problem statement

Conventional archive formats optimize for intact containers. They are good at packaging, transport, and extraction when metadata remains authoritative. They are much less good at the awkward middle state where metadata has become damaged but a meaningful subset of payload still survives.

That gap matters. Real archives are truncated, partially overwritten, inconsistently mirrored, or only partially copied. In those cases a tool faces a choice: fail hard because the container is no longer coherent, or attempt hopeful reconstruction that risks blurring the line between evidence and inference.

crushr rejects both extremes. Its design goal is not to recover everything, and it is not to refuse everything. Its goal is to recover exactly what can still be proven and to classify the result in a way that is operationally honest.

## Design requirements

| Requirement | Meaning |
|---|---|
| Structural truth must survive locally | Surviving payload should still be identifiable without depending on a single global map. |
| Naming must be separable from payload identity | File names are valuable, but loss of naming metadata must not erase structurally verified data. |
| Recovery must fail closed | The tool must never invent names, extent ordering, or metadata when proof is unavailable. |
| Evaluation must be repeatable | Architectural decisions must be justified through deterministic corruption experiments. |


<strong>Everything else is downstream of these requirements.</strong> Once those four constraints are accepted, the current architecture follows naturally: payload-adjacent identity, mirrored naming dictionaries, and deterministic recovery classes.

