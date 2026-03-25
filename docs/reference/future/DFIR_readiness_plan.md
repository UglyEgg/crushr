# crushr DFIR Readiness Plan

## Status
Draft — Pre-0.4.x (Pre-DFIR)

---

## 1. Purpose

This document defines the path required for crushr to transition from an engineering project into a tool that can be taken seriously in DFIR (Digital Forensics & Incident Response) and, eventually, forensic/legal contexts.

This is not a feature roadmap.
This is a **credibility, validation, and adoption roadmap**.

---

## 2. Current State (v0.3.x)

crushr has achieved:

- Deterministic behavior
- Clear CLI product surface
- Explicit trust classification (canonical / recovered / anonymous)
- Recovery-aware extraction
- Corruption testing harness (research-grade)

crushr has NOT yet achieved:

- Formal validation
- Defined forensic scope/claims
- Provenance-rich reporting
- External adoption or review
- Benchmark positioning

Conclusion:
crushr is **technically interesting and structurally promising**, but not yet DFIR-ready.

---

## 3. Guiding Principles

1. **Truth over completeness**
   Never reconstruct data without verifiable justification.

2. **Determinism over convenience**
   Same input must always produce the same output and classification.

3. **Explicit trust boundaries**
   Users must never infer trust — it must be stated.

4. **Validation over claims**
   Nothing is "forensic" until it is validated.

5. **Reproducibility is mandatory**
   Every result must be independently reproducible.

---

## 4. Target Positioning Levels

### Level 1 — DFIR Utility
- Used by practitioners
- Not relied on as sole evidence processor

### Level 2 — Lab-Adoptable Tool
- Can be validated internally by forensic labs
- Used in structured workflows

### Level 3 — Court-Aware Tool
- Outputs and methodology withstand scrutiny
- Expert testimony defensible

Initial target: **Level 1 → Level 2**

---

## 5. Phase Plan

### Phase 0 — Core Completion (Pre-DFIR)

Prerequisites:
- Benchmark harness
- Compression strategy decisions
- Stable format behavior
- CLI fully consistent

Exit Criteria:
- No major architectural churn
- Repeatable pack/extract/verify behavior

---

### Phase 1 — Introspection (v0.4.x)

Goal: Remove black-box nature of archives

Deliverables:
- `crushr info --list`
- File/directory structure listing
- Partial listing under damage
- Stable listing format (text + JSON)

Exit Criteria:
- Users can inspect archive contents without extraction

---

### Phase 2 — Provenance & Reporting

Goal: Make outputs traceable, audit-friendly, and ready for later evidence-handling workflows.

Deliverables:
- Tool version in every output
- Command invocation recording
- Input/output hashing
- Timestamping
- Structured report export (JSON baseline)
- Chain-of-custody sidecar design and initial event model

Example fields:
- tool_version
- command
- parameters
- input_hashes
- output_hashes
- timestamp
- archive_identifier
- report_identifier

Exit Criteria:
- Every run produces a traceable record
- Provenance schema is stable enough to support later custody workflows

#### Chain of Custody Sidecar

Purpose:
- provide a structured audit trail alongside a crushr archive
- preserve custody and provenance events without mutating the archive itself

Initial design direction:
- separate sidecar file bound to the archive identifier/hash
- append-only event model
- machine-readable and human-auditable

Example concept:
- `case001.crs`
- `case001.crs.custody.json`

Initial required event support:
- created
- verified
- transferred
- received
- extracted
- recovered

Initial required event fields:
- event_type
- timestamp
- actor
- system_or_host
- tool_version
- archive_hash
- notes (optional)

Non-goals initially:
- embedding mutable custody history inside the archive
- complex PKI/signing infrastructure
- implying that the sidecar alone makes the archive legally sufficient

---

### Phase 3 — Validation Framework

Goal: Prove correctness under defined conditions

Deliverables:
- Known-answer datasets
- Corruption datasets
- Expected output definitions
- Validation runner scripts
- Versioned validation reports

Requirements:
- Deterministic datasets
- Reproducible runs
- Public artifacts

Exit Criteria:
- crushr behavior can be independently verified

---

### Phase 4 — DFIR Alignment

Goal: Align with real-world forensic workflows

Deliverables:
- Intended Use / Limitations document
- Mapping to forensic workflows
- Output alignment with CASE / DFXML (initial mapping)
- Recovery classification documentation

Exit Criteria:
- A practitioner can understand where crushr fits in an investigation

---

### Phase 5 — External Validation

Goal: Gain credibility outside the project

Deliverables:
- External tester feedback
- Shared validation datasets
- Independent result comparisons
- Practitioner documentation

Optional:
- Lab pilot

Exit Criteria:
- At least one external party has validated behavior

---

### Phase 6 — Court Awareness (Future)

Goal: Prepare for legal scrutiny

Deliverables:
- Methodology documentation
- Error handling documentation
- Limitations clearly defined
- Expert-facing explanation docs

Exit Criteria:
- Claims are defensible and restrained

---

## 6. Validation Requirements

Validation must include:

- Defined scope of use
- Known-answer datasets
- Corruption scenarios
- Expected outcomes
- Repeatable execution
- Version tracking

Validation must NOT rely on:
- ad hoc testing
- anecdotal success
- unstructured datasets

---

## 7. Reporting Requirements

All outputs must support:

- reproducibility
- traceability
- machine parsing

Minimum report structure:

- tool_version
- archive_identifier
- operation_type
- parameters
- result_summary
- classification_counts
- timestamps
- input_hashes
- output_hashes where applicable

Where appropriate, reporting should be designed to align with later chain-of-custody/event sidecar workflows without coupling the archive itself to mutable custody history.

---

## 8. Non-Goals (Important)

crushr is NOT:

- a full forensic suite
- a disk imaging tool
- a replacement for established DFIR platforms

crushr IS:

- a failure-aware archival and recovery tool
- a deterministic inspection and extraction system

---

## 9. Risks

- Overclaiming forensic capability too early
- UI/UX drift breaking trust
- Non-deterministic behavior introduced by optimization
- Lack of external validation

---

## 10. Success Criteria

crushr is considered DFIR-ready (Level 1) when:

- behavior is validated and documented
- outputs are reproducible
- trust classifications are stable
- practitioners can use it without interpretation ambiguity

crushr is considered lab-adoptable (Level 2) when:

- validation artifacts are reusable
- reporting supports case documentation
- external validation exists

---

## 11. Immediate Next Steps

1. Complete v0.3.5 polish
2. Begin v0.4.0 introspection work
3. Define validation dataset structure
4. Begin provenance/report schema design

---

## Final Note

crushr does not become "forensic" by adding features.

It becomes forensic when:
- its behavior is provable
- its outputs are explainable
- and its claims are restrained

Everything else is just engineering.

