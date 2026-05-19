# <Story ID> — <Short Title>

One-line summary of what this story delivers (user-visible or operator-visible).

## Status

`todo`

Status values per `docs/HARNESS.md` state machine: `todo`, `in_progress`,
`human_review`, `rework`, `merging`, `done`, `blocked`.

## Lane

`normal` | `tiny` | `high-risk`

Reason: <one line — why this lane was selected during intake>.

## Depends on

- <story-id or "none">

If this story is `blocked` because a dependency is unfinished, name the
blocker here and ensure status is `blocked` (not `todo`).

## Context

- Phase: <ROADMAP phase number and name>
- Approved architecture: `docs/superpowers/specs/2026-05-18-chronicler-design.md`
- Initiative: `docs/stories/<initiative-folder>/initiative.md`
- Affected product docs: <paths under `docs/product/` or "none yet">

## Acceptance criteria

- [ ] AC1 — <observable behavior> → maps to TEST_MATRIX row `TM-XXX`
- [ ] AC2 — <observable behavior> → maps to TEST_MATRIX row `TM-XXX`

Each AC must be checkable from outside the code (a click, a curl, a file
existing on disk, a log line). "Compiles" is not an AC.

## Validation

Validation rungs that must pass before status can move to `human_review`:

- `validate:quick` — <which checks land in this story; if the script does not
  exist yet, this story creates it>
- Manual smoke — <what a human verifies and screenshots/pastes into Evidence>
- Integration / E2E (if applicable) — <which rung and what it covers>

If a validation script does not exist yet and is in scope, this story
creates the first rung. Do not claim a rung passes until it exists.

## Out of scope

Bullet list of things explicitly deferred to a later story. Add HARNESS_BACKLOG
entries for any structural improvements discovered during execution.

## Evidence

To be filled in during execution:

- Workpad: `<this-story-id>.workpad.md` (sibling of this file)
- PR: <url, once opened>
- Validation logs: <paste or link>
- Manual smoke screenshots: <links or paths>

## Notes for the next agent

If this story is being resumed, the workpad sibling is the priority read — it
contains prior attempt context, decisions, and what stopped the previous run.
