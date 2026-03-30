<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Future Design Lock: Chain of Custody (CoC)

Status: Locked design brief (planner synthesis)  
Scope: Future-facing architectural direction, not yet implementation-approved  
Audience: Planner / Builder / Reviewer / future Rich  

---

## 1. Purpose

This document preserves the design direction and guardrails for a future **Chain of Custody (CoC)** system within crushr.

The goal is to define a clean, bounded, and non-invasive custody model that:

- records how artifacts are handled over time
- provides verifiable event history
- supports forensic and audit workflows
- does not alter or contaminate archive truth

This is not part of the current implementation surface unless explicitly promoted into a task packet.

---

## 2. What a CoC is

A CoC file is an **external, append-only event log** that records custody and interaction events for:

- `.crs` archives
- optional `.aem` evidence maps
- related verification and comparison workflows

The CoC represents **process truth**, not archive truth.

---

## 3. What a CoC is NOT

A CoC must not:

- influence `verify`, `extract`, or `extract --recover`
- provide fallback truth for missing archive data
- upgrade trust or authenticity silently
- act as a repair mechanism
- replace or duplicate `.aem`
- be implicitly consulted by normal commands

The CoC is **explicit, external, and opt-in**.

---

## 4. Core trust model

The system is intentionally layered:

- `.crs` → primary archive truth  
- `.aem` → external evidence reference  
- `.coc` → custody and event history  

The CoC must never override or alter the truth derived from `.crs` or `.aem`.

All comparisons remain:

- archive truth vs evidence truth  
- custody history is reported, not enforced  

---

## 5. File identity

### Name
Chain of Custody

### Extension
`.coc`

### Example artifact set

archive.crs  
archive.aem  
archive.aem.sig  
archive.coc  
archive.coc.sig  

---

## 6. Binding model

A CoC binds to the artifacts it records via fingerprint fields.

Minimum required bindings:

- archive BLAKE3
- optional AEM BLAKE3
- prior CoC event chain hash (if present)

### One-way rule

- CoC binds to archive/AEM
- archive/AEM do not bind back

This preserves:

- reproducibility
- archive independence
- late CoC generation

---

## 7. Event model

A CoC is an **append-only event log**.

Each entry represents a discrete, signed event.

### Example event types

- created
- imported
- transferred
- received
- verified
- compared
- exported
- signed
- inspected

### Event fields (conceptual)

- timestamp
- actor identity
- event type
- referenced artifact(s)
- artifact fingerprint(s)
- optional notes/context
- previous event hash (chain continuity)

---

## 8. Append-only requirement

The CoC must be:

- append-only
- immutable once written
- chain-linked via hashes

This ensures:

- tamper-evidence
- chronological integrity
- auditability

Rewriting or pruning history is not allowed.

---

## 9. Integrity and authenticity

### Required protections

Each CoC must support:

- BLAKE3 checksum sidecar  
- detached signature  

Example:

archive.coc  
archive.coc.b3  
archive.coc.sig  

### Signing model

- minisign (Ed25519)
- detached signatures
- portable user-controlled keys

### What is signed

Each event or the full CoC (design decision deferred), but:

- signatures must be explicit
- signatures must not depend on machine identity

---

## 10. Signature rules

### Good model

- user-owned keys
- portable identity
- explicit signing events
- verifiable independently

### Disallowed

- hardware identifiers (CPU/MAC)
- implicit system trust
- hidden signing
- machine-bound identity schemes

Hardware-backed keys (e.g., YubiKey) may be used, but remain implementation detail.

---

## 11. Command boundary

CoC interaction must be explicit and isolated.

### Preferred namespace

`crushr forensic`

### Example future commands

crushr forensic custody create  
crushr forensic custody append  
crushr forensic custody verify  
crushr forensic custody show  

### Boundary rule

Normal commands must NOT consult CoC:

- `verify` ignores CoC  
- `extract` ignores CoC  
- `extract --recover` ignores CoC  

CoC is consulted only via explicit forensic commands.

---

## 12. Relationship to AEM

The CoC complements the AEM:

- AEM → describes expected vs provable data truth  
- CoC → records how artifacts were handled over time  

A CoC may include references to:

- AEM fingerprint
- AEM signature validation events
- compare results

But must not merge responsibilities.

---

## 13. Forensic posture

The CoC model supports DFIR-style workflows only if:

- events are explicit and signed
- history is append-only
- no silent trust upgrades occur
- mismatch or inconsistency is reported, not hidden

This ensures:

- auditability
- reproducibility
- credibility in external review

---

## 14. Streaming and large CoC files

Future considerations:

- incremental append operations
- streaming verification of event chains
- partial CoC validation

Must follow the same rule as streaming archives:

> expose incremental truth only when it can be proven

---

## 15. Roadmap placement

The CoC system is explicitly **post-AEM**.

Recommended sequence:

1. complete introspection surfaces  
2. implement AEM generation  
3. implement AEM signing + compare  
4. evaluate CoC requirements  
5. implement CoC  

CoC must not precede stable evidence artifacts.

---

## 16. Locked summary

The following decisions are preserved:

- CoC is an external, append-only custody artifact  
- extension is `.coc`  
- CoC does not influence archive truth  
- CoC binds to archive/AEM, not vice versa  
- CoC must be signed and verifiable  
- minisign is the default signing approach  
- CoC uses an append-only, chain-linked event model  
- CoC is accessed only via explicit forensic commands  
- CoC must not be implicitly consulted by normal operations  

---

## 17. Promotion rule

No part of this design becomes implementation scope until it is converted into a formal task packet.

This ensures:

- no premature coupling  
- no semantic drift  
- no partial implementation leakage into core behavior  

---

## End of document
