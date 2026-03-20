<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# .ai/BACKLOG.md

Deferred items only (non-active).

## Active deferrals from current resilience direction

- [ ] Complete the Phase-09 corruption grid after FORMAT-08 placement results are stabilized and the curated grid is approved.
- [ ] Use Phase-09 results to decide whether simple duplicated/redundant metadata surfaces should be retained, demoted to optional experimental ballast, or removed.
- [ ] Evaluate whether any metadata placement strategy meaningfully improves named recovery, anonymous recovery, or metadata-node survival enough to justify permanent adoption.
- [ ] Decide whether anonymous verified recovery should remain research-only or graduate into a broader experimental baseline after richer grid evidence exists.

## Product-completeness track (deferred until current resilience evaluation settles)

- [ ] Add a Unix metadata preservation envelope so crushr cannot be dismissed as “file-bytes only” on Unix-like systems.
  - file type
  - mode
  - uid/gid
  - uname/gname if kept
  - mtime policy
  - symlink target
  - xattrs
- [ ] Decide later whether advanced Unix metadata should be supported:
  - POSIX ACLs
  - Linux capabilities
  - device node metadata
  - SELinux labels
  - hardlink identity

## Compression optimization track (deferred until format structure is stable)

- [ ] Revisit distributed dictionary design after the resilience architecture and metadata-layer pruning decisions settle.
- [ ] Compare archive-global vs clustered vs explicit dictionary-object approaches under crushr’s verification-first rules.
- [ ] Measure whether dictionary gains justify added metadata/dependency complexity for real corpora.

## Later

- [ ] Packaging/distribution polish once research phases complete.
- [ ] Add cross-format fixture curation guide after the current graph/resilience direction stabilizes.
- [ ] Consider TUI experiment snapshot overlays after salvage/recovery graph reporting stabilizes.
