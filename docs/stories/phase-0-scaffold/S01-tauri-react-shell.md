# S01 — Tauri + React shell + minimal IPC

Stand up the Tauri 2 desktop shell embedding a Vite-built React UI, with a
minimal Tauri↔React IPC roundtrip proving the plumbing works. No FastAPI
sidecar in this story.

## Status

`in_progress`

## Lane

`normal`

Reason: scaffolding the first application code in the repo. Risk count = 2
(cross-platform, data model not yet touched). Bounded blast radius — no
existing behavior to disturb.

## Depends on

- none

## Context

- Phase: 0 — Scaffold (`docs/ROADMAP.md`)
- Approved architecture: `docs/superpowers/specs/2026-05-18-chronicler-design.md`
- Initiative: `docs/stories/phase-0-scaffold/initiative.md`
- Affected product docs: none yet
- Component ownership: `docs/ARCHITECTURE.md` ("Tauri / Rust" and "React UI")
- Reference project: `~/Repos/stenoai` (Electron + Python sidecar pattern;
  Tauri's equivalent shape applies to layout and dev-loop ergonomics).

## Acceptance criteria

- [ ] AC1 — `pnpm tauri dev` opens a native Windows window titled "Chronicler"
  rendering the React page with the app title and a "Ping" button. Hot
  reload of the React source updates the window without a manual rebuild.
  → maps to TEST_MATRIX row `TM-001`.
- [ ] AC2 — Clicking "Ping" invokes the Tauri IPC command `ping` defined in
  Rust; the handler returns the literal string `"pong"`; the React UI
  displays the returned value. → maps to TEST_MATRIX row `TM-002`.
- [ ] AC3 — `pnpm tauri build` produces a Windows artifact under
  `frontend/src-tauri/target/release/bundle/`. Code signing is **out of
  scope** for this story (Phase 6); unsigned output is acceptable.
- [ ] AC4 — A first `validate:quick` script exists at the repo root (e.g.
  `scripts/validate-quick.sh` or an npm/just task) and runs: React lint +
  typecheck (`tsc --noEmit`) and `cargo test` in the Rust crate (upgraded from
  `cargo check` so the `ping_returns_pong` unit test also runs in CI).

## Validation

- `validate:quick` — newly created in this story (`scripts/validate-quick.sh`).
  Runs ESLint + `tsc --noEmit` for the React workspace and `cargo test` for the
  Rust crate (which executes the `ping_returns_pong` unit test).
- Manual smoke — start `tauri dev`, take a screenshot of the Tauri window
  with the rendered "pong" after a click. Attach to Evidence.
- Unit test (Rust) — `#[cfg(test)]` test on the `ping` command handler
  asserting it returns `"pong"`. Lives in `frontend/src-tauri/src/`.

## Out of scope

- FastAPI sidecar, PyInstaller, port discovery — `S02`.
- SQLite — `S02`.
- Any audio code — Phase 1.
- Code signing / installer — Phase 6.
- CI workflow files — not yet authorized by harness v0.

## Evidence

- Workpad: `S01-tauri-react-shell.workpad.md` (sibling of this file)
- PR: https://github.com/ndthanh2605/chronicler/pull/1
- `validate:quick`: all steps green (ESLint clean, tsc clean, `ping_returns_pong ... ok`)
- Manual smoke: user-confirmed window opened on Windows with `cargo tauri dev`
- Rust unit test: `test tests::ping_returns_pong ... ok` (cargo test via cmd.exe, 2026-05-21)

## Notes for the next agent

- The workspace layout is fixed by `CLAUDE.md`: React lives under
  `frontend/src/`, Tauri Rust under `frontend/src-tauri/`. Do not propose
  alternative layouts in this story; if you believe a change is needed,
  raise it in `HARNESS_BACKLOG.md` and pause.
- The `ping` command is intentionally trivial. S02 upgrades its handler to
  call `/health` on the FastAPI sidecar. Keep the Rust signature reusable.
- Use `pnpm` if no package manager has been chosen yet — make the choice
  inside an ADR (`docs/decisions/0003-package-manager.md`) before scripts
  land. Update this story's AC4 if a different manager is selected.
