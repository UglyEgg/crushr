# .ai/CHANGELOG.md

## 2026-02-17 — Phase 0, Step 0.1 (migration)

- Created canonical repo root from prime scaffold.
- Imported `crushr` crate sources, docs, and dev tooling.
- Preserved legacy continuity documents under `.ai/imported_crushr/` and `docs/legacy/`.

## 2026-02-17 — Phase 0, Step 0.2 (spec + architecture lock-in)

- Replaced `SPEC.md` with Archive Format v1.0 (BLK3/DCT1/IDX3/FTR4).
- Preserved prior spec as `docs/legacy/SPEC_pre_v1.md`.
- Added `docs/ARCHITECTURE.md` (crate graph, tool suite, no-IPC rule).
- Converted repo to a Cargo workspace and introduced `crushr-format` and `crushr-core` crate skeletons.

## 2026-02-17 — Phase 0, Step 0.3 (TUI live/snapshot contract skeleton)

- Documented the TUI dual-mode data pipeline (live + snapshot) and merge rules in `docs/ARCHITECTURE.md`.
- Added normative snapshot contract: `docs/SNAPSHOT_FORMAT.md`.
- Added snapshot schema placeholders under `schemas/`.
- Added workspace skeleton crates: `crushr-cli-common` and `crushr-tui`.
- Added `crushr-core::snapshot` envelope types (v1 skeleton).

## 2026-02-17 — Phase 0, Step 0.4 (Ledger framing + snapshot fingerprint)

- Implemented `LDG1` ledger framing in `crushr-format` with canonical JSON serialization and BLAKE3 verification.
- Added unit tests for canonicalization and LDG1 round-trip.
- Introduced typed `ArchiveFingerprint` and deterministic derivation helper in `crushr-core::snapshot`.
- Added snapshot serialization tests.

## 2026-02-17 — Phase 0, Step 0.5 (BLK3 primitives)

- Added `crushr-format::blk3` defining BLK3 header layout, strict v1 validation, and read/write helpers.
- Enforced v1 rules: unknown flags rejected; dict flag/id consistency; reserved bytes must be zero.
- Added unit tests covering round-trips and invalid encodings.
- Updated `.ai/PHASE_PLAN.md` and `.ai/BACKLOG.md` with the near-future step plan and decision gates.

## 2026-02-17 — Phase 0, Step 0.6 (DCT1 primitives)

- Added `crushr-format::dct1` defining the DCT1 dictionary table layout.
- Implemented strict read/write with corruption guards (max count/size) and mandatory BLAKE3 dict hash verification.
- Added unit tests for multi-dict round-trips, duplicate dict_id rejection, and hash mismatch detection.

## 2026-02-17 — Phase 0, Step 0.7 (FTR4 primitives)

- Added `crushr-format::ftr4` defining the FTR4 footer layout.
- Implemented strict read/write with presence rules for optional sections (DCT/LEDGER), reserved-zero enforcement, overflow guards, and footer hash verification.
- Added unit tests for round-trip encoding/decoding, reserved-byte corruption, footer hash mismatch, and ledger presence invariants.

- p0s0.8f0: added contracts package, gated roadmap, and project-definition docs

- p0s0.9f0: added failure-domain validation phase and corruption harness skeleton

- p0s0.10f0: added formal failure-domain docs, impact schema, and decompression-free impact enumeration model

- p0s0.10f1: added Codex control-layer docs and first task packet


## 2026-03-08 — Phase 0, Step 0.8 (tail frame helpers)

- Added `crushr-format::tailframe` assembly helpers for deterministic `DCT1? + IDX3 + LDG1? + FTR4` layout.
- Added strict parsing helpers that validate footer/component boundaries and BLAKE3 integrity fields.
- Added round-trip tests (full + minimal) and malformed rejection tests (footer corruption and ledger corruption).
- Fixed a pre-existing syntax defect in `ftr4` tests to restore successful test/clippy runs for `crushr-format`.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 1)

- Repaired CLI source parse errors that prevented `cargo fmt --all` from running.
- Restored `Cmd::Info` to the command `match` in `main.rs`.
- Fixed `cli_ui` tier-2 sink wiring/type usage and corrected a path separator literal in `pack.rs`.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 2)

- Fixed `crushr/src/dict.rs` regressions that blocked `cargo check -p crushr`.
- Imported `walkdir::WalkDir`, restored valid function separation, and updated `zstd::dict::from_continuous` to pass sample sizes.
- Added explicit empty-sample guard for progress-based dictionary training.


## 2026-03-08 — Phase 0, Step 0.9 (fix iteration 3)

- Cleaned `crushr-format` tail frame parsing internals to remove unchecked narrowing casts and keep deterministic bounds checks for component slices.
- Removed duplicate LDG1 parsing pass in trailing-byte detection by validating exact reader consumption directly.
- Re-ran and passed `cargo test -p crushr-format` and `cargo clippy -p crushr-format --all-targets -- -D warnings`.

## 2026-03-08 — Phase 0, Step 0.10 (real info snapshot emission)

- Added `crushr-core::open::open_archive_v1` and real tail-frame archive parsing over `ReadAt + Len`.
- Replaced `InfoSnapshotV1` skeleton fields with typed summary/tail/dict metadata and added mapping from `OpenArchiveV1`.
- Added deterministic snapshot JSON serialization helper + tests for minimal archives, DCT1/LDG1 presence, deterministic serialization, and clean invalid-input failure.
- Added minimal `crushr-info` binary path supporting `crushr-info <archive> --json`.

## 2026-03-08 — Phase 0, Step 0.10 (fix iteration 1)

- Tightened `crushr-core` snapshot emission test assertions to validate typed JSON envelope/payload fields rather than substring presence.
- Re-ran Step 0.10 verification commands (`cargo test -p crushr-core`, `cargo test -p crushr --no-run`) to confirm read-only info snapshot path remains green.
- Documented that `crushr-info` end-to-end CLI JSON testing remains blocked on the `crushr` pack path still producing legacy (non-FTR4) archives.

## 2026-03-08 — Phase 0, Step 0.10 (fix iteration 2)

- Added explicit footer metadata to the open/snapshot mapping path (`footer_offset`, `footer_len`, `has_footer`) so `InfoSnapshotV1` emits footer presence/details from parsed archive state.
- Added `crushr-core` test coverage for the real `crushr-info --json` binary path using synthetic valid archive bytes written to a temp file.
- Re-ran `cargo fmt --all`, `cargo test -p crushr-core`, and `cargo test -p crushr --no-run`.

## 2026-03-08 — Phase 0, Step 0.12 (real fsck JSON metadata path)

- Added `crushr-fsck` binary with real `--json` output over opened archives and deterministic nonzero exit on structural parse/validation failure.
- Extended `crushr-core::snapshot` with typed fsck payload mapping and clean `ImpactReportV1` emission for currently supported metadata validation scope.
- Added synthetic-archive tests covering fsck valid success JSON, deterministic JSON, corrupted-footer failure, and corrupted-IDX3-hash failure.

## 2026-03-08 — Phase 0, Step 0.11 (minimal self-hosting v1 pack path)

- Added `crushr-pack` binary implementing the first bounded v1 writer path using `crushr-format` BLK3 and tailframe helpers.
- New pack path writes BLK3 blocks (one per file), IDX3 payload bytes, and a valid `FTR4` tail frame (without DCT1/LDG1 for now).
- Added integration tests covering single-file and tiny-directory pack flows with successful `open_archive_v1`, `crushr-info --json`, and `crushr-fsck --json` reads, plus deterministic output checks.
