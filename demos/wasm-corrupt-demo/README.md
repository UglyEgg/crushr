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

- **Representative/acceptable:** small payload-focused corruption (`1B`/`256B`) that remains structurally inspectable.
- **Overly destructive by default:** overwrite/remove presets anchored at offset 0.
- **Misleading for resilience demo defaults:** truncation defaulting to offset 0.
- **Structural kill-shots:** truncate-at-0 and remove-from-start actions.

## Corruption preset groups

Preset naming/grouping now maps directly to the Phase 2 harness dimensions:

- **type:** `bit_flip`, `byte_overwrite`, `zero_fill`, `truncation`, `tail_damage`
- **target:** `header`, `index`, `payload`, `tail`
- **magnitude:** `1B`, `256B`, `4KB`

### Representative corruption presets (default group)

- payload bit flip (1B)
- payload byte overwrite (1B)
- payload zero-fill (256B)

### Structural stress presets

- index byte overwrite (1B)
- tail bit flip (1B)
- index zero-fill (256B)

### Catastrophic / kill-shot modes

- truncation at tail boundary (4KB)
- tail damage wipe (4KB)
- header destruction zero-fill (4KB)

## Notes

- Processing is local-only in browser/WASM (no upload).
- Corruption is deterministic for the same selected mode + parameters.
- Download names include preset identity + mode marker + seed marker (`seed<value>` or `seedna`).
- The **What this emulates** panel updates per preset with plain-language explanation of real-world intent and severity.
- Use the introspection demo to inspect the corrupted archive behavior.
