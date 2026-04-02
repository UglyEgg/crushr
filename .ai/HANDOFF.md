<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/HANDOFF.md

## Fresh-instance takeover (current truth)

1. Read, in order:
   - `AGENTS.md`
   - `AI_BOOTSTRAP.md`
   - `REPO_GUARDRAILS.md`
   - `PROJECT_STATE.md`
   - `.ai/INDEX.md`
   - `.ai/STATUS.md`
   - `.ai/PHASE_PLAN.md`
   - `.ai/contracts/README.md`
   - `.ai/DECISION_LOG.md`
2. Treat `.ai/STATUS.md` as authoritative for current truth.
3. Treat `.ai/DECISION_LOG.md` and `.ai/CHANGELOG.md` as historical evidence, not active state.

## Where the repository stands

- Phase 15 and Phase 16 are complete.
- Current work is documentation/control-surface alignment before Phase 17 feature work.
- Public and builder-facing docs are being normalized to one vocabulary and one command surface.

## Canonical product surface

- `crushr pack`
- `crushr verify`
- `crushr info`
- `crushr info --list`
- `crushr info --entry`
- `crushr info --find`
- `crushr info --propagation`
- `crushr extract`
- `crushr extract --recover`
- `crushr about`
- `crushr completion <bash|zsh|fish>`
- `crushr man [--out-dir <path>]`

## Non-canonical surface

- `crushr lab` is internal development harness only.
- Wrapper binaries are not canonical product surface.
- `fsck` is not a live product concept.

## Builder guardrails

- Do not reintroduce `salvage` as current product vocabulary.
- Do not describe crushr as a repair or fixer tool.
- Do not collapse validation and verification into one concept.
- Do not document wrapper binaries as public command surface.
- Do not let historical notes override current canonical docs.

## Trust model you can rely on

- payload integrity is independent from metadata completeness
- strict extraction is fail-closed
- recovery is explicit through `extract --recover`
- trust-bearing non-canonical results are classified explicitly as:
  - `canonical`
  - `metadata_degraded`
  - `recovered_named`
  - `recovered_anonymous`
  - `unrecoverable`

## Evidence map

- current truth: `.ai/STATUS.md`
- sequencing: `.ai/PHASE_PLAN.md`
- non-negotiable boundaries: `.ai/contracts/README.md`
- historical rationale: `.ai/DECISION_LOG.md`
- chronology: `.ai/CHANGELOG.md`

## Latest completed step

- `P17S01f0` complete: `crushr info` truth-surface expansion + wrapper binary removal to single `crushr` binary access.

- `P17S02f0` complete: `crushr info --entry` + `crushr info --find` entry-level introspection expansion with deterministic human/JSON behavior.

- `P17S02f1` complete: `crushr info --entry` enriched with deterministic `payload_blake3`, logical range (`start`/`end`), and `identity_source` in both human and JSON output without trust-model drift.
- `P17S03f0` complete: `crushr info --report propagation` now provides explicit dependency, detected-corruption, activated-impact, and per-entry trust-class explanatory sections with deterministic bounded reasons.
- `P17S03f1` complete: propagation surface renamed to `crushr info --propagation`, with default operator-facing human output and explicit `--propagation --json` machine output while preserving deterministic propagation semantics.
- `P17S03f2` complete: propagation human output now renders dependency-dense entry impacts as deterministic multi-line rows with explicit remainder counts (`+ <n> more`) while JSON semantics remain unchanged.
- `P17S02f2` complete: added clap-generated shell completion output for `bash`/`zsh`/`fish` via `crushr completion <shell>` with stdout-only behavior and deterministic CLI token coverage.
- `P17S02f3` complete: added clap-derived man-page generation via `crushr man [--out-dir <path>]` for root + canonical subcommand pages with deterministic output and contract tests.
- `P17S02f4` complete: aligned README command-surface/product-boundary summaries to the canonical CLI, including `completion`, `man`, and `info --propagation` with concise behavior-accurate descriptions.
- `P17S02f5` complete: restored locked presentation contract for `about` and root `help`, removed `salvage` from user-facing command surfaces, and mapped propagation human output away from internal structure/reason identifiers while preserving JSON semantics.
- `P17S03f3` complete: rebuilt `info --entry` correctness to guarantee deterministic dual-order parsing, non-regular archive rejection without hangs (including FIFO), symlink-to-regular-file acceptance, and explicit malformed-usage errors.
- `P17S02f6` complete: removed legacy `crushr salvage` runtime dispatch + implementation modules, deleted salvage-root tests/golden artifacts, and locked CLI/tests/docs so salvage is rejected and absent from help/completion/man surfaces.
- `P17S02f7` complete: final Phase 17 alignment pass confirmed README command-surface accuracy and locked active versioning discipline language in control docs (`VERSION` canonical + runtime/Cargo sync, no ad hoc bumps).

