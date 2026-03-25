<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CLI Visual Semantics Contract

This contract defines shared user-facing CLI visual tokens and status semantics.

## Visual tokens

The shared presenter token set is implemented in `crates/crushr/src/cli_presentation.rs`.

1. `TitleProductLine` — top product/action line.
2. `SectionHeader` — section titles.
3. `PrimaryLabel` — key labels in key/value rows.
4. `SecondaryText` — muted supporting text.
5. `ActiveRunning` — active/running phase state.
6. `Pending` — pending/not-yet-started state.
7. `CompleteSuccess` — success/complete state.
8. `WarningDegraded` — degraded-but-usable state.
9. `FailureRefusal` — failure/refusal/unrecoverable state.
10. `InformationalNote` — neutral informational notes.
11. `TrustCanonical` — canonical trust class.
12. `TrustMetadataDegraded` — metadata-degraded trust class.
13. `TrustRecoveredNamed` — recovered named trust class.
14. `TrustRecoveredAnonymous` — recovered anonymous trust class.
15. `TrustUnrecoverable` — unrecoverable trust class.

## Status semantics

Primary shared status vocabulary:

- `PENDING`
- `RUNNING`
- `COMPLETE`
- `DEGRADED`
- `FAILED`
- `REFUSED`

Additional bounded statuses used by existing flows:

- `VERIFIED`
- `OK`

`PARTIAL` is treated as a compatibility input and is rendered as `DEGRADED` in user-facing output.

## Recovery trust semantics

Recovery-aware output must keep trust classes distinct and non-overloaded:

- `CANONICAL` (safe/canonical)
- `METADATA_DEGRADED` (data/path/name proven but required metadata restore failed)
- `RECOVERED_NAMED` (caution, degraded trust)
- `RECOVERED_ANONYMOUS` (stronger caution than named recovery)
- `UNRECOVERABLE` (loss/failure)

## Color usage policy

- Color is semantic, not decorative.
- Equal meaning maps to equal visual treatment across commands.
- No-color mode must preserve hierarchy and semantic clarity via labels and structure alone.
