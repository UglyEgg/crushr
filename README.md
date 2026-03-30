<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

[![Policy Gate](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml/badge.svg)](https://github.com/UglyEgg/crushr/actions/workflows/policy-gate.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![REUSE status](https://api.reuse.software/badge/github.com/UglyEgg/crushr)](https://api.reuse.software/info/github.com/UglyEgg/crushr)

# crushr

A deterministic archive system that preserves and exposes data truth under failure.

crushr is built around a narrow claim: archive behavior after damage should be explicit, bounded, and verifiable. Standard archive functionality exists to support preservation completeness, not to redefine the project as a convenience-first archiver.

## Intent

crushr is designed to preserve and expose the truth about data, especially under partial failure or corruption.

crushr prioritizes data integrity, explicit truth, and bounded failure behavior over maximum compression ratio.

It is not a convenience-first archive format. It is a system that defines what is known, what is degraded, and what must be refused.

## Guarantees

- Verified data is never silently corrupted or misrepresented  
- Unverifiable data is never presented as valid  
- Degraded or partial results are explicitly labeled and structured  
- Archive processing fails closed when required truth cannot be established  
- Filesystem writes are constrained and cannot escape intended boundaries  

## Behavior

### Validation vs Verification

crushr enforces a strict separation between:

- **Validation** — structural correctness of archive components  
- **Verification** — integrity correctness of data via cryptographic proof (BLAKE3)  

No output is considered trustworthy without explicit verification.

### Output Classification

All extraction and recovery results are classified into explicit trust classes:

- `canonical` — payload integrity is verified and required metadata is intact  
- `metadata_degraded` — payload integrity is verified, but metadata or structure is incomplete  
- `recovered_named` — payload integrity is verified and identity has been reconstructed within defined constraints  
- `recovered_anonymous` — payload integrity is verified but no reliable identity remains  
- `unrecoverable` — payload integrity cannot be proven to required standards  

These classes reflect the separation of payload integrity from metadata integrity.

### Processing Modes

- **Strict / Default**
  - Requires verified payload integrity and required metadata  
  - Refuses output when canonical guarantees cannot be established  

- **Recover**
  - Allows extraction of data that can be cryptographically verified  
  - Produces classified output when metadata or structure is incomplete  
  - Does not reconstruct, infer, or repair missing data  

## Why crushr exists

Most archive tooling assumes intact metadata, trustworthy structure, and clean success/failure outcomes.

crushr is built around the opposite assumption:

- data can be partially damaged  
- structure can be incomplete  
- metadata can be lost  
- partial truth still has value  

The system makes all outcomes explicit rather than assuming correctness.

## What crushr is now

The project provides:

- archive creation with `crushr pack`  
- integrity verification and strict-extraction viability checks with `crushr verify`  
- strict extraction with `crushr extract`  
- recovery-aware extraction with `crushr extract --recover`  
- archive inspection with `crushr info`  
- pre-extraction archive listing with `crushr info --list`  
- exact entry introspection with `crushr info --entry <logical/path>`  
- deterministic entry search with `crushr info --find <query>`  
- binary build and environment inspection with `crushr about`  

crushr archives are identified by format markers, not by filename extension.

The canonical default extension is:

- `.crs`

If no extension is supplied for `pack -o`, `.crs` is appended automatically.

## Core design principles

### Prove, don't guess

If a path, file identity, or recovery outcome cannot be proven from surviving archive evidence, crushr does not invent certainty.

### Separate trust classes explicitly

All output is classified into explicit trust classes rather than presented as uniformly valid.

### Fail closed by default

Strict operations refuse when canonical guarantees cannot be met. Recovery is explicit.

### Linux-first honesty

crushr is designed for real Linux archival workflows. Other platforms are not allowed to redefine the core metadata model.

### Archives should be inspectable

Archives are not opaque containers. Inspection, listing, and metadata visibility are first-class capabilities.

## Recovery model

`crushr extract` is strict by default.

If strict canonical extraction cannot be completed, the command refuses and requires explicit recovery mode:

- `crushr extract ...` → strict canonical extraction only  
- `crushr extract --recover ...` → recovery-aware extraction  

Recovery-aware extraction separates output by trust class:

- `canonical/`  
- `metadata_degraded/`  
- `recovered_named/`  
- `_crushr_recovery/anonymous/`  
- `_crushr_recovery/manifest.json`  

Recovery results are reported explicitly as:

- `canonical`  
- `metadata_degraded`  
- `recovered_named`  
- `recovered_anonymous`  
- `unrecoverable`  

Anonymous recovered files follow a deterministic naming policy:

- high-confidence classification → `file_<id>.<ext>`  
- medium-confidence classification → `file_<id>.probable-<type>.bin`  
- low/unknown confidence → `file_<id>.bin`  

The recovery manifest preserves structured classification and identity metadata for all recovered outputs.

## Linux-first preservation model

crushr's foundational model is the separation of payload integrity from metadata integrity. Tar-style preservation is layered onto that foundation.

### Preservation profiles

`crushr pack` supports explicit archive preservation contracts:

- `--preservation full` (default)  
- `--preservation basic`  
- `--preservation payload-only`  

The selected preservation profile is recorded in archive metadata and shown by `crushr info`.

#### full

Preserves the complete Linux-first metadata and entry-kind set currently supported.

#### basic

Preserves regular files, directories, empty directories, symlinks, hard links, mode, mtime, and sparse semantics.

Intentionally omits:

- xattrs  
- uid/gid  
- uname/gname  
- ACLs  
- SELinux labels  
- Linux capabilities  
- FIFOs  
- device nodes  

#### payload-only

Preserves only regular-file payload bytes plus logical tree reconstruction directories.

Intentionally omits:

- symlink semantics  
- hard link semantics  
- mode  
- mtime  
- sparse semantics  
- xattrs  
- ownership  
- ACLs  
- SELinux labels  
- Linux capabilities  
- FIFOs  
- device nodes  

If a selected profile excludes an entry kind, crushr warns and omits it rather than fabricating an alternate representation.

### Current Linux-first preservation scope

With `--preservation full`, crushr currently preserves:

- regular files  
- directories  
- empty directories  
- symlinks and link targets  
- hard links  
- sparse files  
- FIFOs  
- char/block device nodes  
- file mode / permissions  
- modification time (`mtime`)  
- extended attributes (`xattrs`)  
- numeric ownership (`uid` / `gid`)  
- optional ownership names (`uname` / `gname`) when available  
- POSIX ACL metadata (`system.posix_acl_access`, `system.posix_acl_default`)  
- SELinux label metadata (`security.selinux`)  
- Linux file capability metadata (`security.capability`)  

Where preservation or restoration cannot be applied due to platform or permission constraints, crushr degrades honestly and warns rather than silently pretending success.

### Long-term preservation goal

crushr aims to support Linux-first archive fidelity suitable for serious tar-based workflows.

This is a staged goal. Full parity is not implied.

## Archive introspection

crushr archives are inspectable without extraction.

- `crushr info` provides archive-level introspection: structure, preservation profile, and declared metadata scope  
- `crushr info --list` provides entry-level introspection: listing, classification, and attributes without extraction  
- `crushr info --entry <logical/path>` provides exact-path truth for one entry without extraction  
- `crushr info --find <query>` provides deterministic substring search over stable logical identities  

Current behavior is fail-closed:

- if structure can be proven, it is reported  
- if required metadata is missing, structure is not invented  
- directory views are derived from stored logical paths  

## Security and assurance

crushr publishes a self-assessed security and assurance set covering:

- threat model  
- integrity guarantees  
- verification semantics  
- architectural invariants  
- control and audit documents  

crushr is designed in alignment with ISO/IEC 27001 control principles (self-assessed) for relevant controls.

This is not a certification claim.

## Documentation

Public material lives under `docs/`.

Primary entry points:

- `docs/index.md` — site landing page  
- `docs/why-crushr.md` — positioning  
- `docs/whitepaper/index.md` — technical whitepaper  
- `docs/reference/index.md` — concise technical reference  
- `docs/chronicles/index.md` — historical development  

Canonical behavior and guarantees are defined by this README and the security documentation.

## Internal project control

Internal planning and control material exists under:

- `.ai/`  
- `.ai/contracts/`  

These are not part of the public documentation surface.

## Product boundary

- **Stable product surface:** `pack`, `verify`, `extract`, `extract --recover`, `info`, `info --list`, `info --entry`, `info --find`, `about`  
- **Bounded internal surface:** workspace Rust crates/modules  
- **Experimental/lab-only surface:** `crushr lab` and research tooling  

`crushr lab` is an internal development harness used to evaluate potential features under controlled conditions.

It is not part of the user-facing product surface and is not covered by stability guarantees. Only behavior that demonstrates clear value and aligns with crushr’s guarantees is promoted into the canonical system.

## CLI presentation

Commands share a consistent presentation model:

- structured output  
- consistent terminology  
- deterministic summaries  
- restrained motion for active operations  

### Silent/scriptable mode

Script-oriented paths emit concise deterministic output suitable for automation.

## Evidence-oriented workflow

1. Data is collected  
2. Archives are created  
3. `verify` establishes integrity and strict-extraction viability  
4. `extract` returns canonical output when possible  
5. `extract --recover` returns classified output when required  
6. Results remain reproducible and verifiable  

## Roadmap direction

- complete Linux-first preservation semantics  
- expand archive introspection  
- deepen structural visibility  
- begin compression benchmarking once semantics stabilize  
- explore reproducible archive modes  

## Product version governance

- `VERSION` is the canonical version source  
- update `VERSION`, then run sync scripts  
- validate with version checks  

## License

Code is dual-licensed under MIT OR Apache-2.0.

Documentation and diagrams are licensed under CC-BY-4.0.

The repository follows REUSE compliance with SPDX metadata.
