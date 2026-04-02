<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# crushr Phase Plan

This document defines the high-level development phases for crushr after the core archive architecture and preservation model were established.

The project is sequenced across three maturity bands:

- **0.x** — product proof, stabilization, preservation scope, introspection, benchmarking, and contract hardening
- **1.x** — stable preservation platform with locked contracts and workflow maturity
- **2.x** — evidence-grade extension layer (custody, signing, classification expansion)

## Current architectural identity

crushr is a deterministic archive system built around:

- explicit separation of payload integrity from metadata completeness
- deterministic validation and verification behavior
- explicit trust classification for non-canonical outcomes
- fail-closed strict extraction semantics
- bounded explicit recovery through `extract --recover`
- archive introspection without extraction
- Linux-first preservation fidelity layered onto an integrity-first model

crushr prioritizes data integrity, explicit truth, and bounded failure behavior over maximum compression ratio.

## Active versioning discipline (current truth)

- Canonical project version source: root `VERSION` file.
- Accepted version synchronization rule: `workspace.package.version` in root `Cargo.toml` and user-facing runtime version output must match `VERSION`.
- Builder rule: do not invent ad hoc version bumps; apply only the locked project versioning policy and keep user-facing version aligned to accepted repo truth.
- Current accepted version baseline: `0.4.20`.

## Phase status

- [x] Phase 15 — Dictionary hardening and namespace factoring
- [x] Phase 16 — Benchmarking and compression evidence
- [ ] Documentation alignment pass — builder/control-doc and public-doc normalization
- [ ] Phase 17 — Introspection and truth-surface expansion
- [ ] Phase 18+ — subsequent roadmap phases as explicitly approved

## Documentation alignment pass (current)

### Goal

Bring `.ai/`, contract, and public documentation onto one canonical vocabulary and one coherent product surface.

### Required outcomes

- no current product language uses `salvage` as a live operator-facing term
- no current product language uses `fsck` as a live command or mental model
- wrapper binaries are not documented as canonical product surface
- validation vs verification are defined consistently
- recovery and trust classes are defined consistently
- public docs and builder-facing `.ai/` docs do not contradict each other

### Guardrails

- do not rewrite historical chronology as if it never happened
- do not invent new command surfaces
- do not broaden public claims beyond current product behavior
- do not let builder-facing docs drift into obsolete vocabulary

## Phase 17 — Introspection and truth surfaces (next after alignment)

### Goal

Expand archive introspection so container truth, entry truth, and structural visibility become richer without weakening fail-closed semantics.

### Focus areas

- deeper archive/container introspection
- richer `info --list` entry attributes and trust context
- improved archive-layout visibility
- explicit truth-surface reporting without extraction

### Guardrails

- introspection is explanatory, not repair behavior
- recovery remains explicit and bounded
- payload integrity and metadata completeness remain distinct

## 2026-03-30 — Active Step Update (P17S01f0)

- Completed: Phase 17 Step 01 fix 0 (`P17S01f0`).
- Implemented archive-level truth summary expansion for `crushr info` (human + JSON), including deterministic structural summary, verification summary, and explicit `strict_extraction_supported`.
- Removed wrapper binaries from `crushr` build targets and source files; command access is now via `crushr` subcommands.
- Updated tests and schema contracts to match the single-binary command surface and new `crushr info` JSON truth shape.
- Next: Phase 17 follow-on introspection packets (entry-level/propagation work remains out of scope for this step).


## 2026-03-30 — Active Step Update (P17S02f0)

- Completed: Phase 17 Step 02 fix 0 (`P17S02f0`).
- Added entry-level introspection commands:
  - `crushr info --entry <logical/path>` (exact path lookup; human + JSON)
  - `crushr info --find <query>` (deterministic substring search; human + JSON)
- Reserved forward-compatible search CLI shape:
  - `--find-mode substring` (currently enforced)
  - `--find-limit <n>`
- Added deterministic not-found and zero-match behavior for `--entry`/`--find`.
- Updated tests and docs for the expanded `info` introspection surface.
- Next: follow-on Phase 17 introspection packets (propagation/impact extensions remain separate scope).

## 2026-03-30 — Active Step Update (P17S02f1)

- Completed: Phase 17 Step 02 fix 1 (`P17S02f1`).
- Enriched `crushr info --entry` with additional deterministic proof/structure detail in both human and JSON outputs:
  - `payload_blake3`
  - `logical_range.start` / `logical_range.end`
  - `identity_source`
