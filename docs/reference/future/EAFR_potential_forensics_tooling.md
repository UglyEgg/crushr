<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Future Design Lock: Evidence-Assisted Forensic Reconstruction (EAFR)

Status: Locked design brief (planner synthesis)  
Scope: Future-facing architectural direction, not yet implementation-approved  
Audience: Planner / Builder / Reviewer / future Rich  

---

## 1. Purpose

This document preserves the design direction and guardrails for a future **evidence-assisted forensic reconstruction** capability in the crushr ecosystem.

This capability is intentionally **not** part of the core `.crs` trust model.

Its purpose is to support expert-only workflows where:

- an archive is damaged or incomplete
- an external `.aem` is available
- a specialist wants to attempt bounded reconstruction using explicit outside evidence
- the resulting output is clearly separated from normal archive-provable truth

This feature is preserved here so the idea does not drift into the core product by accident.

---

## 2. What this capability is

This is a future **forensic extension** that may use:

- a damaged `.crs`
- an explicit `.aem`
- verified archive/AEM binding
- explicit operator intent

to reconstruct or restore information that the archive alone can no longer prove.

This is **not normal crushr recovery**.

This is **external-evidence-assisted reconstruction**.

---

## 3. What this capability is NOT

This feature must not be:

- part of normal `verify`
- part of normal `extract`
- part of normal `extract --recover`
- a silent fallback
- a hidden trust upgrade
- a replacement for the `.crs` archive truth model
- a reason to weaken core archive integrity rules

It must not blur the boundary between:

- archive-provable truth
- externally assisted reconstruction

---

## 4. Core trust boundary

The core crushr trust model remains:

- `.crs` is the primary truth source
- `.aem` is external evidence
- `.coc` is custody/provenance history
- normal commands must operate from archive truth alone

This forensic extension exists **outside** that boundary.

### Locked rule

Normal commands must never consult `.aem` for recovery or verification:

- no silent fallback in `verify`
- no silent fallback in `extract`
- no silent fallback in `extract --recover`

If evidence-assisted reconstruction exists, it must be explicit, isolated, and labeled as such.

---

## 5. Correct framing

This feature must never be described as:

- better recovery
- premium recovery
- enhanced extract
- unlock real recovery

Those framings would poison the architecture.

The correct framing is:

- evidence-assisted forensic reconstruction
- external-evidence-derived identity/structure restoration
- explicit non-canonical reconstruction
- expert-only workflow

---

## 6. Command boundary

### Preferred namespace

`crushr forensic`

### Preserved future command concept

- `crushr forensic reconstruct <archive.crs> <archive.aem>`

Possible future related commands:
- `crushr forensic reconstruct --json`
- `crushr forensic reconstruct --out <dir>`
- `crushr forensic compare <archive.crs> <archive.aem>`

### Boundary rule

This capability must live only under explicit forensic commands.

It must never be added as flags to:
- `verify`
- `extract`
- `extract --recover`

---

## 7. Why this must stay separate

The core recovery model answers:

> What can be proven from the archive?

Evidence-assisted reconstruction answers:

> What can be reconstructed by combining archive remnants with external expected truth?

Those are different epistemic claims.

That difference must remain visible in:
- commands
- output
- documentation
- result taxonomy
- user expectations

---

## 8. Result classification

### Locked rule

This capability must not reuse normal trust classes as if nothing changed.

The normal trust classes remain correct only for archive-provable outcomes:

- `canonical`
- `metadata_degraded`
- `recovered_named`
- `recovered_anonymous`
- `unrecoverable`

Evidence-assisted reconstruction must use clearly distinct terminology.

### Preserved future direction

A future reconstruction workflow may define distinct output classes such as:

- evidence_assisted_named
- evidence_assisted_metadata
- evidence_assisted_partial
- evidence_mismatch
- evidence_unusable

Exact naming is deferred, but the rule is locked:

> externally assisted results must not be confused with archive-native truth classes

---

## 9. Required preconditions

Before any reconstruction attempt proceeds, the system must verify:

1. the `.aem.sig` is valid
2. the `.aem` matches the target archive fingerprint
3. the `.aem` is version-compatible enough for the requested operation
4. the operator explicitly invoked forensic reconstruction mode

If any of these fail, reconstruction must not proceed.

---

## 10. Evidence source rules

