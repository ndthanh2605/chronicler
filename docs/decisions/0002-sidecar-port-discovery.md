# ADR-0002 — Sidecar port discovery

- **Status:** Accepted
- **Date:** 2026-05-19
- **Initiative:** `docs/stories/phase-0-scaffold/initiative.md`
- **Stories:** `S02-fastapi-sidecar-health-sqlite`

## Context

Chronicler's React UI must call the FastAPI sidecar at a known TCP address.
On shipped desktop apps, hard-coded ports are a chronic source of support
tickets — corporate VPN agents, Docker Desktop, IDE debuggers, and other
local services regularly squat on common port ranges.

This decision pairs with ADR-0001 (PyInstaller + `externalBin`): Tauri owns
the sidecar lifecycle, so Tauri can also own port selection.

## Alternatives considered

**A. Fixed port** (e.g. `17821`). React calls `http://127.0.0.1:17821/health`
directly. Simple; one fewer IPC roundtrip. Fails if the port is in use.

**B. Tauri-chosen dynamic port + IPC exposure.** Rust binds a free port via
`std::net::TcpListener::bind("127.0.0.1:0")` to discover one, releases it,
then launches the FastAPI binary with `--port <N>`. The chosen port is
exposed to React via a Tauri IPC command `get_backend_port`. React queries
this command once on startup and caches the result.

**C. FastAPI picks the port and writes a control file** under
`%APPDATA%\Chronicler\` that Rust reads and forwards to React via IPC.

## Decision

**B** — Tauri chooses the port; passes it to FastAPI via `--port <N>` CLI
arg; exposes it to React via IPC `get_backend_port`.

## Consequences

- One IPC roundtrip happens at UI startup before any HTTP fetch. Negligible
  cost (< 1 ms locally) and forces us to build the React "wait-for-backend"
  state machine early — which Phase 2 (live transcription) needs anyway
  for its WebSocket URL.
- The "race" between binding port for discovery and FastAPI binding it is
  small but real. Mitigation: bind with `port = 0`, read back the OS-chosen
  port, drop the binding, immediately spawn FastAPI with `--port <N>`. If
  the second bind fails, retry once.
- React must not assume the port until `get_backend_port` resolves. A
  hard-coded fallback would mask startup races; avoid it.
- The same `get_backend_port` command serves later phases:
  - Phase 2 — WebSocket URL for transcript streaming.
  - Phase 4 — post-processing HTTP calls to FastAPI's `/postprocess`.
  - Phase 5 — diarization endpoint URL.

## Verification

S02's acceptance criteria AC2 and AC3 verify this decision: `invoke('get_backend_port')`
returns a number, and the React UI uses it to reach `/health` successfully.

## References

- `tauri-plugin-shell` documentation for sidecar APIs (cross-check via `context7`).
- Tauri IPC command registration in Rust (`#[tauri::command]`).
