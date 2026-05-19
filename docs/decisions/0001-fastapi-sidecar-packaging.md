# ADR-0001 — FastAPI sidecar packaging strategy

- **Status:** Accepted
- **Date:** 2026-05-19
- **Initiative:** `docs/stories/phase-0-scaffold/initiative.md`
- **Stories:** `S02-fastapi-sidecar-health-sqlite`

## Context

Chronicler ships a FastAPI Python sidecar (ASR, post-processing, diarization)
alongside a Tauri 2 desktop shell. End users will not have Python installed
on their Windows machines, so the sidecar must be a self-contained binary.

For Phase 0 we are wiring the smallest credible round-trip — a `/health`
endpoint reached from the React UI through the Tauri shell. The decision
to make is **how** that sidecar process is built and spawned.

The choice affects:

- the dev loop (one command vs two; auto-restart vs manual);
- whether dev and release have the same spawn path;
- the size of the install artifact;
- how Phase 6 (Onboarding & packaging) closes out.

## Alternatives considered

**A. PyInstaller binary registered as Tauri `externalBin`** — same binary in
dev and release. Tauri spawns the binary as a child process and manages its
lifecycle.

**B. Split dev/release path** — in dev, the developer runs `uvicorn` in a
second terminal (or a `Makefile` task) while `tauri dev` runs separately;
in release, PyInstaller produces the same binary as A, registered as
`externalBin`.

**C. Rust spawns a Python subprocess directly** — no `externalBin`. Rust
uses `std::process::Command` (or `tauri-plugin-shell`'s `Command`) to
start/stop uvicorn. PyInstaller is still required for release.

## Decision

**A** — PyInstaller binary, registered as Tauri `externalBin`, used in both
dev and release.

## Consequences

- Dev requires building the PyInstaller binary at least once per backend
  change. A `pnpm dev:backend` (or equivalent) script must trigger PyInstaller
  before `tauri dev`. We accept this cost in exchange for parity.
- The shipped Windows installer grows by ~30–40 MB for the bundled Python
  runtime. Acceptable for a local-first desktop product.
- Phase 6 (Onboarding & packaging) inherits the packaging mechanism instead
  of building it. Fewer end-of-roadmap surprises.
- Lifecycle is owned by Tauri: closing the window terminates the sidecar.
  Story `S02` AC1 verifies no orphaned processes remain.
- Hot-reload of Python code is **not** automatic in dev. Backend changes
  require a manual rebuild. We accept this; a watcher script can land in a
  later harness improvement if friction surfaces.

## Verification

S02's acceptance criteria AC1 verifies this decision end-to-end (sidecar
spawns, communicates, terminates cleanly).

## References

- Tauri 2 `externalBin` docs (cross-check via `context7` before implementing).
- `~/Repos/stenoai` — Electron + PyInstaller reference; Tauri's equivalent
  applies with minor config differences.
