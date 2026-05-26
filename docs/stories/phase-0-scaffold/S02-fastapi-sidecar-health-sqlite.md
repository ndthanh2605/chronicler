# S02 — FastAPI sidecar + `/health` round-trip + SQLite `config` table

Wire FastAPI as a Tauri 2 `externalBin` sidecar, discover its port via
Tauri IPC, route the React UI's "Ping" button to FastAPI `/health`, and
create the SQLite database with a `config` table on first launch.

## Status

`in_progress`

## Lane

`normal`

Reason: still bounded scaffolding work; risk count = 3 (cross-platform via
Tauri externalBin, external systems via PyInstaller bundling, data model
via the first SQLite migration). No hard gate triggers (no auth, no
authorization, no real external network call beyond localhost).

## Depends on

- `S01-tauri-react-shell` (must be `done`)

## Context

- Phase: 0 — Scaffold (`docs/ROADMAP.md`)
- Approved architecture: `docs/superpowers/specs/2026-05-18-chronicler-design.md`
- Initiative: `docs/stories/phase-0-scaffold/initiative.md`
- Affected product docs: none yet
- ADRs informing this story:
  - `docs/decisions/0001-fastapi-sidecar-packaging.md`
  - `docs/decisions/0002-sidecar-port-discovery.md`
- Reference project: `~/Repos/stenoai` (the Python sidecar packaging shape).

## Acceptance criteria

- [ ] AC1 — On `tauri dev` and on a built `tauri build` artifact, Tauri
  spawns the PyInstaller-bundled FastAPI binary registered as `externalBin`
  in `tauri.conf.json`. Closing the Tauri window terminates the sidecar;
  no orphan `chronicler-backend.exe` processes remain in Task Manager.
  → maps to `TM-003`.
- [ ] AC2 — Tauri picks a free TCP port, passes it to the FastAPI binary
  via `--port <N>` on launch, and exposes the chosen port to React via the
  Tauri IPC command `get_backend_port`. Calling `invoke('get_backend_port')`
  from React returns the port number as a string or number. → maps to `TM-003`.
- [ ] AC3 — Clicking "Ping" in the React UI now calls
  `http://127.0.0.1:<port>/health` and renders the JSON payload (e.g.
  `{"status":"ok","last_seen_at":"<ISO-8601>"}`) in the UI. The "pong"
  string from S01 is replaced. → maps to `TM-003`.
- [ ] AC4 — On first launch, the FastAPI sidecar creates
  `%APPDATA%\Chronicler\chronicler.db` and a `config` table with columns
  `key TEXT PRIMARY KEY, value TEXT`. A second launch does not recreate or
  truncate the file. → maps to `TM-004`.
- [ ] AC5 — `/health` writes `last_seen_at = <now ISO-8601>` to the
  `config` table on every call and returns the previous value (or `null`
  on the very first call). This proves aiosqlite read **and** write.
  → maps to `TM-004`.
- [ ] AC6 — If the sidecar binary fails to start (e.g. PyInstaller bundle
  missing), the React UI shows a clear "Backend unavailable" message — no
  white screen, no silent failure, no infinite spinner.
- [ ] AC7 — `validate:quick` is extended to also run Python lint
  (`ruff check`) and typecheck (`mypy` or `pyright` — pick in ADR if not
  obvious from the architecture spec) against `backend/`.

## Validation

- `validate:quick` — extends the script created in S01 with Python lint
  and typecheck.
- Manual platform smoke (Windows) — fresh user profile, first launch
  creates the DB; restart and confirm `last_seen_at` is preserved.
  Screenshot the rendered JSON payload after clicking "Ping."
- Integration test (Rust) — a `#[tokio::test]` (or equivalent) that
  spawns the PyInstaller binary on a free port, curls `/health`, and
  asserts the JSON shape. Tears the process down at the end.
- Integration test (Python) — `pytest` test against a temporary
  `%APPDATA%`-like dir asserting the `config` table is created and that
  `last_seen_at` round-trips.

## Out of scope

- Full SQLite schema (`meetings`, `transcripts`, `summaries`) — Phase 3.
- Real ASR / whisper code — Phase 2.
- Any production code-signing of the bundled binary — Phase 6.
- Pyinstaller build automation in CI — `HARNESS_BACKLOG` entry; not in v0.

## Evidence

To be filled in during execution:

- Workpad: `S02-fastapi-sidecar-health-sqlite.workpad.md`
- PR: <url, once opened>
- `validate:quick` log: <paste or link>
- Manual smoke screenshots:
  - "Ping → JSON payload": <path>
  - DB file existence + table schema (`sqlite3 chronicler.db .schema`): <paste>
- Rust integration test output: <paste>
- Python integration test output: <paste>

## Notes for the next agent

- Tauri 2's `externalBin` configuration lives in
  `frontend/src-tauri/tauri.conf.json` under `bundle.externalBin`. The
  binary path must include the Rust target triple suffix (e.g.
  `chronicler-backend-x86_64-pc-windows-msvc.exe`) — Tauri docs are
  authoritative here; cross-check with `context7`.
- The PyInstaller spec should produce a one-file binary unless startup
  time is unacceptable — note any deviation in an ADR.
- Choose the Tauri ↔ FastAPI launch shape: `tauri-plugin-shell`'s sidecar
  API (preferred) vs raw `Command::new`. Pick the plugin-API path unless
  there's a documented reason not to.
- The DB path computation must be platform-aware. On Windows it's
  `%APPDATA%\Chronicler\`; document the lookup helper in code comments
  only if non-obvious.
- The "Backend unavailable" UI state (AC6) is critical — Phase 4 will rely
  on the same surface for "Vast.ai endpoint unreachable." Build it as a
  reusable component, not an inline `if` in the Ping handler.