- Added human-readable entry rows for payload BLAKE3, logical range (`hex + decimal`), and identity source.
- Kept trust semantics unchanged (no new trust classes, no extraction behavior change, no archive-physical offsets exposed).
- Updated CLI presentation tests and guide documentation for the expanded `--entry` truth surface.
- Next: Phase 17 follow-on introspection packets outside this bounded step.


## 2026-03-30 — Active Step Update (P17S03f0)

- Completed: Phase 17 Step 03 fix 0 (`P17S03f0`).
- Hardened `crushr info --report propagation` into an explanatory truth surface that explicitly separates:
  - deterministic dependency graph (`nodes`, `edges`)
  - detected corruption inputs (`detected_corruption`)
  - currently activated impacts (`activated_impacts`)
  - per-entry dependency + impact classification support (`entry_impacts`)
- Added bounded, stable dependency reasons and direct/propagated entry dependency semantics.
- Added deterministic entry-level canonical blocking + trust-class support explanation aligned to existing trust classes.
- Updated propagation schema/contract/tests and guide docs for the expanded report shape.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S03f1)

- Completed: Phase 17 Step 03 fix 1 (`P17S03f1`).
- Retired `crushr info --report propagation` and promoted the canonical propagation surface to:
  - `crushr info --propagation`
  - `crushr info --propagation --json`
- Implemented operator-facing default human presentation for propagation with bounded sections covering archive context, detected corruption, impact summary, activated impacts, and entry impacts.
- Preserved machine-readable propagation semantics and deterministic ordering in explicit JSON mode (`--propagation --json`).
- Updated CLI tests, propagation contract docs, and guide docs to reflect the renamed surface and default human mode.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S02f2)

- Completed: Phase 17 Step 02 fix 2 (`P17S02f2`).
- Added deterministic shell completion generation command:
  - `crushr completion bash`
  - `crushr completion zsh`
  - `crushr completion fish`
- Completion scripts are generated from clap and written to stdout only (no files or side effects).
- Included `info` introspection flags (`--list`, `--entry`, `--find`, `--propagation`) plus primary command surface coverage (`extract` / `verify` / `pack` / `about`).
- Added CLI tests for command existence, non-empty output, and expected completion token coverage; updated concise README installation examples.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S03f2)

- Completed: Phase 17 Step 03 fix 2 (`P17S03f2`).
- Improved `crushr info --propagation` human output readability for dependency-dense entries:
  - dependencies are rendered as multi-line rows in deterministic dependency order
  - first 4 dependencies are shown; remaining dependencies are summarized as `+ <n> more`
- Kept propagation JSON behavior unchanged (`--propagation --json` remains full machine-readable truth with unchanged semantics).
- Added deterministic coverage for human-mode dependency summarization and key operator-facing fields (reason, consequence, canonical blocked, trust-class support).
- Updated `docs/guide/info.md` to document the human-mode-only dependency summarization rule.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S02f3)

- Completed: Phase 17 Step 02 fix 3 (`P17S02f3`).
- Added deterministic clap-derived man page generation surface:
  - `crushr man`
  - `crushr man --out-dir <path>`
- Man pages now generate for root and canonical subcommands:
  - `crushr.1`
  - `crushr-info.1`
  - `crushr-extract.1`
  - `crushr-verify.1`
  - `crushr-pack.1`
  - `crushr-about.1`
  - `crushr-completion.1`
- Added CLI tests for command presence, generated-file existence/non-empty checks, and deterministic repeated output.
- Updated README with concise man-page generation usage examples.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S02f4)

- Completed: Phase 17 Step 02 fix 4 (`P17S02f4`).
- Aligned README command-surface/product-boundary wording to the current canonical CLI surface.
- Added explicit README coverage for `completion`, `man`, and `info --propagation` in concise command summaries without expanding into full reference material.
- Removed command-surface drift in README by using one consistent canonical command list and short behavior-accurate descriptions.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-30 — Active Step Update (P17S02f5)

- Completed: Phase 17 Step 02 fix 5 (`P17S02f5`).
- Restored `crushr about` to the locked presentation contract:
  - italicized quoted tagline
  - sections limited to Build / Behavior / Built with / Source
  - removed Data Model, Support, Notices, and all `salvage` references
  - Behavior now uses `recover` with explicit bounded non-canonical wording.
