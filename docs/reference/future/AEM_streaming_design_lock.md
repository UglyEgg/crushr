<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Future Design Lock: Streaming, Archive Evidence Maps, and Forensic Compare

Status: Locked design brief (planner synthesis)  
Scope: Future-facing architectural direction, not yet implementation-approved  
Audience: Planner / Builder / Reviewer / future Rich  

## Purpose

This document preserves a set of future architectural decisions and guardrails so the ideas do not drift, get forgotten, or get distorted by later context loss.

These ideas are **not** part of the current required 0.4.x implementation unless explicitly promoted into a task packet.

They are preserved here so future work can proceed from a stable conceptual base.

---

## 1. Streaming: what it means for crushr

The word “streaming” is relevant to crushr, but it must be framed carefully.

The correct question is not:

> Can crushr stream?

The correct question is:

> Which truths can crushr expose incrementally without violating its prove-don’t-guess model?

### Streaming classes

#### A. Streaming creation
Incremental archive creation with bounded memory use.

This is aligned with crushr and is already partially reflected in prior pack scalability work.

#### B. Streaming extraction
Sequential archive consumption and output emission as archive truth becomes available.

This is compatible with crushr only where authoritative structure and trust are available.  
crushr must not invent certainty before it can prove names, paths, or relationships.

#### C. Streaming verification
Sequential archive verification and integrity checking.

This fits crushr well and is a desirable future direction, especially for large archives and constrained I/O environments.

#### D. Streaming over unreliable networks
This is the most interesting future line.

Potential future relevance:
- resumable download
- partial verification before full receipt
- extent-aware acquisition
- recoverable downloading
- constrained or unreliable transport environments

This should not be treated as “add networking features.”
It is a future archive-structure and verification design axis.

### Streaming guardrail

Streaming is a valid crushr direction only when it preserves the project’s core trust rule:

- do not guess
- do not present provisional structure as proven truth
- expose incremental truth only when the format supports it honestly

---

## 2. Archive Evidence Map (AEM)

### Name
**Archive Evidence Map**

### Extension
`.aem`

### Purpose

An AEM is an optional external forensic reference artifact.

It is **not** part of the normal archive trust path.

It exists to support:
- forensic comparison
- corruption diagnosis
- external evidence workflows
- “what should be present vs what is still provable”
- later extent-aware acquisition or audit workflows

### What an AEM is NOT

An AEM is not:
- runtime metadata required for normal archive use
- a repair blob
- a silent truth upgrade
- a fallback source for `verify`, `extract`, or `extract --recover`

### Core trust rule

The archive remains the primary truth source for:
- normal verification
- normal extraction
- normal recovery

The AEM is external evidence only.

---

## 3. Command boundary

### Preferred command
`crushr forensic compare`

This command is explicit, expert-oriented, and does not pollute the primary user command surface.

Potential future related commands:
- `crushr forensic map`
- `crushr forensic verify`

But the currently preferred preserved command concept is:

- `crushr forensic compare <archive.crs> <archive.aem>`

### Boundary rule

The `crushr` binary should touch AEMs only through explicit forensic commands.

Normal commands must **not** consult AEMs implicitly:
- no silent fallback in `verify`
- no silent fallback in `extract`
- no silent trust upgrade in recovery mode

---

## 4. AEM contents

The AEM should be machine-oriented, versioned, and structured.

It does **not** need to be human-readable by default.

Preferred content classes include:

- logical paths
- filenames
- BLAKE3 hashes
- extent ranges / hex offsets
- entry kinds
- metadata-class presence
- block/file linkage
- archive fingerprint fields

### Required archive-binding fields

An AEM should bind to the exact archive it describes using fields such as:

- archive BLAKE3
- archive size
- archive format version
- archive index version (if relevant)
- tool version (optional but useful)
- optional archive name hint (non-authoritative convenience only)

### Binding rule

The AEM binds to the archive.

The archive does **not** bind back to the AEM.

This is a deliberate one-way relationship.

---

