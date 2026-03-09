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
- `3` partial extraction / refused files (policy-controlled), or repair required/performed in repair-oriented tools
- `4` internal failure


Tool normalization (current workspace baseline):
- `crushr-info` and `crushr-fsck` return `2` for archive open failures and structural/parse/validation failures.
- `crushr-info` and `crushr-fsck` return `1` for usage/flag/argument errors.

- `crushr-extract` extraction mode is explicit via `--mode <strict|salvage>` (default `strict`). Strict behavior is unchanged when salvage is not selected.
- `crushr-extract` strict refusal behavior is policy-controlled via `--refusal-exit <success|partial-failure>`:
  - `success` (default): valid archive structure with one or more refused files exits `0`.
  - `partial-failure`: valid archive structure with one or more refused files exits `3` (unchanged semantics).
  - Structural/open/parse failures still exit `2` regardless of refusal policy.
- `crushr-extract --json` emits deterministic machine-readable extraction reports:
  - strict mode: same existing contract (`overall_status`, deterministic `extracted_files`, deterministic `refused_files`, stable refusal reason `corrupted_required_blocks`)
  - salvage mode: same extraction/refusal semantics plus explicit `mode = "salvage"` and deterministic ordered `salvage_decisions` entries with decisions `extracted_verified_extents` or `refused_corrupted_required_blocks`
  - structural/open/parse failure: nonzero exit with `overall_status = "error"` envelope (no success/partial-success report emitted)