- Corrected root help presentation so canonical commands are `pack`, `extract`, `verify`, `info`, `about`, `completion`, and `man`, while bounded non-primary commands include only `lab`.
- Hardened `crushr info --propagation` human mode to map internal identifiers/reason tokens to operator-safe language (for example archive footer / tail frame / index, and `requires index`) while keeping JSON output unchanged.
- Added/updated tests to lock about/help constraints and propagation human-output abstraction boundaries.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-31 — Active Step Update (P17S03f3)

- Completed: Phase 17 Step 03 fix 3 (`P17S03f3`).
- Rebuilt `crushr info --entry` archive-input gate and argument-order correctness without changing command surface:
  - both forms are supported deterministically:
    - `crushr info <archive> --entry <path>`
    - `crushr info --entry <path> <archive>`
  - archive path is now validated pre-open via symlink-following metadata and rejects non-regular paths with:
    - `archive path is not a regular file`
- Added deterministic test coverage for:
  - both argument orders
  - malformed usage (`missing archive`, `missing value for --entry`)
  - symlink-to-regular-file acceptance
  - non-regular path rejection (directory/FIFO) with bounded timeout check to guard against hangs
  - version baseline lock through existing `version_contract` run
- Baseline version remains aligned to accepted project state: `0.4.20`.
- Next: follow-on Phase 17 introspection packets outside this bounded step.

## 2026-03-31 — Active Step Update (P17S02f6)

- Completed: Phase 17 Step 02 fix 6 (`P17S02f6`).
- Removed legacy runtime command surface `crushr salvage` from command parsing/dispatch and removed the in-tree salvage command implementation module.
- Deleted salvage-specific test/golden coverage that depended on the removed runtime command and replaced coverage with explicit command-rejection + absence assertions for help/completion/man surfaces.
- Updated active reference docs to remove the salvage-model page from the canonical reference index and replaced it with a recovery-model page aligned to `extract --recover`.
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Next: continue Phase 17 follow-on packets; preserve `lab` as the only bounded non-primary command in root help.

## 2026-03-31 — Active Step Update (P17S02f7)

- Completed: Phase 17 Step 02 fix 7 (`P17S02f7`).
- Audited README command surface and confirmed canonical command list is aligned to current CLI (`pack`, `extract`, `verify`, `info`, `about`, `completion`, `man`) with `lab` bounded non-primary.
- Added explicit versioning discipline to active control truth:
  - `VERSION` is canonical
  - runtime/Cargo version surfaces must match accepted repo truth
  - no ad hoc version bumps by builders
- Validation:
  - `cargo test -p crushr --test version_contract`
  - `./scripts/check-version-sync.sh`
- Next: final Phase 17 closure packet sequencing as directed by planner/user.

## 2026-04-01 — Active Step Update (P18S01f0)

- Completed: Phase 18 Step 01 fix 0 (`P18S01f0`).
- Extracted read-only introspection logic into shared `crates/crushr/src/introspection.rs` module with deterministic structured APIs:
  - `inspect_archive(path, product_version)`
  - `inspect_entry(path, entry_path)`
  - `find_entries(path, query, limit)`
  - `analyze_propagation(path)`
- Refactored `crushr info` command to consume the shared introspection module for archive summary JSON, entry lookup, find search, and propagation analysis while preserving existing CLI/human and JSON output behavior.
- Kept ordering deterministic (entry/find sorting and propagation behavior unchanged).
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Next: evaluate whether to promote the shared module into `crushr-core` in a follow-on packet once index-codec dependency boundaries are explicitly approved.

## 2026-04-01 — Active Step Update (P18S02f0)

- Completed: Phase 18 Step 02 fix 0 (`P18S02f0`).
- Added first browser/WASM read-only introspection demo at `demos/wasm-readonly-demo` with:
  - local `.crs` file load in browser
  - archive summary rendering
  - deterministic substring entry search
  - selected entry detail rendering
- Kept parser/introspection semantics in Rust by reusing shared introspection/core modules from `crates/crushr/src/*` in the WASM adapter crate (no JS archive parsing).
- Added shared byte-slice introspection adapters in `crates/crushr/src/introspection.rs` so browser-loaded bytes can invoke the same introspection logic.
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Next: optional drag-drop UX and bounded browser-level integration tests for demo flow.

## 2026-04-01 — Active Step Update (P18S02f1)

