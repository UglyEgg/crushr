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

Maximum safe extraction is now a first-class deterministic extraction report capability. For minimal v1 archives (regular files, one block per file), `safe_files` are those whose required block verifies cleanly, and `refused_files` are those whose required block is corrupted (`corrupted_required_blocks`). No speculative recovery is implemented.
