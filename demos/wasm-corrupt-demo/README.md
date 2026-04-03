# crushr WASM corruption demo (P19S02)

Bounded browser demo that mutates local `.crs` bytes with deterministic corruption controls.

## Requirements

- `wasm-pack`
- Python 3 (for local static serving)

## Build

```bash
cd demos/wasm-corrupt-demo
./build-dist.sh
```

## Run locally

```bash
cd demos/wasm-corrupt-demo/dist
python3 -m http.server 8080
```

Open `http://127.0.0.1:8080`.

## Corruption modes

- random byte flip (seed + count)
- range overwrite (offset + length + byte value)
- truncation (cut at offset)
- block removal simulation (remove byte range)

## Notes

- Processing is local-only in browser/WASM (no upload).
- Corruption is deterministic for the same selected mode + parameters.
- Download names include `corrupted`, mode marker, and seed marker (`seed<value>` or `seedna`).
- Use the introspection demo to inspect the corrupted archive behavior.
