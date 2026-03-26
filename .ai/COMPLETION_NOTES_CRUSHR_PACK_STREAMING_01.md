<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# COMPLETION_NOTES — CRUSHR_PACK_STREAMING_01

## Root cause summary

Recurring OOM in production `pack` was caused by hidden whole-run payload retention in `emit_archive_from_layout`:

- `payload_materialized_by_block` cached `Vec<u8>` raw payload bytes per block.
- those cached raw bytes were kept until run end and scaled with archive-total payload size.
- this happened even though the same loop had already computed raw hashes and written compressed payloads.

## Architecture change summary

Bounded-memory behavior was restored by removing raw-byte residency from the hard-link reuse cache:

- hard-link reuse cache now keeps only `(raw_len, compressed_len, payload_hash, raw_hash, block_scan_offset)`.
- file-manifest digest (`file_digest`) now reuses the already computed per-block `raw_hash` instead of re-hashing retained raw bytes.
- mutation detection remains fail-closed and unchanged (`input changed during pack planning`).

## Repro commands and memory evidence

Dataset generation (deterministic):

```bash
python - <<'PY'
from pathlib import Path
root=Path('/tmp/crushr_pack_streaming_dataset')
if root.exists():
    import shutil; shutil.rmtree(root)
root.mkdir(parents=True)
for i in range(250):
    data=(f'file-{i:04d}-'.encode()* (2*1024*1024//10+1))[:2*1024*1024]
    (root/f'f{i:04d}.bin').write_bytes(data)
print(root)
PY
```

`HEAD~1` measurement (detached worktree build + run):

```bash
git worktree add --detach /tmp/crushr-prev-XXXXXX HEAD~1
cargo build -p crushr
python - <<'PY'
import resource, subprocess, time, pathlib
prev=pathlib.Path('/tmp/crushr-prev-z8CHoK')
cmd=[str(prev/'target/debug/crushr'),'pack','/tmp/crushr_pack_streaming_dataset','-o','/tmp/prev_pack.crs','--silent']
start=time.time()
proc=subprocess.run(cmd,cwd=str(prev),stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
usage=resource.getrusage(resource.RUSAGE_CHILDREN)
print(proc.stdout.strip())
print(f'prev_exit={proc.returncode}')
print(f'prev_maxrss_kib={usage.ru_maxrss}')
print(f'prev_elapsed_sec={time.time()-start:.3f}')
PY
```

Observed:

- `prev_exit=0`
- `prev_maxrss_kib=525800`

Current measurement:

```bash
python - <<'PY'
import resource, subprocess, time
cmd=['target/debug/crushr','pack','/tmp/crushr_pack_streaming_dataset','-o','/tmp/current_pack.crs','--silent']
start=time.time()
proc=subprocess.run(cmd,cwd='/workspace/crushr',stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
usage=resource.getrusage(resource.RUSAGE_CHILDREN)
print(proc.stdout.strip())
print(f'current_exit={proc.returncode}')
print(f'current_maxrss_kib={usage.ru_maxrss}')
print(f'current_elapsed_sec={time.time()-start:.3f}')
PY
```

Observed:

- `current_exit=0`
- `current_maxrss_kib=14400`

## Validation commands run

- `cargo fmt --all`
- `cargo test -p crushr pack_fails_if_file_changes_between_planning_and_emit -- --nocapture`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo test -p crushr --test version_contract`
- `target/debug/crushr info /tmp/current_pack.crs`
- `target/debug/crushr extract --verify /tmp/current_pack.crs`
- `target/debug/crushr extract /tmp/current_pack.crs -o /tmp/current_extract --all --silent`

## Remaining boundedness caveats

- index/tail emission still necessarily retains full `entries` metadata until final tailframe write; this scales with file-count/metadata, not payload bytes.
- optional experimental metadata profiles that emit cumulative checkpoints still retain checkpoint vectors by design for snapshot serialization.
