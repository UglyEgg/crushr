# Experiment Method

Goal: validate crushr's Failure-Domain Determinism claim against controlled corruption.

## Datasets
- many small files
- few large files
- mixed entropy / mixed media

## Corruption models
- single-byte flip
- random N-byte flips
- full-range overwrite
- truncation
- tail corruption

## Baselines
- 7z
- zip
- tar+zstd

## Metrics
- files fully extractable
- affected files enumerated pre-extraction
- blast radius in files / bytes
- time to detect corruption
- time to enumerate impact

## Reproducible runner (current validation set)

Run the current deterministic structural-validation experiment and refresh artifacts:

```bash
cargo run -q -p crushr-lab --bin crushr-lab -- run-first-experiment
```

Artifacts are written to:

- `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip`

This runner reproduces only the current small structural validation loop (pack → byteflip corruption → info/fsck checks). It is not a benchmark or comparative matrix harness.
