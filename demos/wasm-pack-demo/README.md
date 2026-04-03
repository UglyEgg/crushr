# crushr WASM pack demo (P19S01)

Bounded browser demo that packs local files into a `.crs` archive with Rust/WASM logic.

## Requirements

- `wasm-pack`
- Python 3 (for local static serving)

## Demo limits (enforced)

- Max total input size: **256 MiB**
- Max file count: **1,000**
- Max single file size: **128 MiB**

## Build

```bash
cd demos/wasm-pack-demo
./build-dist.sh
```

## Run locally

```bash
cd demos/wasm-pack-demo/dist
python3 -m http.server 8080
```

Open `http://127.0.0.1:8080`.

## Use the demo

1. Select files/folder or drag/drop local files.
2. Confirm selection is within limits.
3. Click **Create .crs archive**.
4. Wait for staged status (`reading input...`, `preparing archive...`, `packing archive...`, `ready for download`).
5. Download the generated `.crs` file.

Notes:
- Processing is local-only in browser/WASM (no upload).
- Browser demo input is transient and bounded.
- For full preservation behavior and production workflows, use the CLI.
