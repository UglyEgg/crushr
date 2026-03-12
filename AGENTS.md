# AGENTS.md

## Mandatory bootstrap rule

Before any new implementation work, AI contributors must read:

- `AI_BOOTSTRAP.md`
- `REPO_GUARDRAILS.md`

If they cannot summarize the current state and active step from those files, they must not modify code.

## 1. Authority & Conflict Resolution

This document defines the default operating contract for AI-assisted development within this repository.

If a repository contains its own `AGENTS.md` and any conflict arises between documents, work must stop and the conflict must be resolved explicitly before proceeding. Silent overrides are not permitted.

No assumptions of precedence are allowed. Resolution must be deliberate and documented.

### Operational precedence order

When multiple continuity documents exist, apply this order:

1. `AGENTS.md` (durable policy for the repository)
2. `.ai/STATUS.md` (authoritative “what is true right now”)
3. `.ai/DECISION_LOG.md` (resolved decisions and non-negotiables)
4. `.ai/PHASE_PLAN.md` (execution plan)
5. `.ai/*` supplements (contextual only; never override policy)

If any conflict is detected, stop and resolve explicitly.

---

## 2. Development Model: Phase → Step Workflow

All work is organized into Phases. Each Phase consists of sequential Steps.

### Phase

A Phase represents a coherent milestone of meaningful progress (feature complete, architectural unit complete, or major refactor complete).

### Step

A Step is a bounded unit of work within a Phase. Steps must:

- Be small enough to complete safely and coherently.
- Avoid scope drift.
- Produce code, documentation, and tests together when applicable.

If work grows beyond the intended scope of a Step:

- Finish the coherent portion.
- Insert a new Step immediately after.
- Do not silently expand the original Step.

---

## 3. Repository AI Workspace (`.ai/`)

This repository uses a non-standard AI workspace directory at the repo root.

- `.ai/` contains documents intended for AI internal tracking and continuity.
- These documents exist to help fresh AI instances ramp quickly with minimal context loss.
- `.ai/` is not product surface area. It is internal project memory.

If `.ai/` does not exist, create it.

### 3.1 Required minimal set

The following files are required and must be kept current:

- `.ai/INDEX.md` — entry point (“start here”) and links to all other `.ai/` documents
- `.ai/STATUS.md` — **single source of truth** for current Phase/Step and active state
- `.ai/PHASE_PLAN.md` — current Phase plan with Step checklist
- `.ai/DECISION_LOG.md` — resolved decisions with rationale and blast radius
- `.ai/BACKLOG.md` — deferred work and future ideas (non-active)
- `.ai/HANDOFF.md` — instructions for a fresh instance to take over with minimal drift
- `.ai/CHANGELOG.md` — concise chronological record of completed Steps

### 3.2 Single source of truth

`.ai/STATUS.md` is authoritative for:

- Current Phase and Step
- What is in progress vs complete
- Active constraints (e.g., musl/glibc, tooling limitations)
- Known risks/gotchas
- Next actions

If chat output conflicts with `.ai/STATUS.md`, treat `.ai/STATUS.md` as correct and stop to resolve the discrepancy.

---

## 4. Session Start and Session End Procedures

These procedures are mandatory to minimize drift during development and across handoff.

### 4.1 Session Start (required)

Before making changes:

1. Read `AGENTS.md`.
2. Read `.ai/INDEX.md`.
3. Read `.ai/STATUS.md`.
4. Read `.ai/DECISION_LOG.md`.
5. Confirm the active Phase/Step (from `.ai/STATUS.md`).
6. Confirm active constraints (build targets, environment limitations, known risks).

Only then proceed.

### 4.2 Session End (required)

Before ending a Step (or handing off):

1. Update `.ai/STATUS.md`:
   - Phase/Step
   - what changed
   - what remains
   - constraints/gotchas
   - next actions
2. Update `.ai/PHASE_PLAN.md` checklist.
3. If any decision was made or a policy changed, update `.ai/DECISION_LOG.md`.
4. Update `.ai/HANDOFF.md` if takeover instructions changed.
5. Append an entry to `.ai/CHANGELOG.md` for the completed Step.
6. Provide the Step Closeout summary (Section 12).

---

## 5. Scope Discipline

Work must remain within the scope of the current repository.

Do not:

- Modify other repositories unless explicitly directed.
- Introduce cross-repository architectural changes without explicit approval.
- Expand feature scope beyond the defined Phase.

If a Step requires architectural, contractual, or feature-direction changes:

- Stop work.
- Document context, options, recommendation, and blast radius.
- Await explicit user decision.

---

## 6. Decision Escalation Protocol

If a decision impacts:

