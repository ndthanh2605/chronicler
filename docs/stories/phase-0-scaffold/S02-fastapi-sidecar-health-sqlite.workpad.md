# S02 Workpad — FastAPI sidecar + `/health` round-trip + SQLite `config` table

## Status

`done`

## Notes

2026-06-15: Windows GUI smoke test **passed** (user-confirmed) — AC1 (no orphan
sidecar on window close), AC2 (`get_backend_port`), AC3 (Ping → `/health` JSON),
AC6 (Backend-unavailable UI). All 7 ACs now checked; TM-003 → `implemented`.
Status → `merging`; landing PR #2 to `main`.

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
- **CORS middleware (AC3 fix)**: the GUI smoke test showed the React Ping button always
  reported "Backend unavailable" even though `/health` returns 200 when curled directly.
  Root cause: `tauri dev`'s webview loads the React app from `http://localhost:1420`
  (Vite devUrl), and FastAPI had no `CORSMiddleware`, so the cross-origin `fetch` to
  `http://127.0.0.1:<port>/health` was blocked by the browser and `App.tsx`'s `catch`
  turned it into `status: "unavailable"`. Fixed by adding `CORSMiddleware` to
  `backend/app/main.py` allowing `http://localhost:1420`, `http://tauri.localhost`, and
  `tauri://localhost` (dev + built-app origins on Windows/Linux/macOS).
- **Orphan sidecar fix (AC1)**: the Rust integration test showed the spawned
  `chronicler-backend-*.exe` survives `CommandChild::kill()` as an orphan — confirmed live
  on Windows (4 orphan `chronicler-backend` processes accumulated across `tauri dev`
  launches, 2 per launch). Root cause: PyInstaller's onefile bootloader extracts and
  launches the real interpreter as a child process; `kill()` only kills the bootloader
  PID. Fixed `frontend/src-tauri/src/lib.rs`'s `on_window_event(Destroyed)` handler to call
  a new `kill_backend()` helper that runs `taskkill /F /T /PID <pid>` on Windows (kills the
  whole process tree); non-Windows falls back to `child.kill()`.
- **Non-fatal sidecar spawn (AC6 fix)**: `setup()` previously used
  `.expect("sidecar binary not found...")` and `.expect("failed to spawn backend
  sidecar")` — if the binary were missing, this would panic the setup closure and crash
  the whole app (`run().expect(...)` aborts), which is exactly the white-screen/crash AC6
  forbids. Changed to a `match` that logs via `eprintln!` and leaves `BACKEND_CHILD` as
  `None` on failure; `BACKEND_PORT` is still set, so React's `/health` fetch fails
  normally and the existing "Backend unavailable" UI path (AC6) handles it gracefully.
- Also changed `App.tsx`'s `catch {}` to `catch (err) { console.error("Ping failed:",
  err); ... }` so future CORS/network failures are visible in devtools instead of
  silently swallowed.
- `cargo check --tests` (Windows) passes after these `lib.rs` changes; `.venv/bin/ruff` +
  `.venv/bin/pyright` + `.venv/bin/pytest` (5/5) pass after the `main.py` CORS change.

## What was not attempted

- AC1 — sidecar spawn confirmed (11:12 session + Rust test), but **teardown is not
  confirmed**: a human must close the `tauri dev` window and check Task Manager for a
  lingering `chronicler-backend-x86_64-pc-windows-msvc.exe`. The Rust test showed the
  spawned process survives `Child::kill()` as an orphan — this may also affect
  tauri-plugin-shell's sidecar teardown.
- AC2 (`get_backend_port` IPC returns a usable port to React) — requires running app on
  Windows with DevTools/console inspection.
- AC3 (Ping → `/health` JSON payload rendered in React UI) — requires running app on
  Windows.
- AC6 (Backend unavailable UI state) — requires running app on Windows (e.g. temporarily
  rename/remove the dev sidecar binary and confirm the UI shows a clear message, not a
  white screen or infinite spinner).
- AC4/AC5 — fully covered by `backend/.venv/bin/pytest` (5/5 ✓), wired into
  `validate:quick`. TM-004 → `implemented`.
- AC7 — `validate:quick` already extended with ruff/pyright/pytest per the "Decisions
  made" section (verify still wired if resuming cold).
- Rust integration test (`--include-ignored`) — passes (see Notes above). Covers spawn +
  `/health` JSON shape for TM-003, but not the manual-click or teardown portions.

## Next step for a cold-start agent

All source code is committed (HEAD includes the build-backend.ps1 fix, the
find_backend_binary fix, and evidence updates). AC1 (teardown)/AC2/AC3/AC6 require a human
at a Windows GUI — they cannot be completed by an agent alone. To finish the story:
1. Run `pnpm tauri dev` from `frontend/` on Windows (binary is already built at
   `frontend/src-tauri/target/debug/binaries/chronicler-backend.exe` and
   `frontend/src-tauri/binaries/chronicler-backend-x86_64-pc-windows-msvc.exe`; rerun
   `pnpm dev:backend` only if these are missing/stale).
2. Click "Ping" — confirm the rendered JSON payload matches `{"status":"ok",
   "last_seen_at": ...}` (AC3). Screenshot it.
3. Confirm `invoke('get_backend_port')` returns a port number (check via DevTools console
   or temporary log) (AC2).
4. Close the Tauri window, then check Task Manager / `Get-Process chronicler-backend-*`
   for a lingering process — must be NONE (AC1 teardown). If one remains, this is a known
   risk (see workpad Notes) and needs a follow-up fix before AC1 can be checked off.
5. Temporarily break the sidecar (rename the binary) and relaunch `tauri dev`; confirm the
   UI shows a clear "Backend unavailable" message, not a white screen/infinite spinner
   (AC6). Restore the binary afterward.
6. Record screenshots/results in the story packet's Evidence section, check off AC1–AC3,
   AC6 (and AC7 if still unchecked).
7. If all ACs pass and `validate:quick` is green: transition story status to
   `human_review` and run `harness-git-push`.
