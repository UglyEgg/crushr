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