- Public contracts
- Data formats
- Architecture
- Feature direction
- Backward compatibility

Then:

1. Stop work.
2. Write context.
3. Provide viable options.
4. Recommend one option.
5. Explain blast radius.
6. Await explicit decision before continuing.

Recommendation criteria:

- Prefer stability and long-term growth over short-term simplicity.
- If the best option implies a major refactor, rewrite, or pivot, recommend it.
- Make scope explicit (what changes, what is removed, what migrates, risk profile, and transition plan).

Decision recording requirement:

- Every resolved decision must be recorded in `.ai/DECISION_LOG.md` (date, decision, alternatives, rationale, blast radius).

No forward progress may occur past this boundary without resolution.

---

## 7. Build & Linking Targets

Unless explicitly overridden by repo-local docs:

- **C++** is compiled and linked against **glibc**.
- **Rust** is normally built as a **statically linked musl** binary.

If required functionality cannot be provided under musl (compatibility, missing features, third-party constraints, etc.):

- Document the limitation explicitly (what fails, why, and alternatives).
- Propose switching that target to glibc.
- Do not silently change the build target.

If Rust is switched from musl to glibc for this repository, it must remain on glibc unless the reason for the switch is removed (dependency eliminated or a verified workaround replaces it). Record the rationale in `.ai/DECISION_LOG.md`.

---

## 8. CI / Build Harness Alignment (podCI)

A small CI-like harness exists for building Rust and C++ using Podman: `podCI`.

- When this repository needs build/test automation, prefer aligning with `podCI` conventions rather than inventing a parallel harness.
- If the harness must be extended, treat changes as architectural/contractual decisions and follow the Decision Escalation Protocol.

Pinning policy (disabled for now):

- Once `podCI` is publicly accessible and stable, record a reference in `.ai/STATUS.md` (tag/commit) as `podCI_ref`.
- Until then, do not block work on `podCI_ref`.

---

## 9. Step Completeness Criteria

A Step is complete only when:

- Code changes are coherent and internally consistent.
- Documentation is updated where applicable.
- Relevant tests are added or updated when appropriate.
- Tests are deterministic and minimal.
- Tests pass in a supported execution environment.

If the current environment does not allow execution of tests, execution is deferred but tests must still be authored correctly.

If an artifact is generated (zip, tarball, release bundle, etc.), record the exact filename in:

- `.ai/STATUS.md`
- `.ai/CHANGELOG.md`

A Step is not complete if only code was written without documentation or tests when they are applicable.

---

## 10. Artifact Naming & Versioning

### Artifact Naming

Artifacts must follow the format:

```
<product_name>_p<phase_number>s<step_number>f<fix_iteration>
```

Where:

- `phase_number` = current Phase
- `step_number` = Step within Phase
- `fix_iteration` = debugging iteration within the Step

`fix_iteration` increments only when debugging during a Step or at an end-of-phase fix.
It resets to `0` once the issue is resolved.

### Versioning Rules

- Completing a Phase increments minor version: `0.1.0` and resets patch to `0`.
- A fix iteration increments patch version: `0.0.1`.

---

## 11. Phase Completion Rules

By default, artifact packaging occurs at Phase completion.

Artifacts may be generated during a Step only when:

- Explicitly requested, or
- A fix checkpoint requires preservation.

---

## 12. Step Closeout Format

At the end of each Step, provide a structured summary including:

1. What changed
2. What remains (next Step)
3. New constraints or gotchas discovered
4. Continue?

This ensures traceability and prevents context drift across sessions.

---

## 13. Documentation Standards

Documentation must:

- Be written for a mid-level engineer.
- Avoid unnecessary verbosity.
- Avoid unexplained assumptions.
- Prefer clarity over stylistic flourish.

Diagrams (e.g., Mermaid) should be included when they materially improve understanding of flows, contracts, or decision paths.

---

## 14. Behavioral Principles

- No silent scope expansion.
- No silent architectural drift.
- No undocumented decisions.
- No incomplete Steps.
- No environment assumptions baked into durable policy.

Process is durable. Execution constraints belong under `.ai/` as supplemental documents.

## Implementation-agent workflow

Implementation agents (Codex/Context/etc.) are not design authorities.

They must:

- treat `SPEC.md`, `docs/CONTRACTS/*`, `.ai/STATUS.md`, and `.ai/DECISION_LOG.md` as canonical
- work from a bounded task packet
- avoid architectural changes unless the packet explicitly authorizes them
- include tests with every meaningful code change

They must not:

- redesign the format
- weaken the Failure-Domain Determinism thesis
- add hidden heuristics or undocumented behavior
- silently update contracts to match code