## 2026-04-01 — Handoff update (P18S01f0 complete)

- Shared introspection logic for `info` surfaces is now centralized at `crates/crushr/src/introspection.rs`.
- `crates/crushr/src/commands/info.rs` now acts as parser/presentation adapter for:
  - archive summary JSON
  - `--entry`
  - `--find`
  - `--propagation`
- If next packet promotes this logic to `crushr-core`, resolve `decode_index` ownership first to avoid duplication or cross-crate leakage.
- Last validated commands:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`

## 2026-04-01 — Handoff update (P18S02f0 complete)

- New browser/WASM read-only demo is at `demos/wasm-readonly-demo`.
- Build + run:
  - `cd demos/wasm-readonly-demo && wasm-pack build --target web --out-dir web/pkg`
  - `cd demos/wasm-readonly-demo/web && python3 -m http.server 8080`
- Shared introspection now also supports byte-slice adapters (`inspect_archive_bytes`, `find_entries_bytes`, `inspect_entry_bytes`) so browser-loaded archive bytes use Rust truth semantics directly.
- Sparse-write helper is now explicitly non-Unix guarded in `extraction_payload_core` for cross-target compilation safety.

## 2026-04-01 — Handoff update (P18S02f1 complete)

- `demos/wasm-readonly-demo/web/main.js` now uses one shared `loadArchive(file)` function for both picker and drag/drop paths.
- `demos/wasm-readonly-demo/web/index.html` now includes an explicit drop zone prompt.
- `demos/wasm-readonly-demo/web/styles.css` adds a minimal drag-over highlight state (`.drop-zone.drag-over`).

## 2026-04-01 — Handoff update (P18S03f0 complete)

- Shared introspection entry reports now include `extent_segments` built from index extents:
  - `extent_index`
  - `block_id`
  - `logical_start`
  - `logical_end`
  - `size_bytes`
- WASM demo UI now includes an `Entry extents` panel that renders:
  - explicit empty state when no entry is selected
  - explicit fallback when extent segmentation is unavailable
  - deterministic segment strip + per-extent metadata rows for selected entries
- Search-result click behavior is now synchronized across panels:
  - selected list item highlight
  - entry detail JSON
  - extent visualization
- Last validated commands:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`

## 2026-04-01 — Handoff update (P18S04f0 complete)

- WASM demo now exposes propagation impact data through shared Rust introspection (`analyze_propagation_bytes`) and renders operator-facing impact language in the browser.
- UI additions in `demos/wasm-readonly-demo/web`:
  - `Show impact` toggle
  - propagation summary section with deterministic no-impact behavior
  - selected-entry propagation detail block (status, consequence, canonical blocked, trust-class support, reasons, relevant structures)
  - search-result impacted badges when propagation mode is enabled
  - extent panel color-state overlay for impacted vs normal selected entry, plus explicit legend
- WASM API additions in `demos/wasm-readonly-demo/src/lib.rs`:
  - `propagation(file_bytes)` returning mapped operator-facing impacted-entry records (no internal `structure:*` token leakage)
