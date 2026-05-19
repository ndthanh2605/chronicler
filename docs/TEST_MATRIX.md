# Test Matrix

Behavior-to-proof control panel. Each row maps a user- or operator-visible
behavior to a planned or implemented proof. Status values:

- `planned` — row declared; the proof does not exist yet.
- `implemented` — proof exists, runs green, and is wired into a validation
  rung (`validate:quick`, `test:integration`, `test:e2e`, etc.).
- `retired` — the behavior or proof has been removed; row retained for
  history but no longer enforced.

| ID | Area | Description | Status | Story |
|----|------|-------------|--------|-------|
| TM-001 | Phase 0 — UI shell | Tauri 2 shell launches and renders the React/Vite page (title + Ping button); hot reload works in `tauri dev`. Proof: manual platform smoke (screenshot in Evidence). | planned | S01 |
| TM-002 | Phase 0 — IPC | Tauri↔React IPC roundtrip: `invoke('ping')` returns the literal `"pong"`. Proof: Rust unit test on the command handler + manual click. | planned | S01 |
| TM-003 | Phase 0 — Sidecar | Tauri spawns the PyInstaller-bundled FastAPI binary as `externalBin`; dynamic port is exposed via IPC `get_backend_port`; React reaches `/health` and renders the JSON payload. Sidecar terminates cleanly on app close. Proof: Rust integration test (spawn + curl) + manual click. | planned | S02 |
| TM-004 | Phase 0 — SQLite | First-run creates `%APPDATA%\Chronicler\chronicler.db` with the `config(key, value)` table; `/health` round-trips `last_seen_at` (read previous + write new). Survives restart. Proof: Python integration test against a fresh temp dir. | planned | S02 |
