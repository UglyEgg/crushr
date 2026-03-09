# .ai/HANDOFF.md

Start with:
1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `PROJECT_STATE.md`
5. `REPO_SNAPSHOT.md`
6. `SPEC.md` and `docs/CONTRACTS/*`

Current shape:

- `crushr-extract` now supports explicit `--mode <strict|salvage>`; strict remains default/unchanged, while salvage adds deterministic `mode` + `salvage_decisions` JSON reporting without attempting reconstruction.
- `crushr-extract` now uses typed internal extraction outcome/error classification (success/partial refusal/usage/structural) for exit-code mapping and JSON error handling, removing message-string exit classification.
- `crushr-extract` now supports `--json` deterministic machine-readable strict extraction reports (`overall_status`, `extracted_files`, `refused_files` with `corrupted_required_blocks`, and `error` envelope on structural failures).
- `crushr-extract` now supports `--refusal-exit <success|partial-failure>` (default `success`) to control strict refusal exit semantics without changing extraction/refusal behavior.
- In `partial-failure` mode, strict extraction returns exit code `3` when one or more files are refused due to corrupted required blocks; structural/open failures remain exit `2`.
- New `crushr-extract` binary implements strict minimal-v1 extraction for regular files only using `open_archive_v1` + `scan_blocks_v1` + IDX3 decode + payload-hash verification.
- In strict mode, files whose required block IDs are corrupted are refused (not extracted), while unaffected files still extract; invalid tail/footer archives fail with exit code `2`.
- `crates/crushr-core/tests/minimal_pack_v1.rs` now covers clean extraction round trips, selective corruption refusal, invalid-footer failure, and deterministic refusal output ordering.
- `crushr-lab run-competitor-scaffold` now creates the first bounded comparison scaffold at `docs/RESEARCH/artifacts/crushr_p0s13f0_competitor_scaffold_byteflip/` with deterministic fixture/corruption plus per-target command and status capture.
- Comparison scaffold currently records `crushr` and `zip` in this environment, while explicitly deferring `tar+zstd` (`zstd` missing) and `7z` (`7z/7za` missing) without false success.
- `crushr-lab run-first-experiment` now provides a deterministic command path for the recorded first corruption experiment and refreshes artifacts under `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/` (with expected-exit checks for corrupt info/fsck calls).
- `crushr-core::verify` now performs read-only BLK3 scan + payload-hash checks across the blocks region; fsck now reports real `corrupted_blocks` when payload bytes are corrupted.
- `crushr-fsck` snapshot/envelope APIs now require reader access so verification runs against archive bytes while preserving deterministic JSON output.
- `crates/crushr-core/tests/minimal_pack_v1.rs` now includes a real payload-byte corruption case asserting `corrupted_blocks: [0]` and retained footer-corruption failure behavior.
- Workspace hygiene pass fixed `crates/crushr/tests/mvp.rs` binary/path assumptions; `cargo test --workspace` now passes in current environment.
- `crushr-info` now mirrors `crushr-fsck` exit-code policy for open/parse/structural failures (exit `2`), with usage errors at exit `1`; binary-path tests enforce this in `crushr-core::snapshot` tests.
- First real e2e corruption experiment is now recorded at `docs/RESEARCH/artifacts/crushr_p0s12f0_first_e2e_byteflip/` and summarized in `docs/RESEARCH/RESULTS.md` (seeded byteflip, clean pass, corrupt parse failure).
- `crushr-pack` binary now exists at `crates/crushr/src/bin/crushr-pack.rs` and writes minimal self-hosting v1 archives (BLK3 + IDX3 + FTR4 tail frame, no DCT1/LDG1 yet).
- `crushr-core/tests/minimal_pack_v1.rs` now validates real pack->open/info/fsck interoperability plus deterministic tiny-directory output.
- `crushr-fsck` binary now exists at `crates/crushr/src/bin/crushr-fsck.rs` and emits real JSON snapshots from parsed archive metadata.
- `crushr-core::snapshot` now provides typed fsck snapshot mapping helpers and clean-impact emission through `ImpactReportV1`.
- `crushr-core` snapshot tests now include real `crushr-fsck --json` success/failure binary-path coverage over synthetic valid/corrupt archives.
- `OpenArchiveV1`/`InfoSummaryV1` now carry explicit footer metadata so info snapshots include footer offset/length/presence directly.
- `crushr-core` now has a real binary-path test that feeds synthetic v1 bytes to `crushr-info --json` via temp file + `cargo run`.
- `crushr-core` snapshot tests now validate parsed JSON field values (not just string contains), keeping envelope/payload assertions strict and deterministic.
- `crushr-info` CLI e2e JSON test is still deferred because `crushr pack` currently produces legacy archives; wire pack to v1 tail frames first.
- `crushr-core` now has a real `open_archive_v1` path reading tail metadata through `crushr-format`.
- `crushr-core::snapshot` now maps opened archives into typed `InfoSnapshotV1` payloads.
- `crushr-info` binary exists at `crates/crushr/src/bin/crushr-info.rs` and emits JSON snapshots.

Next likely implementation packet:
1. Implement Step 0.13 blast-zone dump implementation (still pending in phase plan).
2. Extend salvage semantics to broader entry/metadata cases only via explicit packeting.
3. Continue Phase F claim-validation/result-recording work.
