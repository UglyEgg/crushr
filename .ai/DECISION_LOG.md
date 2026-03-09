# .ai/DECISION_LOG.md

## 2026-02-17 — Canonical continuity policy source

- Decision: Use the prime scaffold `AGENTS.md` as canonical policy; preserve the original `crushr` `AGENTS.md` as legacy reference only.
- Alternatives:
  1. Replace scaffold `AGENTS.md` with the imported `crushr` `AGENTS.md`.
  2. Merge both into a single hybrid policy.
- Rationale: User instruction specifies `.tar.gz` directives as canonical; the scaffold is the `.tar.gz` source.
- Blast radius:
  - Affects how future instances interpret workflow, packaging, and handoff rules.
  - Imported policy references are now informational only.

## 2026-02-17 — Adopt core/format split and multi-tool suite

- Decision:
  - Introduce `crushr-format` (on-disk layouts) and `crushr-core` (engine over minimal IO traits), with `crushr` as the platform/integration crate.
  - Prefer a suite of focused CLI tools (pack/info/fsck/extract) over a monolithic CLI.
  - Enforce a **no-IPC** rule between tools (no JSON protocols, no sockets); all tools link crates and call APIs in-process.
- Alternatives:
  1. Single `crushr` crate + one CLI binary with many subcommands.
  2. Separate tools communicating via JSON/stdio or a daemon.
- Rationale:
  - Needed to support many knobs/features while keeping parsing logic centralized and enabling a "geek visibility" TUI.
  - In-process linking keeps cross-platform basics viable and avoids operational complexity.
- Blast radius:
  - Major repo restructure into a Cargo workspace.
  - Future development must respect crate boundaries to avoid duplicated parsing logic.

## 2026-02-17 — Freeze archive format v1.0 (BLK3/DCT1/IDX3/FTR4) and drop prototype compatibility

- Decision:
  - `SPEC.md` is now the v1.0 contract: BLK3 blocks, optional DCT1, IDX3 index, FTR4 footer.
  - Pre-v1.0 prototype archives are not guaranteed readable by v1.0 tools.
- Alternatives:
  1. Preserve backwards compatibility with BLK2/FTR2/older IDX variants.
  2. Provide a separate conversion tool only.
- Rationale:
  - The codebase was in a spec-drift state; freezing a single contract is required for a technically superior rewrite.
  - Compatibility can be added later as an explicit feature/phase if needed.
- Blast radius:
  - Existing prototype archives may become unreadable until/if a compatibility layer is implemented.

## 2026-02-17 — TUI supports live and snapshot modes

- Decision:
  - `crushr-tui` must support both **live mode** (open archive directly) and **snapshot mode** (load JSON outputs from `crushr-info --json` and `crushr-fsck --json`).
  - Snapshots are versioned and include an `archive_fingerprint`; snapshots with mismatched fingerprints must not be merged.
- Alternatives:
  1. Live mode only.
  2. Snapshot mode only.
  3. Ad-hoc, tool-specific JSON without a documented contract.
- Rationale:
  - Snapshot mode enables offline analysis, sharing, and deterministic regression tests without requiring access to the archive.
  - A documented contract prevents TUI/tool drift.
- Blast radius:
  - Introduces a stable JSON boundary (`docs/SNAPSHOT_FORMAT.md`) and schemas under `schemas/`.
  - TUI and tools must evolve snapshots in a versioned, backward-compatible way.


## 2026-03-08 — Recovery policy: detect and isolate only

- Decision: `fsck` detects and isolates corruption; it does not attempt reconstruction. Raw compressed blast-zone payload bytes may be dumped, and decompressed dumps are emitted only when verification passes.
- Rationale: preserves clarity, avoids ambiguous output, and keeps crushr out of the parity/reconstruction space.


## 2026-03-08 — Dictionary placement is per tail frame

- Decision: DCT1 is embedded per tail frame so each tail frame is self-contained for decode. Dictionary entries carry BLAKE3 hashes.
- Rationale: improves tail survivability without relying on external dictionary state.


## 2026-03-08 — TUI supports live and snapshot modes

- Decision: TUI is planned for both live archive access and versioned snapshot loading.
- Rationale: easier offline analysis, reproducible demos, and lower coupling.


## 2026-03-08 — Adaptive planning starts opt-in

- Decision: any auto-planning heuristics remain opt-in until tested and recorded in the ledger/results.
- Rationale: preserve determinism and avoid hidden behavior drift.

## 2026-03-08 — Adopt contracts, research scaffolding, and Codex control layer

- Decision: treat `docs/CONTRACTS/*`, `docs/RESEARCH/*`, `PROJECT_STATE.md`, `REPO_SNAPSHOT.md`, task packets, and review checklist as canonical project-control surfaces for implementation agents.
- Rationale: reduce drift, preserve the thesis, and keep Codex constrained to bounded tasks.

## 2026-03-08 — Normalize `crushr-info`/`crushr-fsck` open/parse failure exit codes

- Decision: For current workspace baseline, both `crushr-info` and `crushr-fsck` return exit code `2` for archive open failures and structural/parse/validation failures; usage/argument errors remain exit code `1`.
- Alternatives:
  1. Keep pre-existing inconsistency (`crushr-info` parse/open as `1`, `crushr-fsck` as `2`).
  2. Introduce a broader multi-code mapping now (including internal-failure `4`) across all tools in this pass.
- Rationale: This bounded hygiene pass required consistency for open/parse/structural failures without redesigning the full CLI error taxonomy.
- Blast radius:
  - Affects observed nonzero exit code behavior for `crushr-info` callers.
  - No format/snapshot/schema or research-claim semantics changed.