- Last validated commands:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`


## 2026-04-01 — Handoff update (P18S04f1 complete)

- WASM demo hygiene pass completed in `demos/wasm-readonly-demo` without expanding feature scope.
- Warning cleanup:
  - `demos/wasm-readonly-demo/src/lib.rs` now marks source-included shared modules with `#[allow(dead_code)]` to suppress non-actionable warnings from unused shared definitions in the demo adapter context.
- UI/state coherence updates in `web/main.js`:
  - explicit baseline messages for no-file, no-results, no-selection, propagation-disabled, and load-before-search paths
  - deterministic reset on new load and invalid archive attempt to avoid stale summary/results/entry/extent/impact state
  - deterministic no-match rendering for empty search results
- Minor style/readme cleanup:
  - `web/styles.css` adds muted results-row formatting
  - `README.md` documents coherence checks for invalid-load, no-results, and propagation detail state behavior
- Last validated commands:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`

## 2026-04-01 — Handoff update (P18S04f2 complete)

- WASM demo presentation polish completed for showcase readiness in:
  - `demos/wasm-readonly-demo/web/index.html`
  - `demos/wasm-readonly-demo/web/main.js`
  - `demos/wasm-readonly-demo/web/styles.css`
- UI polish highlights:
  - stronger panel hierarchy and calmer visual grouping (hero/panel treatment, search row cohesion)
  - clearer selected vs impacted vs normal result states without semantic/behavior changes
  - improved intentional empty-state rendering for first-load/no-results/no-selection flows
  - extent panel readability improvements (header/meta clarity + legend consistency)
- README demo checks updated with explicit visual-coherence verification bullets.
- Screenshot capture note:
  - browser screenshot tooling is unavailable in this environment, so no new image assets were generated in this packet.
- Last validated commands:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`

## 2026-04-01 — Handoff update (P18S05f0 complete)

- WASM demo now has a deterministic static-bundle build entrypoint:
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
- Build output is host-ready in `demos/wasm-readonly-demo/dist/` with:
  - `index.html`, `main.js`, `styles.css`, `pkg/`, `.nojekyll`
- Static hosting compatibility hardening:
  - `web/main.js` now imports `./pkg/crushr_wasm_readonly_demo.js` (subpath-safe on static hosts including GitHub Pages)
- Public-entry polish + docs:
  - `web/index.html` title/header/usage hint refined for shareable demo context
  - README now documents requirements, build, local serve, and static-host deploy workflow
- Last validated commands:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
  - `cd demos/wasm-readonly-demo/dist && python3 -m http.server 8080`

## 2026-04-01 — Handoff update (P18S05f1 complete)

- `demos/wasm-readonly-demo/build-dist.sh` now passes `--no-opt` to wasm-pack to avoid Binaryen download dependency during demo static bundle builds.
- Verified in this environment:
  - installed `wasm-pack`
  - ran `./build-dist.sh` successfully
  - confirmed `dist/` contains hostable static assets + wasm package outputs
  - served `dist/` via `python3 -m http.server` and confirmed HTTP 200 for key assets
  - exercised `archive_summary` / `find` / `entry` / `propagation` against a generated sample archive using `dist/pkg` JS/WASM exports (data paths backing summary/search/entry/extent/propagation).
- Browser automation note:
  - attempted Playwright browser install, but download is blocked (HTTP 403 Domain forbidden), so full browser-click automation could not be executed in this environment.

## 2026-04-01 — Handoff update (P18S05f2 complete)

- Browser runtime regression hardening landed in `demos/wasm-readonly-demo/web/main.js`:
  - WASM module now initializes through dynamic import fallback paths (`./pkg/...` then `../pkg/...`) instead of a single static import path.
  - Init failures are now surfaced explicitly in the UI (`Failed to initialize wasm runtime...`) instead of failing silently.
  - Archive load/search/propagation actions now require wasm-ready state, preventing no-op interactions when runtime init is broken.
- Validation executed:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
- Environment note:
  - browser screenshot/automation tool is unavailable here; complete picker/drop click-through verification should be run manually in an external browser session.

## 2026-04-01 — Handoff update (P18S05f3 complete)