The `.aem` may be used to restore or reconstruct only what it actually encodes.

Allowed future reconstruction targets may include:

- logical path identity
- filename/path mappings
- metadata-class expectations
- extent/linkage expectations
- expected entry presence
- expected BLAKE3 payload identities

Disallowed:

- invented data beyond archive remnants
- guessed structure not supported by archive remnants and AEM evidence together
- silent assumption filling
- unbounded heuristic reconstruction

---

## 11. Output labeling

Evidence-assisted reconstruction output must be aggressively labeled.

Required properties:

- explicit forensic mode banner
- explicit indication that results are not archive-native truth
- explicit indication that external evidence was used
- explicit machine-readable flags marking assisted reconstruction

This labeling must appear in:
- human-readable CLI output
- JSON output
- manifests written to disk
- result summaries

---

## 12. Manifest requirements

If this feature materializes outputs, it must emit a machine-readable manifest describing:

- archive fingerprint
- AEM fingerprint
- signature verification result
- reconstruction mode used
- per-output reconstruction basis
- mismatch or uncertainty notes
- result classification

This manifest is mandatory.

No evidence-assisted output should exist without an attached explanation trail.

---

## 13. Unlock model

### Preserved commercial direction

This capability may be a commercial-only forensic extension.

### Locked rule

If gated commercially, the gate must not change the meaning of the core OSS product.

The gate may unlock:
- explicit forensic reconstruction commands
- reconstruction manifests
- expert-only workflow surfaces

It must not withhold:
- archive-native verification
- archive-native extraction
- archive-native recovery
- truth surfaces already defined by the core product

### Unlock mechanism direction

A future implementation may use a generated unlock token issued upon payment.

Purpose:
- explicit operator intent
- clear boundary between core and expert-only workflows
- deliberate activation by users who understand the implications

### Design ethics rule

The unlock mechanism must not:
- weaken the OSS core
- create silent behavior changes
- compromise archive trust boundaries
- pretend this is a normal feature hidden behind a paywall

It is a separate forensic workflow, not a crippleware switch.

---

## 14. Open-source reality note

The source is open.

That means:
- a determined user could implement their own evidence-assisted reconstruction
- the commercial value is not secrecy
- the commercial value is:
  - curated workflow
  - safer expert UX
  - bounded semantics
  - manifests and labeling
  - signed evidence handling
  - operational polish

This is acceptable and expected.

The architecture should assume users can inspect and reproduce the approach.

---

## 15. Safety and misuse posture

This feature should exist only if it remains hard to misuse accidentally.

Required properties:

- explicit forensic namespace
- explicit operator opt-in
- explicit warnings and labeling
- no implicit activation by file presence
- no silent AEM discovery
- no reuse of ordinary extraction semantics

The system should bias toward:
- deliberate use
- explicit acknowledgment
- calm, bounded output

Not toward convenience.

---

## 16. Relationship to AEM and CoC

This capability depends on `.aem`.

It may optionally interact with `.coc` by recording:
- reconstruction event
- operator identity
- timestamp
- AEM verification status
- archive fingerprint
- output manifest fingerprint

But `.coc` remains separate:
- `.aem` = evidence map
- `.coc` = custody log
- reconstruction = explicit forensic action

Do not merge these concepts.

---

## 17. Roadmap placement

This capability is strictly **post-AEM**.

Recommended sequence:

1. finish introspection and truth surfaces
2. implement AEM generation
3. implement AEM signing and `forensic compare`
4. stabilize evidence workflows
5. then evaluate evidence-assisted reconstruction

It must not be implemented before `.aem` is real and stable.

---

## 18. Locked summary

The following decisions are preserved:

- evidence-assisted forensic reconstruction is conceptually valid
- it is not part of core crushr recovery
- it must never influence normal `verify`, `extract`, or `extract --recover`
- it must live under explicit forensic commands
- it must rely on validated archive↔AEM binding
- it must be labeled as external-evidence-assisted
- it must not reuse normal trust classes as if nothing changed
- it may be commercial-only
- a generated unlock token is an acceptable future activation model
- the commercial boundary must not distort the OSS core
- manifests are mandatory for any assisted reconstruction output

---

## 19. Promotion rule

No part of this design becomes implementation scope until it is converted into a formal task packet.

That is intentional.

This preserves the idea without contaminating the current product surface or weakening the archive trust model.

---

## End of document
