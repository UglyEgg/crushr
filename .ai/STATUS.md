**This is the single source of truth for current state.**

## Current Phase / Step

- Phase: F
- Step: F.3
- Fix iteration: 0

## Current Objective

Implement the first real end-to-end corruption experiment loop over a produced v1 archive (pack -> deterministic corrupt -> fsck/info) and record an initial empirical result artifact.

## What Changed (since last Step)

- Extended `crushr-lab corrupt` argument handling to support deterministic corruption metadata capture with explicit `--model`, `--seed`, and optional `--offset`, while preserving the bounded byteflip-only model for now.
- Added integration coverage in `crates/crushr-core/tests/first_corruption_experiment.rs` for:
  - full single-file loop (`crushr-pack` -> `crushr-lab` -> `crushr-info`/`crushr-fsck`)
  - clean archive success expectations
  - corrupted archive failure expectations
  - corruption determinism (same seed/model/offset yields identical output + log)
  - doc/artifact linkage assertion for the recorded experiment id.
- Recorded the first real experiment artifacts at:
  - `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/`
  - includes `clean.info.json`, `clean.fsck.json`, `corrupt.corrupt.json`, stderr/exit-code captures, and `experiment_manifest.json`.
- Updated `docs/RESEARCH/RESULTS.md` with experiment identifier, fixture, corruption model, reproducibility details (seed/offset), observed clean/corrupt behavior, and explicit scope limitation.

## What Remains (next actions)

1. Add additional controlled fixture classes for Phase F.3 (many-small and mixed datasets) while keeping deterministic corruption metadata.
2. Extend corruption model coverage incrementally (truncate/tail overwrite) without changing format or product contracts.
3. Prepare bounded Phase F.4 baseline-comparison packet once explicit approval is provided.

## How to Build / Test (best known)

- `cargo fmt --all`
- `cargo test -p crushr-core --test first_corruption_experiment`
- `cargo test -p crushr-core --test minimal_pack_v1`

## Active constraints / gotchas

- `crushr-info` currently returns exit code `1` on parse failures while `crushr-fsck` maps structural corruption to exit code `2`; this inconsistency is pre-existing and not changed in this Step.
- Current experiment is intentionally single-path and structural only (no decompression/salvage/repair claims).