- Completed: Phase 18 Step 02 fix 1 (`P18S02f1`).
- Added drag-and-drop archive loading to `demos/wasm-readonly-demo/web` with a unified browser-side `loadArchive(file)` path used by both file picker and drop events.
- Added lightweight drag-over visual affordance (`drop-zone` highlight) and explicit invalid-input handling (`No file provided.`) while preserving existing loaded state on load failures.
- Kept read-only semantics unchanged (no extraction/write/mutation actions added).
- Validation:
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Next: optional browser-level automated UI checks for picker/drop parity.

## 2026-04-01 — Active Step Update (P18S03f0)

- Completed: Phase 18 Step 03 fix 0 (`P18S03f0`).
- Extended shared introspection entry report data with deterministic extent segment records for visualization (`extent_index`, `block_id`, `logical_start`, `logical_end`, `size_bytes`).
- Added WASM demo entry extent visualization panel and deterministic selected-result highlight behavior:
  - selecting a search result updates entry detail + visualization together
  - selecting another result cleanly replaces highlighted item and rendered segments
  - explicit empty and fallback messages for no-selection/unavailable-segment states
- Kept the demo read-only; no extraction/write/recovery actions were added.
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Next: optional bounded browser automation and visual polish follow-ups if requested.

## 2026-04-01 — Active Step Update (P18S04f0)

- Completed: Phase 18 Step 04 fix 0 (`P18S04f0`).
- Added WASM propagation visualization wiring via shared Rust introspection (`analyze_propagation_bytes`) with no JS reimplementation of propagation logic.
- Added operator-facing propagation panel + toggle in the demo UI:
  - deterministic impacted-entry summary
  - selected-entry impact detail (reason, consequence, canonical blocked state, trust-class support, relevant structures)
  - explicit no-impact messaging when no impacts are detected
- Integrated propagation-aware highlighting into existing demo surfaces:
  - impacted entries labeled in search results when propagation view is enabled
  - extent blocks switch between normal and impacted visual state for selected entries under propagation mode
  - added deterministic legend to distinguish selected / impacted / normal encoding
- Kept demo read-only and bounded to visualization (no extraction/write/mutation/recovery actions added).
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Next: optional bounded browser-level screenshot/automation pass if a browser artifact pipeline is available.


## 2026-04-01 — Active Step Update (P18S04f1)

- Completed: Phase 18 Step 04 fix 1 (`P18S04f1`).
- Performed bounded WASM demo hygiene cleanup focused on warning reduction and deterministic UI state coherence:
  - suppressed non-actionable dead-code warnings in the WASM adapter crate for imported shared modules (`format`, `index_codec`, `extraction_payload_core`, `introspection`) used as read-only source-inclusion dependencies
  - added explicit empty/error/no-selection/no-results messaging so search, entry detail, extent, and propagation panes do not conflict in idle/error/no-impact states
  - added deterministic reset behavior on new archive load and invalid archive attempts to prevent stale summary/result/entry/extent/impact state carry-over
- Kept scope bounded to cleanup only (no new feature controls, no propagation semantics changes, no extraction/recovery behavior additions).
- Validation:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Remaining warnings in WASM demo path: none observed in current `cargo check` output for the demo target.
- Next: optional bounded browser screenshot/automation pass when browser artifact tooling is available.

## 2026-04-01 — Active Step Update (P18S04f2)

- Completed: Phase 18 Step 04 fix 2 (`P18S04f2`).
- Performed bounded presentation-polish pass for `demos/wasm-readonly-demo/web` without feature expansion:
  - strengthened visual hierarchy using panelized section styling, stronger typographic grouping, and clearer search-row layout
  - improved selected/impacted/normal state clarity for search result rows and retained deterministic propagation-driven highlighting
  - refined extent presentation with clearer header/meta hierarchy and improved legend/readability treatment
  - improved empty-state intentionality (`results` no-data/no-match blocks, coherent first-load messaging surfaces)
- Updated demo README checks to include explicit visual-coherence verification points for showcase readiness.
- Kept read-only behavior and semantics unchanged (no archive semantics, extraction/recovery behavior, or propagation-meaning changes).
- Validation:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Screenshot asset status: browser screenshot tooling is not available in this environment; capture was not performed in this packet.
- Next: optional screenshot/asset capture packet when browser artifact tooling is available.

## 2026-04-01 — Active Step Update (P18S05f0)

- Completed: Phase 18 Step 05 fix 0 (`P18S05f0`).
- Promoted `demos/wasm-readonly-demo` to a static-host-ready artifact flow:
  - added deterministic `./build-dist.sh` staging command
  - emits deployable `dist/` with `index.html`, `main.js`, `styles.css`, `pkg/`, and `.nojekyll`
