# crushr Development Status

Current Phase: Phase 1 — Integrity Intelligence

Active Objective:

Step 1.2 — Maximum Safe Extraction Formalization (completed)

Goal status:

Implemented deterministic maximum safe extraction reporting for `crushr-extract --json` on structurally valid minimal v1 archives.

What changed:

- `crushr-extract --json` now explicitly reports the maximal safe extraction set via:
  - `overall_status`
  - `maximal_safe_set_computed`
  - `safe_files`
  - `refused_files`
  - `safe_file_count`
  - `refused_file_count`
- Refused files now carry typed deterministic reason `corrupted_required_blocks`.
- Existing refusal-exit semantics were preserved (`success` => 0 on partial refusal, `partial-failure` => 3 on partial refusal, structural/open/parse => 2, usage => 1).

Active constraints:

- Minimal v1 extraction scope remains limited to regular files with one-block-per-file mapping.
- No speculative recovery, reconstruction, or hole-filling behavior exists.

Next actions:

- Move to Step 1.3 (Extraction Result Formalization) planning packet.
