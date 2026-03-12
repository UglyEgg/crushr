# crushr — Project State

crushr is an **integrity-first archival container** designed for deterministic corruption analysis and maximum safe extraction.

The system exists primarily as:

- a research artifact
- a white-paper implementation
- a reference archive integrity design

It is **not intended to replace general-purpose compression formats** like zip or 7z.

---

# Canonical Toolchain

Active tools:

- crushr-pack
- crushr-info
- crushr-fsck
- crushr-extract
- crushr-lab

Active libraries:

- crushr-core
- crushr-format

Legacy monolith code paths remain in the repository **for historical reference only**.

---

# Archive Model

Minimal v1 archive layout:

BLK3 blocks → IDX3 index → tail frame → FTR4 footer

Verification pipeline:

footer → tail frame → index → blocks → file impact

---

# Current Implementation Scope

The current minimal v1 implementation focuses on:

- regular files
- **one block per file**

Each file is currently stored as a single BLK3 block.

---

# Current Capabilities

- archive packing
- deterministic archive opening
- corruption verification
- block-level corruption detection
- file-level impact enumeration
- strict extraction
- machine-readable extraction reports
- deterministic corruption experiments
- cross-format experiment scaffolding

---

# Known Limitations

The minimal v1 system intentionally excludes:

- multi-block files
- dictionary compression paths (DCT1)
- ledger metadata (LDG1)
- streaming extraction
- metadata fidelity
- parity or reconstruction systems

No speculative recovery exists by design.

---

# Current Development Focus

Next milestone:

**Maximum Safe Extraction Formalization**

This step formalizes strict extraction as a reportable capability for the crushr white paper.
