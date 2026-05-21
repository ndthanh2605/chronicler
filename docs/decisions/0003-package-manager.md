# ADR 0003 — Package Manager: pnpm

**Date:** 2026-05-20  
**Status:** accepted  
**Story:** S01 — Tauri + React shell

## Decision

Use **pnpm** as the Node.js package manager for the `frontend/` workspace.

## Rationale

- Tauri 2 official documentation and `create tauri-app` scaffold default to pnpm.
- Hardlinked content-addressable store: avoids duplicate installs across workspaces,
  faster on repeat installs.
- Strict by default: phantom dependencies are not resolvable, catching missing explicit
  deps early.
- `pnpm create tauri-app` is the canonical bootstrap command; using a different manager
  would require extra flags and produce a less-standard scaffold.

## Consequences

- All scripts in `package.json`, `README`, and `validate:quick` use `pnpm run …`.
- Contributors need pnpm ≥ 8 installed (`npm i -g pnpm` or via corepack).
- S02 and all future frontend stories use pnpm; do not introduce npm or yarn scripts.
