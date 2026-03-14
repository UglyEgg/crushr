# PHASE_2_LOCKS.md

## Status
LOCKED

## Purpose
Define the experimental boundaries for Phase 2 so all testing, capture, aggregation, and reporting remain reproducible, bounded, and claim-safe.

## Claim policy
All claims in the whitepaper must be evidence-bound.
No claim may exceed what is directly supported by measured Phase 2 outputs.

Allowed claim categories:
- corruption detectability
- extraction outcome behavior
- blast-radius observability
- refusal granularity
- diagnostic specificity
- bounded failure-domain behavior

Disallowed claim categories unless separately tested:
- overall superiority
- compression efficiency
- speed/performance superiority
- durability under arbitrary catastrophic corruption
- general-purpose archive replacement claims

## Core publication matrix

### Formats
- crushr
- zip
- tar+zstd
- tar+gz
- tar+xz

### Datasets
- smallfiles
- mixed
- largefiles

### Corruption types
- bit_flip
- byte_overwrite
- zero_fill
- truncation
- tail_damage

### Neutral target classes
- header
- index
- payload
- tail

These target classes are cross-format neutral and are the only targets allowed in the core comparison matrix.

### Magnitude tiers
- 1B
- 256B
- 4KB

### Seed policy
Seeded-random only.
Fixed seeds:
- 1337
- 2600
- 65535

### Core matrix size
3 datasets × 5 formats × 5 corruption types × 4 target classes × 3 magnitudes × 3 seeds = 2700 runs

## Addendum scope

### Optional dataset addendum
- binaryheavy

### Optional magnitude addendum
- 16B

### Optional crushr-specific internal targeting addendum
Allowed only as a separate appendix/addendum.
These runs may not be mixed into the core cross-format comparison claims.

Allowed internal targets may include actual crushr structural terms as implemented in code.

## Stress appendix
A non-core catastrophic corruption appendix may be run at approximately 50% corruption.
These runs are exploratory and may not be used for nuanced comparative claims.

## Required captured metrics
Each run record must capture at minimum:
- scenario_id
- dataset
- format
- corruption_type
- target_class
- magnitude
- magnitude_bytes
- seed
- source_archive_path
- corrupted_archive_path
- tool_kind
- executable
- argv
- cwd (if available)
- detected_pre_extract
- outcome_class
- files_total
- files_safe
- files_refused
- files_unknown (if applicable)
- diagnostic_specificity
- exit_code
- stdout_path
- stderr_path
- json_result_path (if available)
- has_json_result
- invocation_status
- tool_version (truthful detection/unsupported/unavailable)

## Outcome classes
The normalized result surface must classify outcomes into:
- full_success
- partial_success
- refused
- hard_failure

## Diagnostic specificity ladder
The normalized result surface must classify diagnosability into:
- none
- generic
- structural
- precise

## Output artifact layout
/experiments
  /runs
  /archives
  /corrupted
  manifest.json
  summary.json
  summary.csv
  methodology.md

## Reproducibility rules
- scenario enumeration must be deterministic
- output ordering must be deterministic
- seed lists are fixed
- raw results must be preserved
- normalized summaries must never replace raw outputs

## Whitepaper-critical result families
The experiment system must support generation of:
1. outcome matrix by format and corruption type
2. blast-radius distribution by format
3. detection/diagnosability summary by format
4. severity curve by corruption magnitude
5. scenario-level comparative appendix table
