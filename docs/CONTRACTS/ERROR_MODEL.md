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
- `3` repair required or repair performed
- `4` internal failure


Tool normalization (current workspace baseline):
- `crushr-info` and `crushr-fsck` return `2` for archive open failures and structural/parse/validation failures.
- `crushr-info` and `crushr-fsck` return `1` for usage/flag/argument errors.
