<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_PRESERVATION_03 — operator-level completion validation

Environment caveat used for these checks:
- Host/container is Linux and current user is `root` by default.
- A non-privileged extraction path was validated via `nobody` to force explicit special-file degradation warnings.
- Sparse physical allocation visibility is filesystem-dependent; logical-size correctness is checked separately from allocation behavior.

---

## 1) Sparse round-trip validation

### Setup + pack/extract commands
```bash
BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres03_manual
rm -rf "$WORK"
mkdir -p "$WORK/src" "$WORK/out" "$WORK/out_deny"

truncate -s 64M "$WORK/src/sparse.bin"
printf 'BEGIN' | dd of="$WORK/src/sparse.bin" bs=1 seek=0 conv=notrunc status=none
printf 'TAIL' | dd of="$WORK/src/sparse.bin" bs=1 seek=$((64*1024*1024-4)) conv=notrunc status=none

mkfifo "$WORK/src/data.pipe"
mknod "$WORK/src/null.dev" c 1 3

"$BIN/crushr-pack" "$WORK/src" -o "$WORK/pres03.crs" --level 3
"$BIN/crushr-extract" "$WORK/pres03.crs" -o "$WORK/out"
```

### Verification commands
```bash
stat -c 'SRC %n size=%s blocks=%b blksize=%B' "$WORK/src/sparse.bin"
stat -c 'OUT %n size=%s blocks=%b blksize=%B' "$WORK/out/sparse.bin"
du -h "$WORK/src/sparse.bin" "$WORK/out/sparse.bin"
ls -ls "$WORK/src/sparse.bin" "$WORK/out/sparse.bin"
```

### Observed results
```text
SRC /tmp/crushr_pres03_manual/src/sparse.bin size=67108864 blocks=16 blksize=512
OUT /tmp/crushr_pres03_manual/out/sparse.bin size=67108864 blocks=16 blksize=512

8.0K /tmp/crushr_pres03_manual/src/sparse.bin
8.0K /tmp/crushr_pres03_manual/out/sparse.bin

8 -rw-r--r-- ... /tmp/crushr_pres03_manual/src/sparse.bin
8 -rw-r--r-- ... /tmp/crushr_pres03_manual/out/sparse.bin
```

Result:
- Logical size preserved (`64MiB` source == extracted).
- Sparse allocation behavior observable and preserved in this environment (same low block count / disk usage).

---

## 2) FIFO round-trip validation

### Commands
```bash
stat -c '%n mode=%f type=%F' "$WORK/out/data.pipe"
```

### Observed result
```text
/tmp/crushr_pres03_manual/out/data.pipe mode=11a4 type=fifo
```

Result:
- Extracted entry kind remains FIFO (not flattened to a regular file).

---

## 3) Device-node behavior validation

### Privileged path (root extraction)
Commands:
```bash
stat -c '%n mode=%f type=%F rdev=%t:%T' "$WORK/out/null.dev"
```

Observed:
```text
/tmp/crushr_pres03_manual/out/null.dev mode=21a4 type=character special file rdev=1:3
```

### Denied path (non-privileged extraction)
Commands:
```bash
mkdir -p "$WORK/out_deny"
chown -R nobody:nogroup "$WORK/out_deny"
su -s /bin/bash nobody -c "$BIN/crushr-extract '$WORK/pres03.crs' -o '$WORK/out_deny'" || true
test -e "$WORK/out_deny/null.dev" && stat -c '%n %F' "$WORK/out_deny/null.dev" || echo "OUT_DENY/null.dev missing"
```

Observed stderr snippets:
```text
WARNING[special-restore]: could not restore 'null.dev' at '/tmp/crushr_pres03_manual/out_deny/null.dev': Operation not permitted (os error 1)
```

Observed file check:
```text
OUT_DENY/null.dev missing
```

Result:
- In privileged context, device node is restored as device node.
- In denied context, explicit `WARNING[special-restore]` is emitted and extraction continues honestly (no fabricated regular file at that path).

---

## 4) Info visibility validation

### Command
```bash
"$BIN/crushr-info" "$WORK/pres03.crs"
```

### Relevant output snippet
```text
Metadata
  ...
  sparse files           present
  special files          present
```

Result:
- `crushr info` reports sparse/special metadata presence truthfully for archive content.

---

## 5) Backward-compatibility sanity check (pre-IDX5 + IDX5)

### Pre-IDX5 archive sample generation (IDX3)
Commands:
```bash
# generator source prepared under /tmp/idx4_gen (Rust helper), then:
cargo run -q --manifest-path /tmp/idx4_gen/Cargo.toml -- /tmp/crushr_pres03_manual/legacy_idx3.crs
```

### Pre-IDX5 checks
Commands:
```bash
"$BIN/crushr-info" /tmp/crushr_pres03_manual/legacy_idx3.crs
"$BIN/crushr-extract" /tmp/crushr_pres03_manual/legacy_idx3.crs -o /tmp/crushr_pres03_manual/legacy_out
cat /tmp/crushr_pres03_manual/legacy_out/legacy/file.txt
```

Observed:
```text
format markers         FTR4 + IDX3
...
legacy-idx3
```

### IDX5 checks
Commands:
```bash
"$BIN/crushr-info" "$WORK/pres03.crs" | sed -n '1,25p'
"$BIN/crushr-extract" "$WORK/pres03.crs" -o /tmp/crushr_pres03_manual/reextract_idx5
```

Observed:
```text
format markers         FTR4 + IDX5
```

Result:
- Current tooling handles both pre-IDX5 (`IDX3`) and current IDX5 archives for `info` and extraction paths.
