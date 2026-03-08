# Results

No empirical claims should exceed what is recorded in this document.

## Experiment: `crushr_p0s12f0_first_e2e_byteflip`

- **Artifact path:** `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip`
- **Fixture:** single-file input `fixture.txt` with three short text lines.
- **Corruption model:** `byteflip` via `crushr-lab corrupt`.
- **Seed / reproducibility:** seed `1337`, explicit touched offset `416` (`archive_len - 1`), and recorded input/output BLAKE3 in `corrupt.corrupt.json`.

### Observed clean behavior

- `crushr-pack` produced a real v1 archive (`clean.crs`) with footer/tail metadata readable by both tools.
- `crushr-info --json clean.crs` succeeded and reported footer and tail-frame fields.
- `crushr-fsck --json clean.crs` succeeded with `payload.verify.status = "ok"`.

### Observed corrupted behavior

- `crushr-lab` deterministically mutated one byte at offset `416` in `clean.crs`, producing `corrupt.crs` and `corrupt.corrupt.json`.
- `crushr-fsck --json corrupt.crs` failed with structural corruption (`parse FTR4: footer_hash mismatch`, exit code `2`).
- `crushr-info --json corrupt.crs` also failed to parse FTR4 (`footer_hash mismatch`).

### Initial interpretation

This experiment demonstrates the first real end-to-end structural corruption loop over a real v1 archive in this repository: pack → corrupt → inspect/verify.

### Limitation note

This is an initial structural validation result only. It is **not** a comparative benchmark, not a full corruption matrix, and not evidence about salvage/repair or payload-level recovery behavior.