- Hardened static-host path compatibility by switching web module import to relative `./pkg/...` resolution (avoids parent-path breakage under subpath/static hosting).
- Applied minimal public-entry polish in `web/index.html` (`crushr — archive introspection demo` title/header, concise usage hint).
- Updated demo README with concise build/local-serve/deploy instructions including GitHub Pages-compatible publish notes.
- Validation:
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
  - `cd demos/wasm-readonly-demo/dist && python3 -m http.server 8080`
- Next: optional CI automation for publishing `dist/` (out of scope for this packet).

## 2026-04-01 — Active Step Update (P18S05f1)

- Completed: Phase 18 Step 05 fix 1 (`P18S05f1`).
- Hardened static build execution reliability by updating `demos/wasm-readonly-demo/build-dist.sh` to invoke:
  - `wasm-pack build --target web --no-opt --out-dir dist/pkg`
- Rationale: avoids environment-dependent Binaryen download failures while preserving deterministic static bundle output for this demo packet.
- Executed `./build-dist.sh` successfully after installing `wasm-pack`.
- Verified `dist/` static contents and local static serving from `dist/`.
- Verified WASM demo functional paths against a generated sample archive via JS/WASM invocation from the built `dist/pkg` exports:
  - archive load
  - summary
  - search
  - entry detail
  - extent visualization data path (`extent_segments`)
  - propagation toggle data path (`propagation`)
- Validation:
  - `cargo install wasm-pack`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
  - `cd demos/wasm-readonly-demo && find dist -mindepth 1 -maxdepth 2`
  - `cd demos/wasm-readonly-demo/dist && python3 -m http.server 8080` (+ HTTP 200 checks for `index.html`, `main.js`, and wasm asset)
  - Node ESM verification script against `dist/pkg` exports + `/tmp/crushr_demo_sample.crs`
- Next: optional browser-automation smoke (out of scope here due blocked browser download in this environment).

## 2026-04-01 — Active Step Update (P18S05f2)

- Completed: Phase 18 Step 05 fix 2 (`P18S05f2`).
- Fixed browser-runtime WASM initialization/load regression path in `demos/wasm-readonly-demo/web/main.js`:
  - switched from static top-level wasm import to resilient runtime initialization with explicit fallback module paths (`./pkg/...` then `../pkg/...`)
  - added explicit visible init-failure messaging (`Failed to initialize wasm runtime...`) to prevent silent no-op UI behavior
  - guarded load/search/propagation flows behind wasm-ready checks so picker/drop/search cannot fail silently when runtime init fails
- Preserved read-only semantics and shared Rust introspection usage (no JS archive parsing added).
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo install wasm-pack`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
- Constraint: browser screenshot/automation tooling is not available in this environment; runtime click-through verification remains manual in an external browser.
- Next: optional browser automation/screenshot packet once browser artifact tooling is available.

## 2026-04-01 — Active Step Update (P18S05f3)

- Completed: Phase 18 Step 05 fix 3 (`P18S05f3`).
- Implemented WASM demo UX/state-coherence updates for real-browser usability:
  - added explicit UI working-state banner with deterministic transitions (`idle`, `working`, `success`, `error`) during archive load, search, entry detail fetch, and propagation analysis
  - added explicit **Unload archive** control near file load and wired full deterministic reset of summary/search/results/selection/extent/propagation/error/status state
  - made entries/results pane browseable + scrollable and pre-populated it on load via deterministic full-list query
- Updated README with concise interaction-model notes for loading state, unload/reset behavior, and browseable results pane.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint: browser screenshot tooling remains unavailable in this environment.
- Next: optional browser automation/screenshot packet once browser artifact tooling is available.

## 2026-04-01 — Active Step Update (P18S06f0)

- Completed: Phase 18 Step 06 fix 0 (`P18S06f0`).
- Added repeatable introspection baseline harness for CLI + WASM demo:
  - `scripts/perf_introspection_baseline.py`
  - `scripts/perf_wasm_runner.mjs`
- Defined explicit reusable archive set classes and emitted manifest/results under `.bench/introspection_baseline/`:
  - `archive_set.json`
  - `cli_baseline.json`
  - `wasm_baseline.json`
- Produced baseline report at `docs/reference/introspection-baseline-p18s06.md` with comparison findings and next-step recommendation.
- Validation:
  - `cargo build --release -p crushr`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
  - `python3 scripts/perf_introspection_baseline.py --runs 1`
  - `node scripts/perf_wasm_runner.mjs --specs .bench/introspection_baseline/archive_set.json --runs 1 --out .bench/introspection_baseline/wasm_baseline.json`
- Constraint:
  - WASM timing baseline was captured via Node wasm-bindgen path; direct browser UI-blocking timings are still pending browser-session instrumentation.
- Next: follow-up packet to add browser-side performance marks/long-task instrumentation and validate UI responsiveness on the same archive set.

## 2026-04-01 — Active Step Update (P18S07f0)

- Completed: Phase 18 Step 07 fix 0 (`P18S07f0`).
- Added bounded hotspot characterization instrumentation for introspection `find`/`entry` in `crates/crushr/src/introspection.rs` with stage-level timing buckets:
  - `archive_open_read`
  - `index_decode_parse`
  - `summary_index_prep`
  - `traversal`
  - `result_materialization`
- Added reproducible hotspot harness/report flow:
  - `crates/crushr/examples/introspection_hotspot.rs`
  - `scripts/perf_introspection_hotspot.py`
  - report: `docs/reference/introspection-hotspot-p18s07.md`
  - artifacts: `.bench/introspection_hotspot/{archive_set_hotspot.json,cli_hotspot.json,wasm_hotspot.json}`
- Findings (measured): `summary_index_prep` dominates large/very_large `find` and `entry` in CLI; repeated-call medians stay close to cold calls; traversal/materialization are minor contributors; WASM wall time is higher than CLI for same archives.
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo build --release -p crushr`
  - `python3 scripts/perf_introspection_hotspot.py --runs 1`
