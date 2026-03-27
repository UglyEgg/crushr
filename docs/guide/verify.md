<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Verifying archives

`crushr verify` checks archive integrity without extracting files.

Basic form:

```bash
crushr verify <archive.crs>
```

## What it does

Verification answers:

- is this a valid crushr archive?
- can the archive structure be read correctly?
- do payloads and recorded integrity data still agree?

It does **not** restore files.
It does **not** test whether your target environment can apply metadata during extraction.

That last point matters.

!!! note "Verify is about archive truth"
    `verify` checks the archive itself. Extraction is where crushr learns whether your system can actually restore ownership, ACLs, SELinux labels, capabilities, and other metadata.

## Example

```bash
crushr verify backup.crs
```

## When to use it

Use `verify` when you want to:
- sanity-check an archive before extraction
- validate archives after transfer
- separate archive integrity from extraction-time environment issues

## How to interpret results

### Verified / complete
The archive is internally consistent and payload verification succeeded.

### Partial / failed
Something in the archive could not be verified.

If you need data anyway, the next step is usually:
- inspect with `info`
- then try `extract --recover`

## Common mistake

Do not assume this means extraction will be fully canonical.

An archive can verify correctly and still produce `metadata_degraded` extraction results if the target machine cannot apply required metadata.

## Summary

`verify` tells you whether the archive itself is sound.

It does not tell you whether your destination system can fully restore everything the archive contains.
