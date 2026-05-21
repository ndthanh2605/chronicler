# S01 — Tauri + React shell + minimal IPC

## Intake Report

**Input type:** Spec slice — picking up existing story packet S01  
**Lane:** normal  
**Risk flags:** 2 — cross-platform (Tauri 2 native Windows shell/bundle), weak proof (no existing tests; this story creates the first rung)  
**Hard gate:** no  
**Story:** `docs/stories/phase-0-scaffold/S01-tauri-react-shell.md` — status: `todo`  
**v0 constraint:** clear — S01 explicitly authorizes scaffolding application source folders and package scripts  

**Cleared to proceed:** yes

---

## Context

S01 is Phase 0's first story. No application code exists yet — `frontend/` is empty. The goal
is to scaffold the Tauri 2 desktop shell embedding a Vite-built React UI and prove IPC plumbing
with a trivial ping/pong roundtrip. This establishes the dev loop (hot reload, build, validate)
before any audio or sidecar work begins. S02 will extend the same scaffold with the FastAPI
sidecar; the `ping` Rust handler must stay reusable as a health-check shim.

Environment note: working directory is WSL2. `cargo check` and lint/typecheck run fine in WSL2.
The manual smoke test (tauri dev → Chronicler window → Ping button) requires a Windows-native
terminal or WSLg — cannot run from bare WSL2 without display forwarding. The plan accounts for
this: validate:quick is WSL2-runnable; smoke test is a human attestation step.

---

## Implementation Plan

### Phase A — Harness admin

1. **Create workpad sibling**  
   `docs/stories/phase-0-scaffold/S01-tauri-react-shell.workpad.md`  
   Sections: Plan, Notes, Confusions. Record this plan in the Plan section.

2. **Create branch + sync**  
   Branch: `story/s01-tauri-react-shell`  
   Run `harness-git-pull` skill: `git fetch origin && git log HEAD..origin/main --oneline`,  
   then merge or confirm up-to-date. Record sync result (date, status, HEAD SHA) in workpad Notes.

3. **Transition story status**  
   Update `docs/stories/phase-0-scaffold/S01-tauri-react-shell.md` status field: `todo` → `in_progress`.  
   Gate check: workpad created ✓, pull skill run ✓, implementation beginning ✓.

### Phase B — ADR 0003: Package manager

4. **Create `docs/decisions/0003-package-manager.md`**  
   Decision: **pnpm**.  
   Rationale: Tauri 2 official docs default to pnpm; disk-efficient (hardlinked store); fast
   installs; `pnpm create tauri-app` is the canonical Tauri 2 bootstrap path. Must land before
   any package scripts (per S01 story notes).

### Phase C — Tauri 2 + React scaffold

5. **Prerequisite check** (run before scaffold, flag as blocked if missing):
   ```
   pnpm --version   (≥8)
   node --version   (≥18)
   cargo --version
   rustup --version
   ```

6. **Initialize Tauri 2 project in `frontend/`**  
   ```bash
   cd frontend
   pnpm create tauri-app@latest . --template react-ts --manager pnpm \
     --identifier com.chronicler.app
   ```
   If the CLI requires interactive prompts not suppressed by flags, run interactively and confirm:
   - Template: React + TypeScript
   - Package manager: pnpm
   - App name: Chronicler
   - Identifier: com.chronicler.app

   This produces the canonical Tauri 2 layout:
   ```
   frontend/
   ├── src/                   # React TypeScript (App.tsx, main.tsx)
   ├── src-tauri/
   │   ├── src/lib.rs         # Tauri commands
   │   ├── src/main.rs        # entry point
   │   ├── Cargo.toml
   │   ├── tauri.conf.json
   │   └── capabilities/
   ├── index.html
   ├── vite.config.ts
   ├── tsconfig.json
   └── package.json
   ```

7. **Customize `frontend/src-tauri/tauri.conf.json`**  
   Ensure: `productName: "Chronicler"`, `title: "Chronicler"` in the window config,
   `identifier: "com.chronicler.app"`. The scaffold may already set these if flags were passed.

### Phase D — Application code

