<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# CRUSHR_PRESERVATION_04 — operator-level completion validation

Environment caveats for this run:
- Host/container is Linux; commands executed as `root` unless explicitly switched to `nobody`.
- ACL CLI tooling (`setfacl`, `getfacl`) is **not installed** in this environment, so ACL round-trip could not be operator-validated with `getfacl` here.
- SELinux label xattrs are writable/readable in this environment (`security.selinux` via xattr), and `ls -Z` reports labels.

---

## 1) ACL round-trip validation

### Tooling availability check (required caveat)
```bash
command -v setfacl || true
command -v getfacl || true
```

### Observed result
```text
# no output for either command
```

Result:
- `setfacl`/`getfacl` are unavailable in this environment, so ACL round-trip could not be proven here with operator tooling.
- This is an environment limitation, not a claim of successful ACL restoration proof.

---

## 2) SELinux label validation

### Setup + pack/extract commands
```bash
BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres04_selinux
rm -rf "$WORK"
mkdir -p "$WORK/src" "$WORK/out_root" "$WORK/out_nobody"

printf '#!/bin/sh\nexit 0\n' > "$WORK/src/tool.sh"
chmod 755 "$WORK/src/tool.sh"
setcap cap_net_bind_service=+ep "$WORK/src/tool.sh"

python3 - <<'PY'
import os
p='/tmp/crushr_pres04_selinux/src/tool.sh'
os.setxattr(p,'security.selinux',b'unconfined_u:object_r:user_home_t:s0')
print(os.getxattr(p,'security.selinux').decode())
print(os.getxattr(p,'security.capability').hex())
PY

"$BIN/crushr-pack" "$WORK/src" -o "$WORK/pres04_sec.crs" --level 3
"$BIN/crushr-extract" "$WORK/pres04_sec.crs" -o "$WORK/out_root"
```

### Verification commands
```bash
ls -Z "$WORK/src/tool.sh" "$WORK/out_root/tool.sh"
python3 - <<'PY'
import os
p='/tmp/crushr_pres04_selinux/out_root/tool.sh'
print('selinux', os.getxattr(p,'security.selinux'))
print('cap', os.getxattr(p,'security.capability').hex())
PY
```

### Observed results
```text
unconfined_u:object_r:user_home_t:s0
0100000200040000000000000000000000000000

unconfined_u:object_r:user_home_t:s0 /tmp/crushr_pres04_selinux/out_root/tool.sh
unconfined_u:object_r:user_home_t:s0 /tmp/crushr_pres04_selinux/src/tool.sh

selinux b'unconfined_u:object_r:user_home_t:s0'
cap 0100000200040000000000000000000000000000
```

Result:
- SELinux label xattr was captured and restored in this environment.

---

## 3) Linux capability validation

### Commands
```bash
BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres04_manual
rm -rf "$WORK"
mkdir -p "$WORK/src" "$WORK/out_root" "$WORK/out_nobody"

printf '#!/bin/sh\nexit 0\n' > "$WORK/src/tool.sh"
chmod 755 "$WORK/src/tool.sh"
setcap cap_net_bind_service=+ep "$WORK/src/tool.sh"
getcap "$WORK/src/tool.sh"

"$BIN/crushr-pack" "$WORK/src" -o "$WORK/pres04_cap.crs" --level 3
"$BIN/crushr-extract" "$WORK/pres04_cap.crs" -o "$WORK/out_root"
getcap "$WORK/out_root/tool.sh"

# non-root degrade path
chown -R nobody:nogroup "$WORK/out_nobody"
su -s /bin/bash nobody -c "'$BIN/crushr-extract' '$WORK/pres04_cap.crs' -o '$WORK/out_nobody'" || true
```

### Observed results
```text
/tmp/crushr_pres04_manual/src/tool.sh cap_net_bind_service=ep
/tmp/crushr_pres04_manual/out_root/tool.sh cap_net_bind_service=ep

WARNING[capability-restore]: could not restore 'security.capability' on '/tmp/crushr_pres04_manual/out_nobody/tool.sh': Operation not permitted (os error 1)
WARNING[ownership-restore]: could not restore 'root:root' on '/tmp/crushr_pres04_manual/out_nobody/tool.sh': Operation not permitted (os error 1)
```

