# AGENTS.md

This file defines hard rules for working in this repository. Treat it as authoritative.

## 0. Single Source of Truth

- The authoritative working directory is the **single extracted repo root**.
- Do **not** create parallel “current” directories. If a temporary second directory is required, merge changes back immediately and delete it.
- If ambiguity arises about what code is current, **stop work** and ask the user for the last Phase zip to restart cleanly.

## 1. Environment Constraints (Hard)

- There is **no Rust toolchain** available. Do not attempt `cargo test`, `cargo fmt`, `cargo clippy`, or compilation.
- There is **no ripgrep (`rg`)**. Use:
  - `grep -R` / `grep -n` / `grep -R --line-number` for search
  - `find` to locate files
  - `sed -n 'start,endp'` to view sections
- Avoid truncated output. Do not use ellipses. Show complete commands/paths/snippets.

## 2. Deliverables Rules

### Zips

- Produce a **single full repository zip only at the completion of a Phase**.
- Keep the sandbox clean:
  - Only the **two most recent** zip files may remain in `/mnt/data`.
  - Delete older zips when producing a new phase zip.

Practical zip workflow (do not deviate):

- Work tree: `/mnt/data/termgrid-core` (single canonical checkout)
- Zip output path: `/mnt/data/termgrid-core_phase<N>.zip`
- Before writing a new phase zip:
  1. `ls -1 /mnt/data/*.zip` (if any)
  2. Delete all but the newest one (keep at most two total after new zip is created)
- Create the phase zip from the repo root so paths are clean:
  - `cd /mnt/data/termgrid-core && zip -r /mnt/data/termgrid-core_phase<N>.zip .`

### Each Step must include

- Code changes implementing the step’s requirements.
- Documentation updates required by the step.
- Tests for the new behavior.

### Step closure format

At the end of each step, output:

1. **What I did** (short bullets)
2. **What I will do next** (max 2 lines)
3. Ask: **Continue?**

## 3. Decision Policy

If a decision will materially affect the product API or semantics:

- **Stop the step immediately**.
- Present options + consequences clearly.
- Ask the user to choose.

No silent defaults for major-impact choices.

Known locked decision:

- Highlight indexing uses **grapheme indices** (Option A).

Any new locked decisions MUST be appended here with:

- The final choice
- The rationale
- Any API semantics affected

## 4. Engineering Standards

- Staff-level Rust engineering: stable design, clear invariants, additive APIs.
- No “vibe-code” shortcuts.
- Clever solutions are welcome **only if** they are deterministic, safe, and reduce complexity without fragility.
- Prefer:
  - Small, orthogonal APIs
  - Pure functions for layout/measurement
  - Deterministic behavior under a fixed `GlyphRegistry`

## 5. Repository Structure Notes

This repository is treated as a **single canonical crate at the repo root**.

- Canonical locations:

  - `Cargo.toml`, `src/`, `tests/`, `docs/`, `testdata/` at the repository root
- Non-canonical / artifacts:

  - Any nested crate directories are considered **artifacts**.
  - If a nested crate directory exists, it must not be modified as part of normal work.
  - If it is safe to do so, delete nested crates to remove ambiguity.

When adding new public APIs, ensure:

- Root crate code is updated.
- Root crate docs are updated.
- Root crate tests cover the behavior.

## 6. Testing Strategy (Without Running Tests)

Because tests cannot be executed here, tests must be:

- Deterministic
- Minimal
- Written to match existing test conventions

Use the deterministic glyph profile fixtures:
