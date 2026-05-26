# S02 Workpad — FastAPI sidecar + `/health` round-trip + SQLite `config` table

## Status

`in_progress`

## Notes

2026-05-24: Synced with origin/main. Result: clean (already up to date). HEAD: c8b4352.
Branch created: story/s02-fastapi-sidecar-health-sqlite.

## Prior context

S01 merged (PR #1, commit a23ed98). The Tauri 2 + React shell exists with a `ping` IPC
command that returns the hardcoded string `"pong"`. S02 upgrades this to a real FastAPI
round-trip via `get_backend_port` IPC + `http://127.0.0.1:<port>/health`.

## Implementation plan

### 1. Backend scaffolding (`backend/`)
- `backend/main.py` — FastAPI app with `/health` endpoint
- `backend/db.py` — aiosqlite helper; creates `chronicler.db` + `config` table on startup
- `backend/requirements.txt` + `pyproject.toml` (for ruff/mypy)
- `backend/chronicler-backend.spec` — PyInstaller one-file spec

### 2. Tauri sidecar wiring (`frontend/src-tauri/`)
- `tauri.conf.json` — add `bundle.externalBin` entry + `tauri-plugin-shell` permission
- `Cargo.toml` — add `tauri-plugin-shell` dependency
- `src/lib.rs` — replace `ping` command with `get_backend_port`; spawn sidecar with `--port <N>`

### 3. React UI (`frontend/src/`)
- Replace hardcoded ping with `invoke('get_backend_port')` → fetch `/health`
- Add `BackendStatus` reusable component (AC6 — "Backend unavailable" surface)
- Display returned JSON payload on success

### 4. Validate & test
- Extend `validate:quick` with `ruff check` + `mypy` for `backend/` (AC7)
- Python pytest — `config` table creation + `last_seen_at` round-trip (AC5)
- Rust integration test — spawn binary, curl `/health`, assert JSON shape

## Decisions made during this story

- **ADR-0004**: pyright chosen over mypy for Python type checking. See
  `docs/decisions/0004-python-type-checker.md`.
- **Fixture approach**: `httpx.ASGITransport` does NOT trigger ASGI lifespan events.
  Fixed by calling `init_db()` explicitly in the `client` pytest fixture.
- **WSL placeholder binary**: Tauri's build script validates externalBin path at compile
  time. On WSL (Linux), the target triple is `x86_64-unknown-linux-gnu`. `validate:quick`
  creates an empty placeholder via `touch` before running `cargo test`. The placeholder is
  gitignored. Real Windows binary is produced by `pnpm dev:backend`.
- **validate:quick**: Uses `.venv/bin/ruff`, `.venv/bin/pyright`, `.venv/bin/pytest` since
  the tools are in `backend/.venv` (created with `uv venv`). Run `uv venv && uv pip install
  -r requirements-dev.txt` in `backend/` to set up the venv.

## What was not attempted

- AC1 (sidecar spawns + teardown on Windows) — requires Windows + PyInstaller binary
- AC2 (get_backend_port IPC) — requires running app on Windows
- AC3 (Ping → /health JSON payload in React) — requires running app on Windows
- AC6 (Backend unavailable UI state) — requires running app on Windows
- Python integration test for AC5 has passed (5/5 ✓)
- Rust integration test exists but is `#[ignore]` — requires PyInstaller binary

## Next step for a cold-start agent

All source code is committed. To complete the story:
1. On Windows: run `pnpm dev:backend` (builds PyInstaller binary)
2. Run `pnpm tauri dev` from `frontend/`
3. Verify ACs 1–3, 6 manually
4. Record evidence in story packet
5. Mark TM-003 and TM-004 as `implemented`
6. Run Rust integration test with `--include-ignored`
