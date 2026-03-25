<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_PRESERVATION_02 — completion validation notes

Environment caveat:
- Runtime user in this container is `uid=0 gid=0` by default (root).
- Ownership restore warning-path was validated by running extract as `nobody`.

## 1) Hard-link round-trip validation

Setup + pack/extract:
```bash
WORK=/tmp/crushr_pres02_check
rm -rf "$WORK"
mkdir -p "$WORK/src_tree/dir" "$WORK/out"
printf 'payload-123\n' > "$WORK/src_tree/dir/file.txt"
ln "$WORK/src_tree/dir/file.txt" "$WORK/src_tree/dir/file_hard.txt"
/workspace/crushr/target/debug/crushr-pack "$WORK/src_tree" -o "$WORK/preserve.crs" --level 3
/workspace/crushr/target/debug/crushr-extract "$WORK/preserve.crs" -o "$WORK/out"
```

Verification command:
```bash
stat -c 'OUT %n inode=%i links=%h uid=%u gid=%g' \
  "$WORK/out/dir/file.txt" "$WORK/out/dir/file_hard.txt"
```

Observed result:
- Both extracted paths had identical inode (`3975105`) and link count `2`, proving hard-link relationship was preserved.

## 2) Ownership round-trip validation

Command:
```bash
stat -c 'SRC %n inode=%i links=%h uid=%u gid=%g' \
  "$WORK/src_tree/dir/file.txt" "$WORK/src_tree/dir/file_hard.txt"
stat -c 'OUT %n inode=%i links=%h uid=%u gid=%g' \
  "$WORK/out/dir/file.txt" "$WORK/out/dir/file_hard.txt"
```

Observed result in this environment:
- Source ownership: `uid=0 gid=0`.
- Extracted ownership: `uid=0 gid=0`.
- Restore applied successfully in root context (no warning on root-run extraction).

## 3) `info` metadata visibility validation

Command:
```bash
/workspace/crushr/target/debug/crushr-info "$WORK/preserve.crs"
```

Relevant output snippet:
```text
Metadata
  modes                  present
  mtime                  present
  xattrs                 present
  ownership              present
  hard links             present
```

Observed result:
- Presence flags matched the created fixture metadata (xattr + hard links + ownership + normal files/dirs).

## 4) Backward-compatibility sanity check (pre-IDX4)

Built a synthetic valid FTR4+IDX3 archive with a one-off generator (`/tmp/idx3_gen`) using `crushr-format::tailframe::assemble_tail_frame` and a manually encoded IDX3 index.

Generation command:
```bash
cargo run -q --manifest-path /tmp/idx3_gen/Cargo.toml -- /tmp/crushr_pres02_check/legacy_idx3.crs
```

Tooling checks:
```bash
/workspace/crushr/target/debug/crushr-info /tmp/crushr_pres02_check/legacy_idx3.crs
/workspace/crushr/target/debug/crushr-extract /tmp/crushr_pres02_check/legacy_idx3.crs -o /tmp/crushr_pres02_check/legacy_out
cat /tmp/crushr_pres02_check/legacy_out/legacy/file.txt
```

Observed result:
- `info` reported `format markers         FTR4 + IDX3`.
- Extraction succeeded and restored payload content `legacy-idx3`.

## 5) xattr + hard-link interaction check

Setup command (xattr write):
```bash
python - <<'PY'
import os
os.setxattr('/tmp/crushr_pres02_check/src_tree/dir/file.txt', b'user.crushr.test', b'xattr-preserved')
PY
```

Verification command:
```bash
python - <<'PY'
import os
print('XATTR_SRC', os.getxattr('/tmp/crushr_pres02_check/src_tree/dir/file.txt', b'user.crushr.test'))
print('XATTR_OUT', os.getxattr('/tmp/crushr_pres02_check/out/dir/file.txt', b'user.crushr.test'))
PY
```

Observed result:
- `XATTR_SRC b'xattr-preserved'`
- `XATTR_OUT b'xattr-preserved'`
- Hard-link preservation changes did not regress xattr round-trip for the preserved file.

## 6) Warning-path honesty check

Ran extraction as non-root (`nobody`) to force ownership-restore denial:

```bash
mkdir -p /tmp/crushr_pres02_check/out_nobody
chown -R nobody:nogroup /tmp/crushr_pres02_check/out_nobody
su -s /bin/bash nobody -c \
  "/workspace/crushr/target/debug/crushr-extract /tmp/crushr_pres02_check/preserve.crs -o /tmp/crushr_pres02_check/out_nobody"
```

Observed warning output:
```text
WARNING[ownership-restore]: could not restore '0:0' on '/tmp/crushr_pres02_check/out_nobody/dir': Operation not permitted (os error 1)
WARNING[ownership-restore]: could not restore '0:0' on '/tmp/crushr_pres02_check/out_nobody/dir/file.txt': Operation not permitted (os error 1)
WARNING[ownership-restore]: could not restore '0:0' on '/tmp/crushr_pres02_check/out_nobody/dir/file_hard.txt': Operation not permitted (os error 1)
```

Post-check command:
```bash
stat -c 'NOBODY_OUT inode=%i links=%h uid=%u gid=%g' \
  /tmp/crushr_pres02_check/out_nobody/dir/file.txt \
  /tmp/crushr_pres02_check/out_nobody/dir/file_hard.txt
```

Observed result:
- Extraction continued (non-fatal), files were emitted, and hard-link relation was preserved (`links=2`, same inode).
