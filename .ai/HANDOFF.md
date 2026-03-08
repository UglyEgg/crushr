# .ai/HANDOFF.md

Start with:
1. `AGENTS.md`
2. `.ai/STATUS.md`
3. `.ai/DECISION_LOG.md`
4. `PROJECT_STATE.md`
5. `REPO_SNAPSHOT.md`
6. `SPEC.md` and `docs/CONTRACTS/*`

Current shape:
- `crushr-core` now has a real `open_archive_v1` path reading tail metadata through `crushr-format`.
- `crushr-core::snapshot` now maps opened archives into typed `InfoSnapshotV1` payloads.
- `crushr-info` binary exists at `crates/crushr/src/bin/crushr-info.rs` and emits JSON snapshots.

Next likely implementation packet:
1. Last-valid-tail scan behavior for damaged/trailing tails.
2. Real `crushr-fsck --json` emission on top of open path.
3. Controlled corruption experiment recording for Phase F.