- WASM demo interaction-model usability pass completed in:
  - `demos/wasm-readonly-demo/web/index.html`
  - `demos/wasm-readonly-demo/web/main.js`
  - `demos/wasm-readonly-demo/web/styles.css`
  - `demos/wasm-readonly-demo/README.md`
- Added deterministic UI state banner for working transitions (`idle`/`working`/`success`/`error`) and surfaced non-silent long operations (load/search/entry detail/propagation analysis).
- Added explicit **Unload archive** control with full reset of archive summary, query/results, selection/detail, extent panel, propagation state, and status/error surfaces.
- Results pane is now scrollable and browseable, and is pre-populated on successful archive load (deterministic empty-query listing).
- Last validated commands:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Screenshot note:
  - browser screenshot tooling remains unavailable in this environment.

## 2026-04-01 — Handoff update (P18S06f0 complete)

- Added repeatable baseline harness:
  - `scripts/perf_introspection_baseline.py`
  - `scripts/perf_wasm_runner.mjs`
- Baseline artifacts now generated under `.bench/introspection_baseline/`:
  - `archive_set.json`
  - `cli_baseline.json`
  - `wasm_baseline.json`
- Comparison report written to:
  - `docs/reference/introspection-baseline-p18s06.md`
- Environment note:
  - WASM baseline uses Node + wasm-bindgen execution path in this environment; direct browser responsiveness/UI-blocking measurements still require a browser session with explicit instrumentation.

## 2026-04-01 — Handoff update (P18S07f0 complete)

- Added stage-level hotspot instrumentation for introspection `find`/`entry` in `crates/crushr/src/introspection.rs` via profile surfaces used only by measurement harnesses.
- Added reproducible hotspot measurement harness:
  - `crates/crushr/examples/introspection_hotspot.rs`
  - `scripts/perf_introspection_hotspot.py`
- Generated hotspot artifacts + report:
  - `.bench/introspection_hotspot/{archive_set_hotspot.json,cli_hotspot.json,wasm_hotspot.json}`
  - `docs/reference/introspection-hotspot-p18s07.md`
- Key measured result: `summary_index_prep` dominates large/very_large `find` + `entry`; repeated calls still pay similar cost as cold calls; traversal/materialization are minor.
- WASM note: this packet compares WASM wall times vs CLI decomposition; fine-grained browser render/main-thread marks remain a follow-up if browser instrumentation is requested.

## 2026-04-01 — Handoff update (P18S08f0 complete)

- Shared introspection now has bounded reusable state in `crates/crushr/src/introspection.rs`:
  - `ArchiveIntrospectionState` (decoded/derived entry surfaces + path lookup map)
  - CLI path-based single-entry cache keyed by path + file metadata (size/mtime)
  - byte-side state-prep/query helpers for explicit reuse by WASM adapters
- WASM demo adapter (`demos/wasm-readonly-demo/src/lib.rs`) now stores loaded introspection state in Rust-side session memory and serves `find` / `entry` from that state after `archive_summary`.
- Updated hotspot outputs generated:
  - `.bench/introspection_hotspot/{archive_set_hotspot.json,cli_hotspot.json,wasm_hotspot.json}`
  - `docs/reference/introspection-hotspot-p18s08.md`
- Last validated commands:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo build --release -p crushr`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `python3 scripts/perf_introspection_hotspot.py --runs 1`

## 2026-04-01 — Handoff update (P18S08f1 complete)

- Initial WASM demo load was de-eagered to prevent large-archive UI hangs:
  - removed load-time empty-query `find` from `web/main.js` (results pane now shows an explicit search prompt after summary load)
  - moved WASM introspection-state construction out of `archive_summary`; state now builds lazily on first `find`/`entry` use in `src/lib.rs`
- Reuse behavior is preserved:
  - once first search/detail triggers state build, repeated `find`/`entry` calls reuse the same Rust session state.
- Characterization confirmed previous eager load path included:
  - state build at summary load
  - empty-query browse load
  - immediate full results DOM rendering
- Last validated commands:
  - `cargo fmt --all`
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`

## 2026-04-01 — Handoff update (P18S08f2 complete)

