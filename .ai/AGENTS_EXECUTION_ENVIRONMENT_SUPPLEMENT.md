# AGENTS_EXECUTION_ENVIRONMENT_SUPPLEMENT.md

This document defines execution-environment constraints that apply in restricted or sandboxed development contexts.

This file is AI-internal and must reside under the repository root at:

```
.ai/AGENTS_EXECUTION_ENVIRONMENT_SUPPLEMENT.md
```

It is not part of the durable engineering contract. Core process rules live in `AGENTS.md`.

---

## 1. Purpose

This supplement exists to separate temporary execution constraints from long-lived engineering policy.

Nothing in this document may override `AGENTS.md`. If a conflict appears, stop and resolve explicitly.

---

## 2. Toolchain Availability Constraints

In some environments (e.g., chat-based sandboxes):

- A Rust toolchain may not be installed.
- `cargo` commands may not be executable.
- `rg` (ripgrep) may not be available.
- Container runtimes may be unavailable.
- Build scripts may not be runnable.

When tooling is unavailable:

- Do not assume compilation is possible.
- Do not fabricate execution results.
- Use available substitutes (e.g., `grep`, `find`, `sed`).
- Prefer repository-provided container or build scripts when execution is supported.
- Explicitly state when capabilities are unverified.

---

## 3. Test Execution Limitations

In restricted environments:

- Tests may not be executable.
- Integration validation may not be possible.

In such cases:

- Tests must still be authored correctly.
- Tests must remain deterministic and minimal.
- Execution is deferred to a supported environment.
- No claim of "tests passing" may be made without actual execution.

---

## 4. Artifact Handling in Sandbox Environments

When artifacts are generated in constrained environments:

- They may be written to temporary paths (e.g., `/mnt/data`).
- Storage may be ephemeral.
- Retention may be limited.

These are operational constraints of the execution context and must not influence durable artifact naming or versioning rules defined in `AGENTS.md`.

---

## 5. Output Size & Truncation

Some environments may truncate large outputs.

When generating large artifacts or logs:

- Avoid overwhelming the interface with excessive raw output.
- Prefer summarized reporting with clear references.
- Ensure critical information is not silently truncated.

---

## 6. Environment Verification Principle

Before assuming:

- Tool availability
- Filesystem permissions
- Build capability
- Container runtime availability

Explicitly verify or clearly state that capability is unconfirmed.

Environment-specific limitations must never be permanently encoded into repository policy.

---

This supplement is contextual memory for AI operation only.

It must remain under `.ai/` and must not be treated as product documentation.
