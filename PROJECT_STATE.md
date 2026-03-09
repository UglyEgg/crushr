# PROJECT_STATE

## Current state

crushr is in foundation and validation work.

Implemented:
- workspace split (`crushr-format`, `crushr-core`, `crushr`, `crushr-cli-common`, `crushr-tui`)
- LDG1 ledger framing
- BLK3 / DCT1 / FTR4 format primitives
- snapshot envelope types
- initial impact enumeration model
- strict minimal-v1 extraction path (`crushr-extract`) for regular files with corruption refusal by verified block health
- contracts / research scaffolding

Not yet complete:
- tail frame assembly helpers
- real archive open path
- salvage extraction/repair semantics and non-regular metadata fidelity (symlinks/xattrs/dicts/append behavior)
- end-to-end corruption experiments

## Thesis

crushr exists to demonstrate bounded failure domains and deterministic corruption impact enumeration in an archival compression container.


## Repository hygiene

Canonical source-of-truth files live at the repo root, `.ai/`, `docs/`, `schemas/`, and `TASK_PACKETS/`.
Legacy or historical material lives under `docs/legacy/` and `.ai/imported_crushr/` and is not authoritative.
The canonical workspace-level CI definition is `.github/workflows/ci.yml`.