## 5. Why one-way binding is correct

Do **not** embed AEM references back into the `.crs` archive format.

Why:
- it creates circular artifact coupling
- it complicates reproducible archive generation
- it makes late AEM generation awkward
- it forces “all artifacts produced together” workflows
- it contaminates the archive’s self-contained trust model

Correct model:

- archive is self-contained primary artifact
- AEM is explicit external evidence artifact
- AEM contains archive fingerprint
- forensic commands compare AEM truth to archive reality

---

## 6. AEM integrity and authenticity

### Locked decision
AEMs should support integrity and authenticity from the start.

### Sidecar files

For an archive:
- `archive.crs`

An AEM workflow may produce:
- `archive.aem`
- `archive.aem.b3`
- `archive.aem.sig`

### Checksum
The AEM should have a default BLAKE3 checksum sidecar.

### Signature
The AEM should have a detached signature from the start.

Locked signing approach:
- **minisign**
- detached signature
- Ed25519-based trust model

### Why minisign
- modern
- simple
- credible
- small operational footprint
- appropriate for systems tooling

### What gets signed
Sign the raw `.aem` bytes.

This is sufficient because the AEM already contains the archive fingerprint and expected archive truth.

Do not require separate archive signatures as part of the base AEM design.

---

## 7. Signature model

### Good model
- portable keys
- user-controlled signing identity
- detached signatures
- explicit verification

### Bad model
Do **not** use:
- hardware identifiers
- machine IDs
- CPU IDs
- MAC addresses
- machine-bound “signature” schemes

Those are brittle, non-portable, and not professional trust anchors.

### Future-friendly note
Hardware-backed signing (such as YubiKey-backed Ed25519) may be added later, but hardware is an implementation detail of the key, not the identity model itself.

---

## 8. Future compare model

A future `crushr forensic compare` should be able to verify:

1. the `.aem.sig` is valid
2. the `.aem` matches the provided archive fingerprint
3. the archive’s actual surviving structure/data matches or differs from the AEM expectations

Desired output categories may eventually include:
- signature valid / invalid
- AEM ↔ archive fingerprint match / mismatch
- missing extents
- damaged extents
- expected entries not provable in archive
- archive-provable entries mismatching expected evidence
- archive-internal truth vs external reference truth

---

## 9. DFIR and evidence posture

The AEM concept is compatible with DFIR-style workflows only if these rules remain true:

- AEM is explicit
- AEM is signed
- archive truth and external evidence truth remain distinct
- no silent trust upgrades occur
- mismatch reporting is calm and explicit

This is not paranoia. It is the minimum bar for a serious forensic-side artifact.

---

## 10. Roadmap placement

These ideas are preserved for future 0.4.x+ / 0.5.x+ architectural lines.

They should **not** be treated as immediate implementation scope unless promoted into explicit task packets.

Recommended future sequencing:
1. finish Linux tar-class preservation semantics
2. continue archive introspection and layout visibility
3. benchmark and stabilize core archive semantics
4. then evaluate:
   - streaming verification/extraction improvements
   - AEM generation
   - `crushr forensic compare`
   - extent-aware acquisition / constrained-network workflows

---

## 11. Locked summary

The following decisions are preserved:

- “Streaming” is a real future design axis for crushr, but only where incremental truth can be exposed honestly.
- The correct framing is incremental truth, not vague streaming support.
- The sidecar concept should be called **Archive Evidence Map**.
- The extension is `.aem`.
- The AEM is optional and external.
- The AEM is not part of the normal archive trust path.
- The archive must not bind back to the AEM.
- AEMs should be protected with:
  - `.aem.b3`
  - `.aem.sig`
- Signatures should use **minisign** from the start.
- `crushr forensic compare` is the preferred future command entry point.
- Normal `verify` / `extract` / `extract --recover` must never silently consult the AEM.

---

## 12. Promotion rule

No part of this document becomes implementation scope until it is turned into a formal task packet.

That is intentional.

This preserves the architecture without forcing premature execution.
