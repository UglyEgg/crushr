# .ai/HANDOFF.md

Start with:
1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `PROJECT_STATE.md`
5. `REPO_SNAPSHOT.md`
6. `SPEC.md` and `docs/CONTRACTS/*`

Current shape:
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
