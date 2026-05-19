# Harness

The project goal is to provide a reusable operating harness that lets humans and
agents turn a future product spec into safe, validated work.

The app is what users touch. The harness is what agents touch.

## Mental Model

```text
------------------+
| Human intent    |
+------------------+
         |
         v
+------------------+
| Feature intake   |
+------------------+
         |
         v
+------------------+
| Story packet     |
+------------------+
         |
         v
+------------------+
| Agent work loop  |
+------------------+
         |
         v
+------------------+
| Product delta    |
+------------------+
         |
         v
+------------------+
| Validation proof |
+------------------+
         |
         v
+------------------+
| Harness delta    |
+------------------+
         |
         v
+------------------+
| Next intent      |
+------------------+
```

Every task has two possible outputs:

1. Product delta: app code, tests, API shape, data model, or product docs.
2. Harness delta: docs, templates, validation expectations, backlog items, or
   decision records that make the next task easier.

## Harness v0 Scope

Harness v0 includes:

- Agent entrypoint.
- Empty product documentation structure.
- Feature intake and risk lanes.
- Story templates.
- Decision log template.
- Validation report template.
- Test matrix placeholder.
- Harness growth backlog.
- Git workflow skill contracts (`.claude/skills/`).

Harness v0 deliberately excludes:

- A project-specific `SPEC.md`.
- Pre-sliced product domains.
- A locked application stack.
- App source scaffolding.
- Package scripts.
- Test runner config.
- CI workflows.
- Database migrations or infrastructure.

Those should arrive only when a selected story needs them.

## Source Hierarchy

```text
User-provided spec or prompt
  input material for first buildout or future changes

docs/product/*
  current product contract derived from accepted input

docs/stories/*
  story-sized work packets and historical evidence

docs/TEST_MATRIX.md
  behavior-to-proof control panel

docs/decisions/*
  why the contract changed

.claude/skills/*
  git workflow skill contracts (commit, push, pull, land) invoked during execution
```

Before implementation, product docs describe intent. After implementation,
product docs plus executable tests become the living contract.

## Spec Lifecycle

Harness v0 starts without a tracked project spec. When the human provides a
specification, treat it as input material, not as a permanent operating manual.
Use it to populate product docs, story packets, architecture decisions, and
validation expectations during the first buildout.

After the specification has been decomposed, do not keep extending it as the
living product plan. Ongoing work should update the smaller product docs,
stories, test matrix, and decision records.

Ongoing work should enter the harness as one of these input types:

- New spec: a project specification that needs to become product docs and
  initial story candidates.
- Spec slice: a selected behavior from the provided spec.
- Change request: a bounded behavior change, bug fix, or product refinement.
- New initiative: a larger product area that needs multiple stories.
- Maintenance request: dependency, architecture, performance, security, or
  operational work.
- Harness improvement: a process, template, proof, or agent-instruction change.

The spec-to-work loop is:

```text
human intent or supplied spec
  -> classify input type
  -> update or create product contract
  -> create story packet or initiative notes when needed
  -> define validation proof
  -> implement or document the blocker
  -> update product docs, stories, test matrix, and decisions
  -> capture harness friction
```

Large product areas should use scoped initiative notes instead of a second
monolithic specification. An initiative should explain the goal, affected
product docs, candidate stories, validation shape, open decisions, and exit
criteria. If initiative work becomes a repeated pattern, add a template or
proposal to `docs/HARNESS_BACKLOG.md`.

## Execution Phase

Stories move through a defined execution state machine. Agents follow the
default posture rules and honor quality gates at every transition. The state
machine applies to both the Normal lane (single `story.md` packet) and the
High-Risk lane (multi-file bundle — the `## Status` field lives in
`execplan.md`).

### State Machine

Transition the story `## Status` field only when the gate for that state is
fully met. Never advance status optimistically.

| From | To | Gate |
|---|---|---|
| `todo` | `in_progress` | Agent creates workpad sibling, runs pull skill, begins implementation |
| `in_progress` | `human_review` | Workpad Plan + AC checked off, validation green, PR linked, no uncommitted changes |
| `human_review` | `merging` | Human approves PR |
| `human_review` | `rework` | Human posts actionable feedback |
| `rework` | `in_progress` | Agent resets workpad, re-implements |
| `merging` | `done` | PR merged, test matrix row updated, Evidence complete |
| `any` | `blocked` | External blocker documented, human action needed |

Story-execution status (`todo … done`) is distinct from behavior-proof
status in `TEST_MATRIX.md` (`planned … implemented … retired`). Both stay.
See `docs/decisions/0004-execution-state-machine.md` for the rationale.

### Default Posture

1. **Reproduce before changing** — confirm the current broken or missing
   behavior (for bugs) or confirm the feature is absent (for new work) before
   editing code. Record the reproduction signal in workpad Notes.
2. **Workpad-first** — update the workpad sibling's plan before writing
   implementation. Planning lives in the file, not in the agent's memory.
3. **Sync before editing** — invoke the `pull` skill before any code changes.
   Record the sync result in workpad Notes.
4. **Status gates** — only transition story status when the gate for that
   state is fully met. Do not advance status optimistically.
5. **Scope discipline** — out-of-scope improvements discovered during
   execution go to `HARNESS_BACKLOG.md`, not into the current story.
6. **Operate autonomously until truly blocked** — `blocked` is an escape hatch
   for external blockers (missing auth, missing required tool, required human
   decision after exhausting all fallbacks). It is not a shortcut for
   difficult problems.

### Blocked-Access Escape Hatch

When blocked after exhausting all documented fallbacks:

1. Add to workpad Confusions: what is missing, why it is needed, and what
   human action unblocks it.
2. Set story status to `blocked`.
3. Stop. Do not loop or stall indefinitely.

## Growth Rule

The harness grows from friction.

When an agent is confused, repeats manual reasoning, needs a new validation
command, discovers a missing rule, or sees a recurring failure pattern, it must
either improve the harness directly or add a proposal to `HARNESS_BACKLOG.md`.

## Future Validation Ladder

No validation scripts exist yet. When implementation begins, the expected ladder
is:

```text
validate:quick
  format, lint, typecheck, unit tests, architecture check

test:integration
  backend, database, provider, or service checks as the stack requires

test:e2e
  user-visible end-to-end flows

test:platform
  shell, mobile, desktop, or deployment smoke checks as the stack requires

test:release
  full suite, log checks, and performance smoke
```

Agents must not claim these commands pass until they exist and have been run.
