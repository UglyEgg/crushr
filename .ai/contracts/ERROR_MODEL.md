<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Error Model

Primary classes:
- user/configuration error
- unsupported feature / unsupported version
- structural corruption
- verification failure
- I/O failure
- internal bug

Exit code guidance:
- `0` success
- `1` user error
- `2` corruption / verification failure
- `3` partial extraction / refused files (policy-controlled)
- `4` internal failure

Tool normalization (current workspace baseline):
- `crushr-info` and `crushr-extract --verify` return `2` for archive open failures and structural/parse/validation failures.
- `crushr-info` and `crushr-extract --verify` return `1` for usage/flag/argument errors.
- `crushr-extract` strict refusal behavior is policy-controlled via `--refusal-exit <success|partial-failure>`:
  - `success` (default): valid archive structure with one or more refused files exits `0`.
  - `partial-failure`: valid archive structure with one or more refused files exits `3`.
  - Structural/open/parse failures still exit `2` regardless of refusal policy.
- `crushr-extract --json` emits deterministic machine-readable extraction reports:
  - Extraction result contract reference: `.ai/contracts/EXTRACTION_RESULT_V1.md`
  - strict mode: explicit maximum-safe-extraction contract (`overall_status`, `maximal_safe_set_computed`, deterministic `safe_files`, deterministic `refused_files`, `safe_file_count`, `refused_file_count`, stable refusal reason `corrupted_required_blocks`)
  - structural/open/parse failure: nonzero exit with `overall_status = "error"` envelope (no success/partial-success report emitted)