- Constraint:
  - Direct browser render/main-thread breakdown is still not captured in this Node wasm-bindgen path.
- Next:
  - implement bounded decoded-index/entry-surface cache reuse packet for `find`/`entry` and re-run this harness for before/after evidence.

## 2026-04-01 — Active Step Update (P18S08f0)

- Completed: Phase 18 Step 08 fix 0 (`P18S08f0`).
- Added bounded reusable introspection state in `crates/crushr/src/introspection.rs`:
  - `ArchiveIntrospectionState` now owns decoded/derived entry surfaces and path lookup map.
  - path-based introspection uses a single-entry metadata-keyed cache (path + size + mtime seconds) to reuse state across repeated `find`/`entry` calls.
  - byte-based introspection exposes state-preparation + state-query helpers for explicit reuse in WASM.
- WASM demo Rust adapter now keeps loaded archive state in Rust-side session memory and reuses it for `find`/`entry` after `archive_summary`.
- Generated updated hotspot evidence:
  - `.bench/introspection_hotspot/{archive_set_hotspot.json,cli_hotspot.json,wasm_hotspot.json}`
  - `docs/reference/introspection-hotspot-p18s08.md`
- Validation:
  - `cargo fmt --all`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo build --release -p crushr`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `python3 scripts/perf_introspection_hotspot.py --runs 1`
- Constraint:
  - Existing hotspot script/report template still writes to `docs/reference/introspection-hotspot-p18s07.md`; step-specific interpretation is recorded in `...p18s08.md`.
- Next:
  - optional harness refinement to isolate truly-cold `entry` before any warm-up call in the same process.

## 2026-04-01 — Active Step Update (P18S08f1)

- Completed: Phase 18 Step 08 fix 1 (`P18S08f1`).
- Characterized initial-load hot path in the WASM demo and confirmed eager operations on load:
  - eager introspection-state construction in `archive_summary` (full entry-record/path map build)
  - eager empty-query `find` call from UI load flow
  - eager results-pane DOM rendering of the full browse list
- De-eagered initial archive load behavior:
  - `archive_summary` now stores loaded bytes and clears cached state; introspection state is prepared lazily on first `find`/`entry` request.
  - initial UI load no longer executes empty-query `find`; it renders an explicit “run Find to browse” prompt instead.
- Preserved reuse where it pays:
  - first search/detail call builds state once, then repeated `find`/`entry` reuse the same Rust-owned session state.
- Validation:
  - `cargo fmt --all`
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser screenshot/automation tooling is unavailable in this environment, so browser verification remains manual outside this runtime.
- Next:
  - if large non-empty queries still render too many rows, add explicit deterministic result capping + visible truncation messaging in a follow-on packet.

## 2026-04-01 — Active Step Update (P18S08f2)

- Completed: Phase 18 Step 08 fix 2 (`P18S08f2`).
- Fixed lazy-state `find` crash root path in WASM adapter:
  - removed full-archive byte cloning during lazy state build (`ensure_loaded_state`) and now borrows loaded bytes directly when preparing introspection state.
  - this avoids a second full in-memory copy during first `find`/`entry`, which could trigger runtime failure on very large archives.
  - UI now calls `find`/`entry`/`propagation` with an empty byte argument and relies on Rust-owned loaded-session bytes, avoiding repeated wasm-bindgen transfer/allocation of full archive bytes on each interaction.
- Added explicit Rust-side session reset hook (`reset_loaded_archive`) and wired UI reset/unload through that hook to clear Rust loaded bytes/state deterministically.
- Added explicit browser-visible WASM error messaging:
  - action-scoped UI errors now render as `"<action> failed: <detail>"`.
  - raw `RuntimeError: unreachable executed` is surfaced with explicit operator guidance to reload/retry, instead of an uncontextualized exception.
- Preserved P18S08f1 de-eager behavior:
  - archive load still shows summary without eager browse prepopulation
  - results still require explicit `Find`
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser tooling is not installed in this environment (`playwright` missing), so real-browser click-through verification remains required externally.

## 2026-04-01 — Active Step Update (P18S08f3)

- Completed: Phase 18 Step 08 fix 3 (`P18S08f3`).
- Stabilized WASM large-archive search path:
  - added bounded find result model in shared introspection (`BoundedFindResult`) with deterministic ordering, explicit `total_matches`, and `truncated` signal.
  - demo WASM adapter now caps browser find responses to `MAX_FIND_RESULTS = 500` and returns bounded metadata.
- Hardened browser UX for bounded behavior:
  - result pane now shows explicit truncation messaging (`Showing first N of M matches...`) when limit is hit.
  - search/detail/propagation continue to use Rust-owned loaded-session bytes with no eager pre-browse restore.
- Regression guard added:
  - unit test locks bounded-find determinism (`total_matches`, `truncated`, sorted first-N paths).
- Validation:
  - `cargo fmt --all`
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cargo test -p crushr introspection::tests::bounded_find_reports_total_and_truncation_deterministically`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
- Constraint:
  - this environment still lacks browser automation tooling (`playwright` missing), so real-browser verification must be executed externally.

