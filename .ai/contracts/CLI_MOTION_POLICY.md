<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CLI Motion and Animation Policy

This contract defines restrained motion behavior for long-running CLI operations.

## Goals

- Communicate real work in progress.
- Highlight phase transitions without repaint-heavy churn.
- Keep completed output stable and readable.
- Preserve calm, trustworthy operator tone.

## Anti-goals

- No decorative-only effects.
- No color-cycling/RGB effects.
- No fake percentage/ETA/progress bars without real data.
- No motion on final static summaries.

## What may animate

- Only **active phase rows** in the `Progress` section for long-running commands (`pack`, `extract`, `extract --recover`, `verify`).
- Optional bounded active detail text for data already available cheaply (for example phase-local counts).

## What must not animate

- `Result`, `Target`, trust-class, warnings/failure domain, and other settled output.
- Completed phase rows.
- `info` command default human summary surface.

## Phase-state semantics

- `PENDING`: reserved for not-yet-started work when explicitly rendered.
- `RUNNING`: active phase may animate in interactive TTY mode.
- `COMPLETE` / `OK` / `VERIFIED`: phase settles to a stable non-animated row.
- `FAILED` / `REFUSED` / `DEGRADED`: phase settles to stable non-animated failure/degraded rows.

## Redraw cadence and churn limits

- Full motion mode: ~120 ms tick cadence.
- Reduced motion mode: ~240 ms tick cadence.
- No full-screen repaint loops; update only the currently active phase line.
- Clear active ephemeral line before writing stable settled rows.

## Motion controls

- `CRUSHR_MOTION=full|reduced|off` controls motion mode.
- `CRUSHR_NO_MOTION=1` forces no-motion behavior.
- Default mode is `full` for interactive TTY; non-TTY output never animates.

## Non-color behavior

- Motion does not depend on color.
- In no-color terminals, animation remains structural/textual and semantic labels remain clear.

## Non-interactive/stdout-not-tty behavior

- Never emit spinner carriage-control output into pipes/logs.
- Render stable text rows only.