Result:
- Capability restores in privileged extraction context.
- In non-root context, explicit `WARNING[capability-restore]` is emitted and extraction continues.

---

## 4) Info visibility validation

### Command
```bash
/workspace/crushr/target/debug/crushr-info /tmp/crushr_pres04_selinux/pres04_sec.crs
```

### Relevant output snippet
```text
Metadata
  ...
  ACLs                   absent
  SELinux labels         present
  capabilities           present
```

Result:
- Metadata visibility truthfully reflects what is present in this fixture (`SELinux labels` + `capabilities` present; ACL absent due setup limits).

---

## 5) Non-root / degraded restore honesty check

### Command
```bash
BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres04_selinux
mkdir -p "$WORK/out_nobody"
chown -R nobody:nogroup "$WORK/out_nobody"
su -s /bin/bash nobody -c "'$BIN/crushr-extract' '$WORK/pres04_sec.crs' -o '$WORK/out_nobody'"
```

### Warning snippet
```text
WARNING[ownership-restore]: could not restore 'root:root' on '/tmp/crushr_pres04_selinux/out_nobody/tool.sh': Operation not permitted (os error 1)
WARNING[capability-restore]: could not restore 'security.capability' on '/tmp/crushr_pres04_selinux/out_nobody/tool.sh': Operation not permitted (os error 1)
```

### Extraction outcome
```text
Result
  safe files             1
  refused files          0
  status                 COMPLETE
  message                strict extraction completed
```

Result:
- Extraction continues and completes while surfacing explicit degradation warnings.
- No silent success claim for blocked capability/ownership restoration.

---

## 6) Backward-compatibility sanity check (pre-IDX6 + IDX6)

### Pre-IDX6 (IDX3) archive generation + checks
```bash
# generate a minimal IDX3/FTR4 archive
cargo run -q --manifest-path /tmp/idx3_gen/Cargo.toml -- /tmp/crushr_pres04_manual/legacy_idx3.crs

BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres04_manual
mkdir -p "$WORK/legacy_out"

"$BIN/crushr-info" /tmp/crushr_pres04_manual/legacy_idx3.crs
"$BIN/crushr-extract" /tmp/crushr_pres04_manual/legacy_idx3.crs -o "$WORK/legacy_out"
cat "$WORK/legacy_out/legacy/file.txt"
```

Observed:
```text
format markers         FTR4 + IDX3
...
legacy-idx3
```

### IDX6 archive checks
```bash
BIN=/workspace/crushr/target/debug
"$BIN/crushr-info" /tmp/crushr_pres04_selinux/pres04_sec.crs
"$BIN/crushr-extract" /tmp/crushr_pres04_selinux/pres04_sec.crs -o /tmp/crushr_pres04_manual/idx6_out
```

Observed:
```text
format markers         FTR4 + IDX6
...
Result ... status COMPLETE ... strict extraction completed
```

Result:
- Current tooling successfully handles both pre-IDX6 (`IDX3`) and IDX6 archives for `info` and strict extraction paths.

---

## 7) Recovery-mode classification check (CRUSHR_RECOVERY_MODEL_07 boundary)

### Command
```bash
BIN=/workspace/crushr/target/debug
WORK=/tmp/crushr_pres04_selinux
mkdir -p "$WORK/recover_nobody"
chown -R nobody:nogroup "$WORK/recover_nobody"
su -s /bin/bash nobody -c "'$BIN/crushr-extract' '$WORK/pres04_sec.crs' -o '$WORK/recover_nobody' --recover" || true
```

### Observed snippet
```text
Result
  canonical files        1
  named recovered        0
  anonymous recovered    0
  unrecoverable          0
  status                 COMPLETE
  message                recovery extraction completed

WARNING[ownership-restore]: could not restore 'root:root' ... Operation not permitted
WARNING[capability-restore]: could not restore 'security.capability' ... Operation not permitted
```

Answer to required question:
- **Current behavior**: entries remain treated as canonical when path/name/data integrity is intact, even if ACL/SELinux/capability restoration emits warnings.
- This is therefore currently **warned-only** for these metadata restoration failures and is a known trust-model semantic gap to be handled by **CRUSHR_RECOVERY_MODEL_07**.
