# S02 Workpad — FastAPI sidecar + `/health` round-trip + SQLite `config` table

## Status

`in_progress`

## Notes

2026-05-24: Synced with origin/main. Result: clean (already up to date). HEAD: c8b4352.
Branch created: story/s02-fastapi-sidecar-health-sqlite.

2026-06-13: Synced with origin/main. Result: clean (already up to date). HEAD: f7af37b.
Resuming: found a staged-but-uncommitted fix to `scripts/build-backend.ps1` (adds dev-binary
copies to `target\debug\binaries\` for the Tauri-dev sidecar path resolution issue described
in the 2026-05-28 session log). `git log -- scripts/build-backend.ps1` shows this fix was
never actually committed despite a prior log entry claiming so. Continuing from the
workpad's "Next step for a cold-start agent": verify the fix, rebuild the backend binary,
run `tauri dev`, verify AC1/2/3/6.

2026-06-13 (cont.): Earlier in this session (before context compaction) `pnpm tauri dev` was
launched via a background PowerShell job on Windows. Result: the FastAPI sidecar spawned
without panic under `tauri dev`, and a background PowerShell job confirmed `/health`
responded while Tauri was running. This is recorded here as durable evidence — it was
previously only in ephemeral session memory (`.remember/today-2026-06-13.md`). The job had
already exited by the time this session resumed, so no log/output artifact was captured and
AC1's teardown check (no orphan `chronicler-backend.exe` after window close), AC2 (port via
`get_backend_port`), AC3 (Ping renders JSON in React), and AC6 (Backend unavailable UI state)
were NOT verified — those need a human at the Windows GUI.

Committed the staged `build-backend.ps1` fix (53e717b). Fixed a latent bug in
`frontend/src-tauri/tests/integration_test.rs`'s `find_backend_binary()`: it picked the
first file matching `chronicler-backend-*` in `binaries/`, which could be the 0-byte WSL
`x86_64-unknown-linux-gnu` placeholder instead of the real Windows `.exe`, causing
`Command::spawn()` to fail. Now skips zero-byte files. Ran the Rust integration test
(`cargo test --test integration_test -- --include-ignored`) via Windows cargo — result
pending, see below.

Ran `backend/.venv/bin/pytest -q`: 5/5 pass (AC4 + AC5 fully covered, wired into
`validate:quick`). TM-004 → `implemented`.

Rust integration test result (Windows cargo, `cargo test --test integration_test --
--include-ignored`, first run took 1m44s to compile + 2.83s to run):
```
running 1 test
INFO:     Started server process [12140]
INFO:     Waiting for application startup.
INFO:     Application startup complete.
INFO:     Uvicorn running on http://127.0.0.1:56874 (Press CTRL+C to quit)
INFO:     127.0.0.1:56878 - "GET /health HTTP/1.1" 200 OK
INFO:     127.0.0.1:56878 - "GET /health HTTP/1.1" 200 OK
test test_health_endpoint_json_shape ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.83s
```
Spawn-with-`--port`, startup, and `/health` JSON shape (`status: "ok"`, `last_seen_at`
present) are confirmed by this test.

**Finding relevant to AC1**: after the test reported `ok` and exited, the spawned
`chronicler-backend-x86_64-pc-windows-msvc.exe` (PID 12140) kept running as an orphan for
~1 hour — `Guard::drop()`'s `self.0.kill()` did not actually terminate it (or terminated a
wrapper while the real server process detached/survived). I had to `Stop-Process -Force` it
manually from PowerShell. This is the exact failure mode AC1 prohibits ("no orphan
chronicler-backend.exe processes remain in Task Manager"). **The manual Windows smoke test
for AC1 must explicitly check Task Manager for a lingering `chronicler-backend-*.exe`
process after closing the Tauri window** — if tauri-plugin-shell's sidecar kill behaves the
same way `Child::kill()` did here, AC1 would fail and need a follow-up fix (e.g. killing by
process tree / job object, or having the FastAPI process handle SIGTERM and the Rust side
send that instead of TerminateProcess).

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
- **`find_backend_binary()` fix**: the Rust integration test's binary lookup picked the
  first `chronicler-backend-*` file in `binaries/`, which could be the 0-byte WSL
  `x86_64-unknown-linux-gnu` placeholder rather than the real Windows `.exe` — `Command::
  spawn()` would fail on a 0-byte file. Fixed to skip zero-byte files.

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
