# Harness Backlog

Structural harness changes that need human confirmation before implementation.

| Item | Raised by | Date | Notes |
|------|-----------|------|-------|
| Extract an `initiative.md` template into `docs/templates/` | Phase 0 planning | 2026-05-19 | Phase 0 inlined its initiative-note structure. Extract once a second initiative exists so the shape isn't over-fitted to one case. |
| Reconcile dangling decision reference in `docs/HARNESS.md` | Phase 0 planning | 2026-05-19 | `HARNESS.md` references `docs/decisions/0004-execution-state-machine.md`, which does not exist. Either create the ADR or remove the reference. Non-blocking. |
| Decide on package manager and capture in ADR | Phase 0 planning | 2026-05-19 | S01 currently assumes `pnpm` in its acceptance criteria. If a different manager is preferred (npm, yarn, bun), record `docs/decisions/0003-package-manager.md` before S01 starts and update S01's AC4. |
| Auto-rebuild backend on Python change in `tauri dev` | ADR-0001 | 2026-05-19 | PyInstaller is not hot-reload friendly. A watcher script that rebuilds and respawns the sidecar on backend source change would remove the main dev-loop friction introduced by ADR-0001. Defer until friction is observed. |
| PyInstaller build automation in CI | S02 planning | 2026-05-19 | Harness v0 excludes CI workflows. Once an authorized story permits CI, the workflow must build the PyInstaller binary before the Tauri build step. |
| Constrain `gitnexus analyze` block injection to `AGENTS.md` only | AGENTS.md refinement | 2026-05-19 | The regenerator currently appends the same `<!-- gitnexus:start … end -->` block to both `AGENTS.md` and `CLAUDE.md`, duplicating content. CLAUDE.md was de-duped manually; if the next `npx gitnexus analyze` re-injects into CLAUDE.md, configure or patch the tool to target `AGENTS.md` only. |
