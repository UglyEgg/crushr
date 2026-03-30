<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Welcome

## Intent

This documentation defines the architecture, guarantees, and behavior of crushr.

crushr is a deterministic archive system that preserves and exposes data truth under failure.

crushr prioritizes data integrity, explicit truth, and bounded failure behavior over maximum compression ratio.

The system is designed to make archive contents inspectable, verification explicit, and extraction behavior bounded by policy.

## Guarantees

- Verified data is never silently corrupted or misrepresented
- Unverifiable data is never presented as valid
- Degraded or partial results are explicitly labeled and structured
- Archive processing fails closed when required truth cannot be established
- Filesystem writes are constrained and cannot escape intended boundaries

## Behavior

### Validation vs Verification

crushr separates:

- **Validation** — structural correctness of archive components
- **Verification** — integrity correctness of payload data via cryptographic proof

Structural correctness and payload integrity are related, but they are not interchangeable.

### Classification and Recovery

crushr classifies extraction and recovery outcomes using explicit trust classes:

- `canonical` — payload integrity is verified and required metadata is intact
- `metadata_degraded` — payload integrity is verified, but metadata or structure is incomplete
- `recovered_named` — payload integrity is verified and identity has been reconstructed within defined constraints
- `recovered_anonymous` — payload integrity is verified but no reliable identity remains
- `unrecoverable` — payload integrity cannot be proven to required standards

These classes reflect crushr’s core design boundary: payload truth is distinct from metadata completeness.

### Modes

- **Strict / Default**
  - Requires verified payload integrity and required metadata
  - Refuses output when canonical guarantees cannot be established

- **Recover**
  - Allows extraction of data that can be cryptographically verified
  - Produces classified output when metadata or structure is incomplete
  - Does not reconstruct, infer, or repair missing data

### Introspection

crushr archives are inspectable without extraction.

- `crushr info` provides archive-level introspection: structure, preservation profile, and declared metadata scope
- `crushr info --list` provides entry-level introspection: listing, classification, and attributes without extraction
- `crushr verify` determines whether strict extraction requirements are satisfied or whether recovery mode is required

## Boundaries / Non-goals

crushr is not a convenience-first archive format and is not defined by compression novelty.

Non-goals:

- No best-effort reconstruction
- No hidden failure smoothing
- No compression-first tradeoffs
- No external decode dependencies

## Documentation Structure

- **Security** — guarantees, threat model, verification semantics, and architectural invariants
- **Reference** — on-disk format, metadata model, and structural definitions
- **Whitepaper** — design rationale and system philosophy
- **Chronicles** — historical development and decision evolution

Canonical behavior and guarantees are defined by this document, the README, and the security documentation.

Reference and historical documents may retain legacy terminology where required for schema continuity or historical accuracy, but they do not redefine the current product vocabulary.
