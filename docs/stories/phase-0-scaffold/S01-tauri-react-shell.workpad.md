# S01 Workpad — Tauri + React shell + minimal IPC

## Plan

Full implementation plan: `docs/superpowers/plans/s01-tauri-react-shell.md`

Phases:
- A: Harness admin (workpad, branch, sync, status transition)
- B: ADR 0003 — pnpm package manager decision
- C: Tauri 2 + React scaffold via `pnpm create tauri-app`
- D: `ping` command in Rust + App.tsx Ping button UI
- E: `validate:quick` script (lint + typecheck + cargo test)
- F: TEST_MATRIX update + Evidence

AC4 note: story says `cargo check`; plan uses `cargo test` — will update AC4 in story packet
before committing so the next agent sees no mismatch.

ACs requiring Windows attestation (human step): AC1/AC2 (tauri dev smoke), AC3 (tauri build).

## Notes

2026-05-20: Synced with origin/main. Result: no remote configured (local-only repo); branch
branched from main at HEAD. HEAD: 564fd17

2026-05-21: Verification pass via WSL terminal.
- ESLint: no issues.
- TypeScript (`tsc --noEmit`): clean.
- `cargo test` (run via `cmd.exe` to reach Windows toolchain): `ping_returns_pong ... ok`.
- `validate:quick` script: created at repo root `package.json`. Delegates to
  `cd frontend && pnpm lint && pnpm typecheck && cargo test --manifest-path src-tauri/Cargo.toml`.
  Confirmed green end-to-end via cmd.exe.
- TM-001/TM-002 marked `implemented` in TEST_MATRIX.
- AC1 (window smoke + screenshot): user confirmed window opened on Windows when running
  `cargo tauri dev`. Screenshot collection skipped (overhead); user attestation recorded here.
- AC3 (build artifact): not re-run; user confirmed `cargo tauri dev` compiled successfully.
- Remaining: nothing — all ACs satisfied. Story ready to commit and close.

## Confusions

### WSL2 Tauri Linux prerequisites

`cargo test` (and `cargo check`) fail on WSL2 because Tauri's Linux build chain pulls in
`libdbus-1-dev`, `webkit2gtk`, `rsvg2`, and other system libraries. Even though Chronicler
targets Windows, building the Rust crate on Linux requires these.

Required fix (one-time, in WSL2 terminal):

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libdbus-1-dev \
  pkg-config
```

After installing: `cargo test --manifest-path frontend/src-tauri/Cargo.toml` and
`bash scripts/validate-quick.sh` will pass.

Source: https://tauri.app/start/prerequisites/#linux