8. **Add `ping` command in `frontend/src-tauri/src/lib.rs`**  

   ```rust
   #[tauri::command]
   fn ping() -> String {
       "pong".to_string()
   }
   ```

   Register in `tauri::Builder`:
   ```rust
   .invoke_handler(tauri::generate_handler![ping])
   ```

   Add unit test in the same file:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn ping_returns_pong() {
           assert_eq!(ping(), "pong");
       }
   }
   ```

   Keep the function signature `fn ping() -> String` — S02 will extend this handler to call
   FastAPI `/health` without renaming the IPC command.

9. **Replace `frontend/src/App.tsx`** with the Chronicler shell UI:
   - Title `<h1>Chronicler</h1>`
   - "Ping" button that calls `invoke<string>("ping")` from `@tauri-apps/api/core`  
     (Tauri 2 import path — not the v1 `@tauri-apps/api/tauri`)
   - State to hold the response; render it below the button when present
   - For error state: display the error string in the UI (non-blocking; shapes the pattern for
     future IPC calls where silent failure would obscure real problems)

   Implement `handlePing` as a complete async function — do not leave a placeholder requiring
   human completion before the story can progress.

### Phase E — Validation tooling (AC4)

10. **Add npm scripts to `frontend/package.json`**:
    ```json
    "lint": "eslint src --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "typecheck": "tsc --noEmit"
    ```

11. **Create `scripts/validate-quick.sh`**:
    ```bash
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="$(cd "$(dirname "$0")/.." && pwd)"

    echo "=== ESLint ==="
    (cd "$ROOT/frontend" && pnpm run lint)

    echo "=== TypeScript typecheck ==="
    (cd "$ROOT/frontend" && pnpm run typecheck)

    echo "=== Rust unit tests ==="
    (cd "$ROOT/frontend/src-tauri" && cargo test)

    echo "=== validate:quick passed ==="
    ```
    Run `chmod +x scripts/validate-quick.sh`.

    Note: `cargo test` (not just `cargo check`) is used so the `ping_returns_pong` unit test
    also runs in validate:quick. The story's AC4 literally says `cargo check`; update AC4 in
    the story packet to say `cargo test` before committing, so the next agent sees no mismatch.

### Phase F — TEST_MATRIX + Evidence

12. **Update `docs/TEST_MATRIX.md`**  
    Rows TM-001 and TM-002: status `planned` → `in_progress`.

13. **Update S01 Evidence section** after validation passes:
    - Workpad path filled in
    - `validate:quick` log pasted
    - Rust unit test output pasted
    - PR URL (after `harness-git-push`)
    - Manual smoke screenshot — AC1/AC2 (human step, Windows env): `pnpm tauri dev`, click Ping
    - Build artifact attestation — AC3 (human step, Windows env): `pnpm tauri build` exits 0
      and bundle appears under `frontend/src-tauri/target/release/bundle/`

    ACs 1 and 3 both require Windows — two of four ACs are human attestation steps, not
    agent-runnable. Confirm both before moving to `human_review`.

---

## Verification

**Agent-runnable (WSL2-safe):**

```bash
# 1. Rust unit test
cargo test --manifest-path frontend/src-tauri/Cargo.toml
# Expect: test ping_returns_pong ... ok

# 2. validate:quick (lint + typecheck + cargo test)
bash scripts/validate-quick.sh
# Expect: exit 0
```

**Human attestation required (Windows terminal or WSLg):**

```bash
# 3. AC1/AC2 smoke — tauri dev
cd frontend && pnpm tauri dev
# Expect: native window titled "Chronicler", Ping button visible, hot reload works
# Click Ping → "pong" appears in UI; take screenshot for Evidence

# 4. AC3 — tauri build
cd frontend && pnpm tauri build
# Expect: exit 0; artifact exists under frontend/src-tauri/target/release/bundle/
# Paste bundle path in Evidence (code signing is out of scope — unsigned is fine)
```

Story can move to `human_review` only after all four pass and PR is opened.

---

## Files Modified/Created

| File | Action |
|---|---|
| `docs/stories/phase-0-scaffold/S01-tauri-react-shell.workpad.md` | create |
| `docs/stories/phase-0-scaffold/S01-tauri-react-shell.md` | edit (status update) |
| `docs/decisions/0003-package-manager.md` | create |
| `frontend/` (full scaffold) | create (via `create tauri-app`) |
| `frontend/src/App.tsx` | modify (Ping UI, replacing scaffold default) |
| `frontend/src-tauri/src/lib.rs` | modify (ping command + unit test) |
| `frontend/src-tauri/tauri.conf.json` | modify (productName, title, identifier) |
| `frontend/package.json` | modify (lint + typecheck scripts) |
| `scripts/validate-quick.sh` | create |
| `docs/TEST_MATRIX.md` | edit (TM-001, TM-002 status) |

---

## Risks / Open Items

- **WSL2 smoke test**: `pnpm tauri dev` needs a display. Plan doc will note this; human provides
  the screenshot attestation. `validate:quick` (cargo test + lint + typecheck) is fully WSL2-safe.
- **`create tauri-app` interactivity**: If the CLI requires prompts, run interactively; capture
  choices in workpad Notes.
- **Tauri 2 API import**: `@tauri-apps/api/core` (not v1 `@tauri-apps/api/tauri`) — the scaffold
  template should handle this; verify in App.tsx.
- **ADR 0004 gap**: `HARNESS.md` references `docs/decisions/0004-execution-state-machine.md` but
  it doesn't exist. Not in scope for S01 — add note to `HARNESS_BACKLOG.md` during execution.
- **Branch naming convention**: `story/s01-tauri-react-shell` is invented (HARNESS.md has no
  documented convention). Add a HARNESS_BACKLOG entry to propose a convention (e.g.
  `story/<id>-<slug>`) so future agents don't each invent their own.