## 2026-04-02 — Active Step Update (P18S08f4)

- Completed: Phase 18 Step 08 fix 4 (`P18S08f4`).
- Added persistent visible UI note in the search panel stating the 500 result cap and explicit query-refinement requirement to access deeper matches.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser automation remains unavailable in this environment (`playwright` missing), so real-browser visual verification remains external.


## 2026-04-02 — Active Step Update (P18S08f5)

- Completed: Phase 18 Step 08 fix 5 (`P18S08f5`).
- Implemented browser-safe worker offload for the WASM demo heavy-path operations:
  - added module worker runtime (`web/wasm-worker.js`) that hosts wasm init and all heavy calls (`archive_summary`, state prep, `find`, `entry`, `propagation`).
  - main thread now uses request/response message passing only; heavy introspection no longer executes on the UI thread.
- Added deterministic staged progress signaling across worker boundaries:
  - load path: `Loading archive...` → `Preparing archive...` → `Ready`
  - search path: `Searching...` → `Rendering results...` → `Ready`
- Hardened lifecycle/state boundaries:
  - worker reset path is now explicit (`reset`) and used for unload/new-load to avoid cross-archive state contamination.
  - added `prepare_loaded_archive_state()` wasm export so state-prep is explicit and worker-driven.
- Preserved bounded behavior and semantics:
  - existing bounded find contract (`max 500`, deterministic order, truncation metadata) remains unchanged under worker execution.
  - worker error messages are surfaced to UI as explicit action errors instead of raw runtime panic leakage.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - real-browser and screenshot tooling remains unavailable in this environment; packet-level interactive browser verification remains required externally.

## 2026-04-02 — Active Step Update (P18S08f6)

- Completed: Phase 18 Step 08 fix 6 (`P18S08f6`).
- Improved worker-backed WASM demo operation-state clarity for long-running archive interactions:
  - archive-load staged status now reports deterministic worker phases:
    - `Reading archive...`
    - `Inspecting archive summary...`
    - `Preparing archive state...`
    - `Ready`
  - worker status stage identifiers were tightened to explicit operation-scoped values (`load_*`, `search_busy`) for deterministic UI interpretation.
