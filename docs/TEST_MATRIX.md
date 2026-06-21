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
| TM-001 | Phase 0 — UI shell | Tauri 2 shell launches and renders the React/Vite page (title + Ping button); hot reload works in `tauri dev`. Proof: manual platform smoke (user-confirmed on Windows 2026-05-21). | implemented | S01 |
| TM-002 | Phase 0 — IPC | Tauri↔React IPC roundtrip: `invoke('ping')` returns the literal `"pong"`. Proof: Rust unit test `ping_returns_pong` passes via `cargo test` (verified 2026-05-21). | implemented | S01 |
| TM-003 | Phase 0 — Sidecar | Tauri spawns the PyInstaller-bundled FastAPI binary as `externalBin`; dynamic port is exposed via IPC `get_backend_port`; React reaches `/health` and renders the JSON payload. Sidecar terminates cleanly on app close. Proof: Rust integration test (spawn + curl) + manual click (user-confirmed on Windows 2026-06-15). | implemented | S02 |
| TM-004 | Phase 0 — SQLite | First-run creates `%APPDATA%\Chronicler\chronicler.db` with the `config(key, value)` table; `/health` round-trips `last_seen_at` (read previous + write new). Survives restart. Proof: Python integration test against a fresh temp dir. | implemented | S02 |
| TM-005 | Phase 1 — Mic capture | WASAPI mic stream opens, format-negotiates (resamples) to 16 kHz mono, streams non-silent frames. Proof: Rust integration test (synthetic/mocked capture client) + manual smoke. Partial (`3b85bfc`): mixer resample/downmix→16 kHz mono unit-tested green in `validate:quick`; native WASAPI capture is `#[cfg(windows)]` build-pending + manual smoke pending. Code merged `d710ffc` (S03, self-approved); manual Windows/device smoke still **deferred**. | planned | S03 |
| TM-006 | Phase 1 — Loopback capture | WASAPI loopback flag opens render-device capture; survives silence (writes silent PCM, no stall). Proof: manual smoke (music → frames; pause → silent frames). Partial (`3b85bfc`): mixer silence-on-gap (no stall) unit-tested green; native loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK`) build-pending + manual smoke pending. Code merged `d710ffc` (S03, self-approved); manual Windows/device smoke still **deferred**. | planned | S03 |
| TM-007 | Phase 1 — Mixed WAV | 60-second recording produces a valid 16 kHz mono WAV; ffprobe reports correct sample rate + duration ±100 ms; both streams audibly mixed. Proof: Rust integration + manual playback. Partial (`3b85bfc`): WAV header/finalize/repair unit-tested green (valid 16 kHz mono PCM shape); end-to-end recording + ffprobe pending Windows build. Code merged `d710ffc` (S03, self-approved); manual Windows/device smoke still **deferred**. | planned | S03 |
| TM-008 | Phase 1 — VU meters | React UI shows two level bars (mic + loopback) updating ≥10 Hz during recording via Tauri events. Proof: manual smoke + screenshot. Partial: React VU bars + `vu-levels` event wiring land green (eslint/tsc); per-stream peak unit-tested in mixer/vu; live update + screenshot pending Windows build. Code merged `d710ffc` (S03, self-approved); manual Windows/device smoke still **deferred**. | planned | S03 |
