# Architecture

## Goals

- Provide a vendorable Rust library with a strict, versioned on-disk format contract.
- Offer a small suite of CLI tools and a TUI for "geek-level" inspection and repair.
- Keep all structural parsing and validation in the library (no duplicated parsers in tools).
- Avoid IPC/RPC between tools; all tools link to the same crates and call APIs in-process.
- Keep basic functionality portable; degrade OS-specific metadata (xattrs, permissions) on non-Unix platforms as "best effort" with explicit reporting.

## Crate graph

- `crushr-format`
  - Byte layouts (BLK*/IDX*/DCT*/FTR*)
  - Encoding/decoding
  - Structural invariants and validation helpers
  - No filesystem IO, no CLI/TUI logic

- `crushr-core`
  - Engine and algorithms (verify/repair planning, structural traversal)
  - Operates over minimal random-access IO traits (`ReadAt`, `WriteAt`, `Len`, etc.)
  - Returns typed reports (verify/repair outcomes) suitable for both CLI and TUI

- `crushr`
  - Platform integration
  - Filesystem walking, metadata capture/restore (xattrs behind `cfg(unix)`)
  - Caching, concurrency orchestration
  - Convenience wrappers around `crushr-core`

- `crushr-cli-common`
  - Global flags (e.g., `--json`, `--color`, `--verbose`, `--threads`, `--cache-mib`)
  - Logging initialization
  - Output formatting (human + JSON)
  - Standard exit codes

## Tool suite

The CLI is expected to evolve into multiple focused tools rather than a single binary with many subcommands.

Tagging (mutation boundaries):

- **Read-only**: `crushr-info`
- **Writes new archive**: `crushr-pack`
- **May mutate existing archive** (bounded): `crushr-fsck` (tail repair/salvage/append-style operations)
- **Writes filesystem**: `crushr-extract` (archive is read-only)
- **Interactive**: `crushr-tui` (read-mostly; writes only via explicit actions)

No tool should implement ad-hoc parsing. Tools should call library APIs and render typed structures.

## TUI data pipeline

`crushr-tui` supports two input modes:

1. **Live mode**: open an archive directly and compute the same structural views as `crushr-info`/`crushr-fsck`.
2. **Snapshot mode**: load precomputed JSON outputs from `crushr-info --json` and/or `crushr-fsck --json`.

Both modes feed a shared, typed model (summary, tail frames, dict table, block map, file/index entries, and fsck impact/blast reports).
Snapshot mode exists to enable offline analysis, sharing, regression tests, and deterministic repro cases.

### Snapshot compatibility

Snapshots are versioned and include an `archive_fingerprint`.

- If multiple snapshots are loaded, their `archive_fingerprint` values **must match** to be merged.
- If they do not match, the TUI must present them as separate datasets and warn the user.

The normative snapshot contract is documented in `docs/SNAPSHOT_FORMAT.md`.

## No-IPC rule

All tools link to the same Rust crates and communicate via in-process function calls.

- No JSON-over-stdio protocols between tools.
- No Unix sockets.
- No background daemons.

If a future use case truly requires IPC (e.g., remote inspection), it must be an explicit architectural decision recorded in `.ai/DECISION_LOG.md`.


## Source-of-truth map

For documentation precedence and repo layout, see `docs/README.md` and `../REPO_LAYOUT.md`.