- Added explicit search busy indication in the UI search controls:
  - search button now switches label to `Searching...` while find is active
  - inline busy chip (`Searching…`) appears adjacent to search controls
  - search busy state is cleared on success and on worker error/reset boundaries
- Updated demo README interaction-model notes to reflect the new archive-load staging and search-control busy indication.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraints/gotchas:
  - `PROJECT_STATE.md` is referenced by bootstrap docs but is currently absent in this repository root.
  - Browser automation/screenshot tooling remains unavailable in this environment, so packet-level real-browser verification must still be executed externally.
- Next:
  - run explicit real-browser verification checklist for staged load progress, search busy-state, reset coherence, and failure-state transitions in an external browser session.

## 2026-04-02 — Active Step Update (P18S08f6 rework)

- Corrected P18S08f6 sequencing and UI presentation after review feedback.
- Deferred execution restored in worker flow:
  - archive load now stops after summary (`Reading archive...` → `Inspecting archive summary...` → `Ready`)
  - search lazily prepares state only when needed (`Preparing search state...` before first search)
  - entry detail lazily prepares state when needed (`Preparing entry state...` before first entry fetch)
- Replaced thin inline status banner with an unmistakable progress overlay layer (centered spinner + stage text) that appears during working operations and clears on success/error/reset.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser automation/screenshot tooling remains unavailable in this environment; required real-browser verification for this packet must be completed externally.
- Next:
  - run explicit real-browser validation for deferred-load behavior and overlay lifecycle across success/error/reset/switch-archive paths.

## 2026-04-02 — Active Step Update (P18S08f6 overlay-hidden fix)

- Fixed progress overlay visibility bug where CSS could override HTML `hidden` attribute on initial render.
- Added explicit `.progress-overlay[hidden] { display: none; }` guard so overlay appears only during active worker operations.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser automation/screenshot tooling remains unavailable in this environment; real-browser visual verification remains external.

## 2026-04-02 — Active Step Update (P18S08f6 propagation-overlay-clear fix)

- Fixed propagation-toggle no-impact path leaving the progress overlay active after internal refresh search/entry worker calls.
- Added explicit post-refresh settlement (`setUiState("success", "Ready")`) in the propagation-toggle handler so overlay state is cleared deterministically after refresh requests complete.
- Validation:
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
- Constraint:
  - browser automation/screenshot tooling remains unavailable in this environment; real-browser verification remains external.

## 2026-04-02 — Active Step Update (P18S08f7)

- Completed: Phase 18 Step 08 fix 7 (`P18S08f7`).
- Characterized initial WASM summary-load path and confirmed, with code evidence, that load summary currently performs:
  - full IDX3 decode/parse
  - block payload verification scan
  - but **does not** perform deferred `find`/`entry` state prep/path index build/per-entry materialization.
- Reduced initial summary-load overhead in WASM adapter by removing a second full archive-byte clone in `archive_summary`:
  - changed wasm export to take owned `Vec<u8>` and reuse it for both summary computation and loaded-session storage.
  - preserved deferred execution boundaries (`load => summary only`, lazy state prep on first `find`/`entry`).
- Added concise measurement report and artifacts:
  - `docs/reference/introspection-summary-load-p18s08f7.md`
  - `.bench/introspection_baseline/wasm_baseline_p18s08f7.json`
- Measured improvement on same baseline archive set (Node wasm-bindgen path, 1 run):
  - `large`: `21.103 ms` -> `13.576 ms` (~35.7% faster)
  - `very_large_stress`: `89.722 ms` -> `62.322 ms` (~30.5% faster)
- Validation:
  - `cargo fmt --all`
  - `node --check demos/wasm-readonly-demo/web/wasm-worker.js`
  - `node --check demos/wasm-readonly-demo/web/main.js`
  - `cargo check --manifest-path demos/wasm-readonly-demo/Cargo.toml --target wasm32-unknown-unknown`
  - `cargo test -p crushr --test cli_contract_surface --test cli_presentation_contract`
  - `cd demos/wasm-readonly-demo && ./build-dist.sh`
  - `node scripts/perf_wasm_runner.mjs --specs .bench/introspection_baseline/archive_set.json --runs 1 --out .bench/introspection_baseline/wasm_baseline_p18s08f7.json`
- Constraint:
  - Browser automation/screenshot tooling remains unavailable in this environment; required packet real-browser verification remains external.
