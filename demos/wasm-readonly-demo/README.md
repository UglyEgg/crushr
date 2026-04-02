# crushr WASM read-only demo (P18S05)

Static, backend-free browser demo for archive introspection.

## Requirements

- `wasm-pack` (https://rustwasm.github.io/wasm-pack/installer/)
- Python 3 (for local static serving)

## Build deployable static bundle

```bash
cd demos/wasm-readonly-demo
./build-dist.sh
```

Output directory:

```text
dist/
  index.html
  main.js
  styles.css
  .nojekyll
  pkg/
    *.wasm
    *.js
    *.d.ts
    package.json
```

The `dist/` directory is static-host ready (no runtime build step, no backend).

## Run locally from static output

```bash
cd demos/wasm-readonly-demo/dist
python3 -m http.server 8080
```

Open `http://127.0.0.1:8080` in a browser.

## Deploy to static hosting

Any static host works (nginx, GitHub Pages, Netlify, S3 static hosting, etc.) as long as the full `dist/` contents are served.

### GitHub Pages (simple/manual path)

1. Build: `./build-dist.sh`
2. Publish `demos/wasm-readonly-demo/dist/` contents to your Pages source branch/folder.
3. Keep files at the root of the published folder (so `index.html` and `pkg/` are siblings).

`.nojekyll` is emitted for compatibility with Pages sites that do not use custom Jekyll config.

## Interaction model (P18S08f3)

- Heavy archive introspection executes inside a dedicated Web Worker; the main UI thread only handles rendering and message passing.
- Initial archive load is deferred to summary-only work with staged messages: `Reading archive...` → `Inspecting archive summary...` → `Ready` (no eager search-state prep on load).
- Long-running work is presented with a centered progress overlay layer (spinner + stage text), and search also keeps explicit local busy-state visibility near controls (`Searching…` indicator + disabled button state).
- Find now performs deferred stage prep when needed: `Preparing search state...` → `Searching...` → `Rendering results...` → `Ready`.
- The demo tracks explicit operation states (`idle`, `working`, `success`, `error`) and surfaces active work through the progress overlay stage text.
- Use **Unload archive** to fully reset archive summary, search query/results, selected entry, extent view, propagation state, and status/error surfaces to the empty first-load state.
- The entries/results pane does not pre-populate on archive load; the demo stays responsive and prompts the user to run **Find** explicitly.
- Find results are browser-bounded; when matches exceed the limit, the UI explicitly reports truncation (`Showing first N of M matches`) and asks the user to refine the query.
- Entry extent visualization now shows a deterministic data-vs-metadata split:
  - data segment = extent payload bytes
  - metadata segment = **derived** IDX extent-record overhead (`28 bytes` per extent)
- Entry content preview is bounded to first `5 KiB`:
  - UTF-8 preview renders as scrollable monospace text
  - non-UTF-8 payloads show deterministic binary classification (`appears to be`, `possibly ... (unverified)`, or `unknown`)

## Demo checks

1. Choose a local `.crs` file from the file picker **or drag/drop it into the drop zone**.
2. Confirm archive summary JSON appears.
3. Enter a substring query and click **Find**.
4. Click a result to view entry detail JSON and the extent visualization panel.
5. Enable **Show impact** to load propagation impact data.
6. Confirm impacted entries are labeled in search results and propagation detail appears for impacted selections.
7. Toggle **Show impact** off/on and confirm summary/detail/highlights update deterministically.
8. Select different results repeatedly; detail + extent view should update deterministically with no stale highlight.
9. Coherence checks:
   - invalid archive load clears prior summary/results/entry/extent state and shows an explicit error
   - empty search results render an explicit no-match message
   - propagation detail remains explicit for disabled, no-selection, and no-impact states
   - progress overlay appears during long operations and clears on both success and error
10. Runtime error check:
   - if wasm runtime initialization fails, an explicit error appears (no silent no-op UI)
11. Visual checks:
   - selected vs impacted vs normal result states are immediately distinguishable
   - extent legend and extent segment rows clearly map state to color (including data-vs-derived-metadata split)
   - first-load/no-data views look intentional, not empty placeholders
12. Preview checks:
   - select a text entry and confirm preview renders UTF-8 text in monospace with scroll
   - select a binary entry and confirm classification message appears
   - confirm preview never exceeds 5 KiB and clears on unload/reset

Behavior is read-only: no extraction, write, or mutation actions exist in this demo UI.
