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
