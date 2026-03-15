# crushr

## Project Positioning Statement

crushr is an integrity-first archival and compression format designed around a single premise:

> When data corruption occurs, the system should maximize what can still be proven and recovered.

Most archive formats optimize for convenience, speed, and compatibility. They assume the archive will remain structurally intact. When structural metadata is damaged, these formats typically fail catastrophically.

crushr explores a different design space: an archive format intentionally built to remain partially recoverable even after significant structural damage.

The format incorporates multiple layers of verifiable metadata and recovery signals so that salvage tools can reconstruct valid data even when canonical indexes are unavailable.

---

## Core Thesis

The central thesis of crushr is:

**Recoverability under corruption should be treated as a first-class design property of archive formats.**

Rather than relying on a single metadata structure such as a central directory or index, crushr experiments with redundant and distributed metadata strategies including:

- deterministic block identities
- repeated mapping information
- embedded file identity records
- distributed checkpoint structures
- verification through cryptographic hashing

These mechanisms allow salvage tools to reason about archive contents even when portions of the structure are damaged.

---

## Project Goals

The goals of the crushr project are:

### 1. Integrity First
The format prioritizes verifiability and data integrity over convenience features.

### 2. Deterministic Recovery
Recovery behavior should be predictable and evidence-driven rather than heuristic guesswork.

### 3. Graceful Degradation
Partial corruption should result in partial recovery rather than total failure.

### 4. Empirical Evaluation
Recovery strategies are tested through controlled corruption experiments and comparative analysis.

### 5. Transparent Design
The format and its recovery behavior are documented and reproducible.

---

## What crushr Is Not

crushr is **not intended to replace mainstream archive formats** such as ZIP or TAR for everyday use.

Those formats are optimized for compatibility, ecosystem support, and simplicity.

crushr instead explores a niche design space focused on:

- archival durability
- forensic recoverability
- experimental archive structure research

---

## Intended Use Cases

The format may be useful in contexts where data durability and recoverability matter more than convenience:

- research data preservation
- archival storage
- forensic evidence containers
- reproducible artifact storage
- integrity-focused backup systems

In these scenarios, the ability to recover verified data from a damaged archive can be more valuable than an all-or-nothing design.

---

## Architecture Overview

The crushr format is built around several structural principles.

### 1. Block-Structured Data Layout
Archive payloads are divided into independently identifiable blocks. Each block can be verified independently using cryptographic hashes.

This enables salvage tools to validate data fragments without relying on a single global index.

### 2. Redundant Metadata Surfaces
Multiple metadata surfaces describe the mapping between files and blocks. These may include:

- canonical indexes
- checkpoint maps
- file identity records
- path checkpoints
- file manifests

These redundant signals allow reconstruction of file structure even when the canonical index is lost.

### 3. Deterministic Verification
Every recovery path must ultimately verify data using cryptographic hashes.

The system prefers provable recovery over speculative reconstruction.

### 4. Salvage-First Tooling
The project includes dedicated salvage tooling capable of analyzing damaged archives and producing recovery plans based on the surviving evidence.

---

## Research Component

crushr includes an experimental testing framework used to evaluate recovery strategies under simulated corruption.

This framework generates comparative results across multiple archive variants and corruption scenarios. The goal is to measure how design choices affect recoverability.

Rather than relying solely on theoretical reasoning, the project evaluates format variants through empirical experiments.

---

## AI Co-Development

crushr was developed through an open human-AI engineering collaboration.

The project intentionally documents this process as part of its design history.

AI systems were used to:

- assist with implementation
- explore design alternatives
- generate experimental work packets
- perform hostile code reviews
- assist with documentation and architecture refinement

The human developer retained architectural authority and validation responsibility.

All design decisions, acceptance criteria, and testing loops remained human-directed.

This project therefore represents an experiment in **AI-assisted systems engineering**, where AI acts as a development partner rather than a simple code generator.

---

## Why This Matters

Complex systems projects are often constrained by limited development resources.

AI collaboration can reduce the cost of exploring large design spaces by enabling rapid iteration, implementation assistance, and structured review cycles.

crushr demonstrates how a single developer can pursue a complex experimental systems project with the assistance of AI while maintaining engineering rigor.

---

## Project Status

crushr is an active research and development project.

The project currently focuses on:

- format design iteration
- recovery model experimentation
- corruption testing
- architecture stabilization
- boundary hardening and repo scrub work where needed

Future development will determine which experimental mechanisms become permanent parts of the format specification.

---

## Long-Term Vision

The long-term vision for crushr is to produce:

1. A stable integrity-first archival format
2. A robust salvage and verification toolchain
3. A well-documented case study of AI-assisted systems engineering

Even if crushr ultimately remains a niche format, it aims to contribute useful ideas to the broader conversation around archival resilience and data integrity.

---

## Summary

crushr is an experimental archival format built around recoverability, verification, and empirical design iteration.

It explores how archive systems might behave when corruption is treated as an expected condition rather than a catastrophic failure.

The project also serves as a demonstration of human-AI collaborative engineering applied to a non-trivial systems problem.

