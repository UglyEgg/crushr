# crushr WASM read-only demo (P18S02)

Local browser demo for archive introspection.

## Build

```bash
cd demos/wasm-readonly-demo
wasm-pack build --target web --out-dir web/pkg
```

## Run

```bash
cd demos/wasm-readonly-demo/web
python3 -m http.server 8080
```

Open `http://127.0.0.1:8080` in a browser.

## Demo checks

1. Choose a local `.crs` file from the file picker **or drag/drop it into the drop zone**.
2. Confirm archive summary JSON appears.
3. Enter a substring query and click **Find**.
4. Click a result to view entry detail JSON and the extent visualization panel.
5. Select different results repeatedly; detail + extent view should update deterministically with no stale highlight.

Behavior is read-only: no extraction, write, or mutation actions exist in this demo UI.
