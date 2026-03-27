<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Extracting archives

crushr has two extraction modes:

- strict extraction
- recovery-aware extraction

Basic forms:

```bash
crushr extract <archive.crs> -o <out-dir>
crushr extract <archive.crs> -o <out-dir> --recover
```

## Strict extraction

Strict extraction is the default.

```bash
crushr extract archive.crs -o ./out
```

Strict mode means:

- crushr expects canonical restoration
- if the archive contract cannot be satisfied, extraction refuses
- partial trust is not silently accepted

This is the right mode when you want:
- correctness first
- clear refusal on degraded outcomes
- no mixed result tree

!!! tip "Use strict mode first"
    If the archive is healthy and your system can restore the required metadata, strict extraction gives the cleanest result.

## Recovery-aware extraction

Recovery mode is for:
- damaged archives
- partial truth
- environments where some metadata cannot be restored
- situations where you want everything crushr can still prove

```bash
crushr extract archive.crs -o ./out --recover
```

Recovery mode separates outputs into trust buckets.

Typical layout:

```text
out/
  canonical/
  metadata_degraded/
  recovered_named/
  _crushr_recovery/
    anonymous/
    manifest.json
```

## What each output bucket means

| Bucket | Meaning |
|---|---|
| `canonical/` | Fully restored according to the archive's recorded preservation profile |
| `metadata_degraded/` | Data and identity are correct, but required metadata could not be restored |
| `recovered_named/` | Recovered with evidence-backed identity, but not fully canonical |
| `anonymous/` | Recovered content with no trustworthy original name/path |
| `manifest.json` | Structured record of what happened |

## Why strict mode can fail even when data is intact

This is the part many users miss.

If an archive was created with `--preservation full`, canonical extraction may require:
- ownership
- xattrs
- ACLs
- SELinux labels
- capabilities
- special file restoration

If your environment cannot restore those, strict extract can fail even though the file bytes are fine.

That is expected. It is not crushr being difficult for fun. It is the tool refusing to lie.

## Reading extraction results

### `canonical`
This is the best outcome. The archive contract was satisfied.

### `metadata_degraded`
The file is still useful, but the environment blocked full restoration.

Common reasons:
- running without privilege
- target filesystem lacks support
- security policy blocks restore

### `recovered_named`
The file was recovered, but not to full canonical standards.

### `recovered_anonymous`
The content survived, but crushr could not prove the original identity.

### `unrecoverable`
Nothing trustworthy could be restored for that entry.

## Common workflows

### Normal extraction

```bash
crushr extract archive.crs -o ./restore
```

### Recovery extraction

```bash
crushr extract archive.crs -o ./restore --recover
```

### Verify-only extraction path

If you want a strict integrity check before extraction, use:

```bash
crushr verify archive.crs
```

## What is expected versus not expected

### Expected

- strict mode refusal when canonical restoration is not possible
- recovery-mode separation into multiple trust buckets
- clear warnings on blocked metadata restoration

### Not expected

- silent fallback from strict to recover
- fake canonical output
- guessed names for anonymous recovery

## Summary

Use:
- strict extraction when you want all-or-nothing truth
- recovery extraction when you want everything crushr can still prove

If you see `metadata_degraded`, the archive may still be fine. The environment may be the limiting factor.
