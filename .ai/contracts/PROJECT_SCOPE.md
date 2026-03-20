<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Project Scope

crushr is an integrity‑first archive design for modeling corruption impact and safe extraction.

The project intentionally avoids implementing:

- parity reconstruction
- speculative decompression
- heuristic recovery logic
- automatic archive repair

The system focuses on:

- deterministic verification
- corruption impact enumeration
- extraction of verified safe content only


Current minimal v1 extraction formalization:

- maximum safe extraction is a first-class reporting capability
- safe extraction set is computed deterministically from verified file->required-block mapping
- current scope is regular files with one block per file
