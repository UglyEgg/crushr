# Failure Domain Formalization

The crushr repository now contains a working implementation of the failure-domain model.

Current capabilities include:

- archive packing
- deterministic archive parsing
- block-level corruption verification
- file impact enumeration
- strict extraction
- JSON extraction reports
- reproducible corruption experiments

The minimal v1 system stores **each file as a single block**, simplifying analysis of corruption boundaries.

Maximum safe extraction is currently implemented behaviorally through strict extraction and refusal reporting.
