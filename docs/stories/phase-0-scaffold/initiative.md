# Initiative — Phase 0: Scaffold

## Goal

Stand up Chronicler's three-process topology (Tauri 2 Rust shell ↔ React UI ↔
FastAPI Python sidecar) and confirm SQLite persistence works on first run,
with no real audio, ASR, post-processing, or diarization.

Phase 0 is the first time application code lands in this repo. Its declared
output (from `docs/ROADMAP.md`):

> Pressing a UI button performs a backend round-trip to a FastAPI `/health`
> endpoint, the Tauri shell embeds the React/Vite UI, and a SQLite file is
> created on first run. No real audio or ASR yet.

## Why first

Every later phase (audio capture, live transcription, persistence,
post-processing, diarization, packaging) depends on the three-process
topology and on the dev/release build loop being trustable. Phase 0
de-risks the platform plumbing before any ML or audio code lands.

## Candidate stories

| ID  | Title | Status | Lane | Depends on |
|-----|---|---|---|---|
| S01 | Tauri + React shell + minimal IPC | `todo` | normal | — |
| S02 | FastAPI sidecar + `/health` + SQLite `config` | `in_progress` | normal | S01 |

## Affected product docs

None yet. Phase 0 produces the first application code; user-facing product
docs land alongside Phase 1+ behaviors (audio capture, recording UI, etc.).

## Design decisions (recorded in `docs/decisions/`)

- ADR-0001 — FastAPI sidecar packaging: PyInstaller binary registered as
  Tauri `externalBin`, used in both dev and release.
- ADR-0002 — Sidecar port discovery: Tauri picks a free port, passes it via
  `--port <N>` to FastAPI, exposes it to React via IPC `get_backend_port`.

Phase 0 explicitly defers:

- Full SQLite schema (`meetings`, `transcripts`, `summaries`) → Phase 3.
- Vast.ai settings UI → Phase 4.
- HF token UI → Phase 5.
- Installer signing → Phase 6.

## Validation shape

- `validate:quick` — first rung lands inside S01; covers lint + typecheck
  (React/TypeScript) and `cargo check` (Rust). Extended in S02 with a
  Python lint/typecheck step.
- Manual platform smoke (Windows) — Tauri shell launches; "Ping" button
  produces the expected response; SQLite file exists at the expected path
  after first run.
- Integration test — `Story 2` adds a Rust-level test that spawns the
  PyInstaller binary, queries `/health`, and verifies the JSON payload.

Test-matrix rows: TM-001..TM-004 (see `docs/TEST_MATRIX.md`).

## Open decisions

None at initiative kick-off. Any new decisions surfaced during S01 or S02
must be recorded as ADRs and linked back to this initiative.

## Exit criteria

This initiative closes (and Phase 0 is "verified" per `docs/ROADMAP.md`) when:

- S01 and S02 are both at `done`.
- A clean Windows machine can run `pnpm tauri dev` (or equivalent), see the
  Tauri shell, click "Ping," and observe a `/health` payload from the
  FastAPI sidecar.
- `%APPDATA%\Chronicler\chronicler.db` exists with the `config` table and
  the `last_seen_at` row written by `/health` survives a full app restart.
- `validate:quick` runs green.
- TM-001..TM-004 are marked `implemented` in `docs/TEST_MATRIX.md`.

When this initiative closes, decompose Phase 1 (Audio capture) under a new
initiative note.
