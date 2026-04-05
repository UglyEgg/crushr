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

## Semantics audit (P19S02f2)

Prior defaults were skewed toward structural kill-shots (`off0` overwrite/remove and truncate-at-0), which made the default demo path look like immediate container assassination.

- **Representative/acceptable:** seeded scattered flips and bounded non-zero offset damage.
- **Overly destructive by default:** overwrite/remove presets anchored at offset 0.
- **Misleading for resilience demo defaults:** truncation defaulting to offset 0.
- **Structural kill-shots:** truncate-at-0 and remove-from-start actions.

## Corruption preset groups

### Demo corruption presets (default group)

- bounded middle overwrite
- bounded payload overwrite
- bounded header damage (non-zero offset)

### Structural destruction / kill-shot modes

- scattered random damage (seeded deterministic flips; can still invalidate structure)
- kill-shot: tail structure damage
- kill-shot: truncate at offset 0
- kill-shot: remove from offset 0
- kill-shot: remove middle window

## Notes

- Processing is local-only in browser/WASM (no upload).
- Corruption is deterministic for the same selected mode + parameters.
- Download names include preset identity + mode marker + seed marker (`seed<value>` or `seedna`).
- Use the introspection demo to inspect the corrupted archive behavior.