- Fixed WASM lazy-state crash path in `demos/wasm-readonly-demo/src/lib.rs`:
  - `ensure_loaded_state()` no longer clones `LOADED_BYTES` (full archive copy) before building introspection state.
  - state now builds directly from borrowed loaded bytes, then is cached in `LOADED_STATE`.
  - added `reset_loaded_archive()` to clear Rust-side loaded bytes/state on UI reset/unload.
- Improved explicit browser error surfacing in `demos/wasm-readonly-demo/web/main.js`:
  - operation-scoped error formatting (`<action> failed: <detail>`)
  - explicit internal WASM runtime guidance when browser reports `RuntimeError: unreachable ...`
  - removed generic load-failure overwrite so detailed runWorking error remains visible.
  - switched UI interaction calls (`find`/`entry`/`propagation`) to use an empty byte argument and rely on Rust-owned loaded-session bytes to avoid repeated large wasm argument transfers.
- De-eager behavior remains in place:
  - no empty-query pre-browse on load
  - explicit Find prompt after summary.
- Last validated commands:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Environment note:
  - `playwright` is not installed here, so real-browser click-through verification must be run outside this environment.

## 2026-04-01 — Handoff update (P18S08f3 complete)

- Large-archive browser search stabilization landed:
  - shared introspection now exposes bounded find metadata (`matches`, `total_matches`, `truncated`) in deterministic path order.
  - WASM adapter caps find responses at `MAX_FIND_RESULTS = 500`.
- UI now renders explicit truncation notice when bounded find limit is hit:
  - `Showing first N of M matches. Refine your search to narrow results.`
- De-eager load remains unchanged:
  - no load-time empty-query pre-browse; summary-first usable screen is preserved.
- Added regression test in shared introspection for bounded-find determinism/truncation accounting.
- Last validated commands:
  - `cargo fmt --all`
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo test -p crushr introspection::tests::bounded_find_reports_total_and_truncation_deterministically`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Environment note:
  - browser automation remains unavailable in this environment (`playwright` missing), so real-browser validation must be executed externally.

## 2026-04-02 — Handoff update (P18S08f4 complete)

- Added a persistent, always-visible note in the Search and browse panel (`web/index.html`) that states:
  - find is capped at 500 results
  - users must refine queries to access deeper matches
- Validation executed:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`


## 2026-04-02 — Handoff update (P18S08f5 complete)

- WASM demo heavy-path execution is now worker-hosted:
  - added `web/wasm-worker.js` module worker that owns wasm init + calls for load/state-prep/find/entry/propagation.
  - `web/main.js` now uses request/response message passing and UI-only rendering logic.
- Added staged progress transitions visible in UI:
  - load: `Loading archive...` → `Preparing archive...` → `Ready`
  - search: `Searching...` → `Rendering results...` → `Ready`
- Worker lifecycle/state guardrails:
  - explicit worker `reset` used for unload/new-load
  - added wasm export `prepare_loaded_archive_state()` for explicit archive state prep inside worker
- Static bundle script now copies worker asset into `dist/` (`build-dist.sh` copies `wasm-worker.js`).
- Last validated commands:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Environment note:
  - browser screenshot/automation tools are unavailable in this environment; real-browser packet verification must be executed externally.

## 2026-04-02 — Handoff update (P18S08f6 complete)

- Worker progress messaging is now more explicit for archive load in `demos/wasm-readonly-demo/web/wasm-worker.js`:
  - `Reading archive...`
  - `Inspecting archive summary...`
  - `Preparing archive state...`
  - `Ready`
- Search now has control-local busy visibility in `demos/wasm-readonly-demo/web`:
  - search button label switches to `Searching...`
  - inline busy indicator (`Searching…`) appears beside search controls while find is active
  - busy state clears on success/error/reset boundaries.
- README interaction-model notes updated for the new staged load/busy-state behavior.
- Last validated commands:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Environment note:
  - browser automation/screenshot tooling is still unavailable in this environment; real-browser verification remains an external/manual requirement.
