# Updated docs/guide/info.md

<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Inspecting archives

Use `crushr info` when you want to inspect an archive without extracting it.

Basic forms:

crushr info <archive.crs>
crushr info <archive.crs> --list
crushr info <archive.crs> --entry <logical/path>
crushr info <archive.crs> --find <query>
crushr info <archive.crs> --propagation

`info` is an introspection surface. It does not extract files, mutate the archive, or write to the filesystem.

---

## `crushr info`

This shows the archive contract and archive-level truth surface in human-readable form.

Typical sections include:

- archive path
- format version
- global flags
- preservation profile
- structural summary
- verification summary
- whether strict extraction is supportable

---

## `crushr info --list`

Lists archive contents without extraction.

- metadata/index-driven
- fail-closed
- deterministic

If crushr cannot prove the listing, it does not guess.

Use:
- `--entry` for a single path
- `--find` for search

---

## `crushr info --entry`

Exact logical-path lookup without extraction.

crushr info archive.crs --entry src/main.rs
crushr info archive.crs --entry src/main.rs --json

Fields include:

- logical path
- trust class
- payload verified
- metadata complete
- extent count
- size bytes
- payload BLAKE3
- logical range (`start..end`) in both hex and decimal form
- identity source
- strict extraction supported
- reason (if non-canonical)

Trust classes:

- canonical
- metadata_degraded
- recovered_named
- recovered_anonymous
- unrecoverable

Notes:

- payload integrity is independent from metadata completeness
- logical range is logical, not archive-physical
- JSON mode is the authoritative machine-readable surface
- not-found is explicit and deterministic (no fallback to search)

---

## `crushr info --find`

Deterministic search over known identities.

crushr info archive.crs --find src
crushr info archive.crs --find .rs --json

Behavior:

- substring match (default)
- deterministic lexical ordering
- no extraction
- no identity guessing

Search scope:

- canonical
- metadata_degraded
- recovered_named

Not included:

- recovered_anonymous (no stable identity)

Output:

- default → CLI presentation
- --json → structured output

Reserved flags:

- --find-mode substring
- --find-limit <n>

Unsupported modes produce deterministic errors.

---

## `crushr info --propagation`

Explains *why* entries are impacted.

crushr info archive.crs --propagation
crushr info archive.crs --propagation --json

Answers:

- what corruption was detected
- which structures are affected
- which entries are impacted
- why they are impacted
- whether canonical extraction is blocked
- what trust-class outcomes are supported by current evidence

Modes:

- default → human-readable explanation
- --json → full propagation graph


## `crushr info --report propagation`

Use this when you need deterministic dependency and impact explanation without extraction.

```bash
crushr info archive.crs --json --report propagation
```

The propagation report separates:

- dependency graph (`nodes`, `edges`)
- currently detected corruption (`detected_corruption`)
- activated impact from current corruption (`activated_impacts`)
- per-entry dependency + impact explanation (`entry_impacts`)

Per-entry explanation includes:

- direct vs propagated dependencies
- bounded dependency reasons (for example `requires_index`, `requires_block_payload`)
- whether canonical extraction is currently blocked
- trust classes supported by current evidence only

The report is explanatory only. It does not imply repair behavior and does not make speculative survivability claims.

Rules:

- deterministic
- evidence-based
- no speculation
- no repair
- no survivability prediction

---

## `info` vs `verify` vs extraction

info:
- what exists
- what can be proven
- what can be inspected
- whether strict extraction is supportable

verify:
- whether strict extraction requirements are satisfied
- whether recovery is required

extract:
- what was actually restored
- final trust-class outcomes

---

## Common workflow

crushr info archive.crs
crushr info archive.crs --list
crushr info archive.crs --entry src/main.rs
crushr info archive.crs --find src
crushr info archive.crs --propagation

Then:

- crushr verify
- crushr extract
- crushr extract --recover

---

## Summary

Use info to answer:

- what exists?
- what can be proven?
- is strict extraction supportable?
- what do we know about this entry?
- why is this entry impacted?

Use verify to answer:

- will strict extraction succeed?

Use extract to answer:

- what actually restored?
