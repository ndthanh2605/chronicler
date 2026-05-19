---
name: harness-intake
description: >
  Run the harness feature intake workflow before starting any task in this project.
  Classifies the input, selects the execution lane, locates or proposes the story
  packet, and confirms the agent is cleared to proceed. Invoke at the start of
  every non-trivial task — before reading code, before planning, before any
  implementation. Triggers on any user request that involves creating, changing,
  or fixing something in the project.
---

# Harness Intake

This skill is **rigid**. Follow every step in order. Do not skip steps because
the task seems obvious. The intake process is the harness's primary defence
against scope drift and misclassified risk.

---

## Step 1 — Read the source files

Before classifying, read these two files fresh:

```
docs/FEATURE_INTAKE.md   ← classification rules and lane definitions
docs/stories/            ← list what stories already exist
```

Do not rely on memory of prior reads. Harness docs are living documents.

---

## Step 2 — Identify the input type

Match the user's request to exactly one of:

| Type | Signal |
|------|--------|
| **New spec** | User provides a full product spec or requirements document |
| **Spec slice** | User provides a focused subset of an existing spec |
| **Change request** | Modify or extend something already in the harness |
| **New initiative** | Adds a capability with no prior story or spec |
| **Maintenance request** | Fix, clarify, or update without adding capability |
| **Harness improvement** | Change to the operating model, skills, or docs infrastructure |

If the request spans more than one type, treat it as the **highest-risk** type present.

---

## Step 3 — Run the risk checklist

Read the Risk Checklist in `docs/FEATURE_INTAKE.md` and count how many flags apply
to this request. Then apply:

| Flags | Lane |
|-------|------|
| 0–1 | **tiny** or **normal** based on code impact |
| 2–3 | **normal** with stronger validation |
| 4+  | **high-risk** |

**Hard gates — always high-risk regardless of flag count:**
- Auth or authorization changes
- Data loss or migration
- Audit or security
- External provider behavior
- Removing or weakening any validation requirement

If a hard gate applies and the human has not explicitly narrowed scope,
stop and ask before proceeding.

---

## Step 4 — Locate or propose the story

Search `docs/stories/` for an existing story that covers this request.

- **Found:** note the file path and current `## Status`. Confirm the status gate
  allows the next action (see state machine in `docs/HARNESS.md`).
- **Not found (tiny/normal):** propose creating a new `story.md` packet.
  Do not create it until the human confirms.
- **Not found (high-risk):** propose creating the full high-risk bundle
  (`story.md`, `execplan.md`, `workpad.md`). Do not create until confirmed.

---

## Step 5 — Check scope against Harness v0 constraint

Confirm the request does not violate the v0 rule:

> Agents do not scaffold application source folders, platform shells, package
> scripts, CI, or tests unless a story explicitly moves the project into
> implementation.

If the request would violate this, surface it explicitly before proceeding.

---

## Step 6 — Produce the intake report

Before doing any implementation work, output a short report in this exact format:

```
## Intake Report

**Input type:** <type>
**Lane:** <tiny | normal | high-risk>
**Risk flags:** <N> — <list flags, or "none">
**Hard gate:** <yes — <which one> | no>
**Story:** <path/to/story.md — status: <status>> | proposed: <title>
**v0 constraint:** <clear | violation: <description>>

**Cleared to proceed:** <yes | no — reason>
```

Do not begin implementation until the report is written and, for high-risk
requests, the human has acknowledged it.

---

## Step 7 — Transition story status

If proceeding:

- If story is `todo`: transition to `in_progress` only after running the `pull`
  skill and creating the workpad sibling. Record the transition in the workpad.
- If story is already `in_progress`: confirm you are the intended continuation
  (check workpad Notes for prior agent context).
- If story is `blocked`: do not proceed without human resolution of the blocker.

---

## Done definition reminder

When you finish the task, confirm each item before declaring done:

- [ ] Requested change completed or blocker documented
- [ ] Affected product docs, stories, and TEST_MATRIX.md rows updated
- [ ] Validation commands run (if they exist)
- [ ] Out-of-scope discoveries added to `docs/HARNESS_BACKLOG.md`
- [ ] Story status at the correct state-machine gate
- [ ] Final response states what changed and what was not attempted
