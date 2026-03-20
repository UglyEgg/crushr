<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Quality Gates

Local / CI gates:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- parser golden tests
- corruption harness tests
- schema stability tests for snapshots / impact reports

Release gates:
- docs updated
- research claims backed by recorded results
- no unreviewed architectural drift
