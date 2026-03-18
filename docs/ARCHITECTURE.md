# Architecture

This document defines the product architecture for the runtime tools.

## Canonical runtime modules

Runtime behavior is organized as:

- `core/archive_format` — BLK3/IDX3/FTR4 layout and strict verification semantics
- `core/dictionary` — mirrored naming dictionary validation and conflict handling
- `core/extent_identity` — payload-adjacent identity and extent verification
- `core/extraction` — strict extract behavior
- `core/salvage` — deterministic salvage planning with fail-closed naming

The runtime tools are:

- `crushr-pack`
- `crushr-info`
- `crushr-extract --verify`
- `crushr-extract`
- `crushr-salvage`

## Lab modules

Research and comparison code is isolated under lab-facing modules and binaries:

- `lab/format_experiments`
- `lab/corruption_harness`
- `lab/comparison_runners`

In this repository these are currently implemented by the `crushr-lab-salvage` binary and its `lab/*` module tree.

## Boundary rules

- Canonical extraction must remain strict and deterministic.
- Runtime logic must fail closed when naming proof is inconsistent.
- Anonymous fallback is required when payload identity verifies but naming proof does not.
- Lab experiments must not silently redefine runtime behavior.
