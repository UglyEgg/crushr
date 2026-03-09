# .ai/HANDOFF.md

Start with:
1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `PROJECT_STATE.md`
5. `REPO_SNAPSHOT.md`
6. `SPEC.md` and `docs/CONTRACTS/*`

Current shape:
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
1. Last-valid-tail scan behavior for damaged/trailing tails.
2. Real `crushr-fsck --json` emission on top of open path.
3. Controlled corruption experiment recording for Phase F.
