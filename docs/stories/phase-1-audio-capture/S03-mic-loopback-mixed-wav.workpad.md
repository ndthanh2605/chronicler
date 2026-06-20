# S03 Workpad — Mic + loopback capture into a mixed 16 kHz mono WAV

## Status

`in_progress`

## Base / sync

2026-06-20 (impl session): Docs bundle landed to `main` via PR #4 (`f6f68dd`,
recorded `0331c8b`). Implementation branch `story/s03-mic-loopback-mixed-wav`
re-created fresh off `main` (in sync with `origin/main`, 0/0). Status
transitioned `todo → in_progress`. Pre-existing uncommitted `M CLAUDE.md` and
`?? pnpm-lock.yaml` are unrelated to S03 and deliberately kept out of S03
commits.

**Step 0 resolved — meetily reference LOCATED** at
`~/Repos/meetily/frontend/src-tauri/`. Key finding: meetily's
`src/audio/devices/platform/windows.rs` is **cpal-based** (forked cpal
`rev=51c3b43` with loopback), *not* raw `windows-rs` WASAPI. Since **ADR-0005
mandates raw `windows-rs`**, the reference split is:
- **Mixer / ring-buffer algorithm** → meetily `src/audio/pipeline.rs`
  (`AudioMixerRingBuffer`) is a valid language-agnostic reference.
- **Raw WASAPI loopback activation** (`AUDCLNT_STREAMFLAGS_LOOPBACK`, COM
  activation, `GetBuffer`/`ReleaseBuffer`) → meetily does NOT use the raw API,
  so use **MS WASAPI docs** (microsoft-learn / context7) as primary, per the
  execplan Step 0 fallback clause.

**Environment constraint (impl session):** this WSL host has only the
`x86_64-unknown-linux-gnu` Rust target; `cargo test` builds for **Linux**.
Pure modules (`meeting_id`, `wav_writer`, `mixer`, VU math) are TDD'd and
verified green here; WASAPI native code (steps 4–7) is written `cfg(windows)`-
gated and can only be built/smoke-tested by a human on Windows.

---

## Implementation progress (2026-06-20)

Executed execplan steps 1→8 TDD-first. Two commits on
`story/s03-mic-loopback-mixed-wav`:

- **`3b85bfc`** — pure audio core (steps 1–3, verifiable on Linux):
  `meeting_id` (UUID v7), `wav_writer` (header + finalize + sentinel repair),
  `mixer` (timestamp alignment, downmix, linear resample, silence-on-gap,
  bounded drain, per-stream peak), `vu` (peak/RMS + `VuLevels` payload).
  **23 unit tests, all green**; `windows` crate target-gated so the cross-
  platform gate stays green off-Windows.
- **`<second commit>`** — Windows native + controller seam + React UI
  (steps 4–7).

### Verified on this Linux host
- `cargo test --lib`: **24 tests pass, pristine** (23 audio + S02's
  `pick_free_port`). Satisfies **AC7** — the mixer + WAV-writer tests run
  inside `validate:quick` (which already invokes `cargo test`; **no script
  change was needed** for step 8).
- `AudioController` + `start_recording`/`stop_recording` IPC compile on Linux
  with `wasapi` gated out and bodies returning a clear "Windows-only" error
  (the advisor's platform-seam discipline).
- React UI (`RecordControls.tsx` + `App.tsx` wiring): **eslint + tsc clean**.
  VU bars + Record/Stop wired to the `vu-levels` Tauri event and the two IPC
  commands.

### BUILD-PENDING on Windows (cannot be compiled or smoke-tested here)
- `audio/wasapi/{mod,capture,loopback}.rs` — `#[cfg(windows)]`, **never
  compiled**. Written against the **MS Core Audio "Capturing a Stream"**
  pattern (polling drain — deliberately *not* event-driven, since WASAPI
  loopback doesn't reliably support `AUDCLNT_STREAMFLAGS_EVENTCALLBACK`) and
  `windows` crate 0.58. **Expect to fix a few `windows`-crate signature
  details on the first Windows build** (Activate/GetService generics, GetBuffer
  out-params, mix-format float assumption, optional `CoTaskMemFree` of the mix
  format). The *algorithm* is the load-bearing part and follows the docs.
- All of **AC1–AC6, AC8** need the Windows GUI + real devices — see
  `validation.md`. None can be agent-verified.

### Step 0 reference note
meetily's `windows.rs` is **cpal-based**, so it was used only for the
ring-buffer *algorithm* shape (`pipeline.rs`); the raw `windows-rs` WASAPI
calls were grounded in **MS WASAPI docs** via microsoft-learn (execplan Step 0
fallback). ADR-0005's "raw windows-rs" decision is unchanged.

### Resampler decision (ADR-0005 left crate open at impl)
Chose a **hand-rolled linear resampler** (not `rubato`) for Phase 1: keeps the
pure modules dependency-light and deterministic for unit tests; adequate for
16 kHz speech. Swappable for `rubato` later without touching alignment logic.

### Next steps (Windows machine required)
1. `pnpm tauri dev` on Windows → resolve any `windows`-crate signature errors
   in `audio/wasapi/`.
2. Smoke-test AC1–AC6, AC8 per `validation.md`; attach evidence (VU
   screenshot, WMP playback, `ffprobe`, tick-injection sync) to the story.
3. Thread-budget drift (§7 of design: 3 threads) still needs architect
   ratification before `CLAUDE.md`/`ARCHITECTURE.md` update (logged in
   `HARNESS_BACKLOG.md`).

---

2026-06-15 (update): **S02 has landed** — user-confirmed the Windows GUI smoke
test (AC1/2/3/6), PR #2 squash-merged to `main` as `a6073bf`. This branch was
then rebased `--onto main cef8f5d` (dropping the redundant S02 commits) and
force-pushed; it now sits on a clean post-S02 `main`. PR #3 auto-closed when the
S02 base branch was deleted → superseded by **PR #4** (base `main`, ready).
The S02-land gate is **cleared**; S03 implementation can begin.

Originally: branch was based on the S02 tip `cef8f5d` (off S02's work, not
pre-S02 `main`) so the harness state it sits on (ADR-0004, TM-004) stayed
consistent with what S03's docs assume.

## Notes

2026-06-15: Authored the full S03 docs bundle this session — initiative
note, story packet (`S03-mic-loopback-mixed-wav.md`, 8 ACs), and
`S03-mic-loopback-mixed-wav.{design,execplan,validation}.md` — via
subagent-driven drafting (Tasks 1 and 2 of this docs effort). This session
(Task 3) adds ADR-0005, this workpad, TM-005..008, and two
`HARNESS_BACKLOG.md` entries.

**No Rust code was written or attempted.** S03 implementation is gated on
S02 landing to `main` (see "Next step" below).

## What this session did

- Authored `docs/decisions/0005-audio-capture-library.md` (windows-rs WASAPI
  decision, ADR-0001-shaped).
- Authored this workpad.
- Appended TM-005..008 to `docs/TEST_MATRIX.md` (status `planned`, story
  `S03`).
- Appended two entries to `docs/HARNESS_BACKLOG.md`: extracting a high-risk
  story template, and reconciling the Rust thread-budget drift.

## Decisions made

- **ADR-0005**: `windows-rs` WASAPI direct (not `cpal`, not vendoring
  meetily) — see `docs/decisions/0005-audio-capture-library.md`.
- Three design sub-decisions resolved in `S03-mic-loopback-mixed-wav.design.md`
  §5:
  - §5.1 — two-stream sync model: **shared ring buffer + real-time mix**.
  - §5.2 — WAV write policy: **streaming append with header fixup** (write
    PCM frames continuously, back-patch the WAV header's size fields on
    Stop/finalize).
  - §5.3 — partial-file recovery: **auto-repair on next launch** (no DB
    bookkeeping; detect a partial WAV header at startup and fix it in
    place).
- Branch was deliberately based on the S02 tip (`cef8f5d`), not pre-S02
  `main`, so this story's harness files (ADR numbering, TM row numbering,
  backlog) stay consistent with S02's in-flight state rather than diverging
  and requiring a reconciliation merge later.

## Open risks / not attempted

1. **meetily reference not located on this machine.** ADR-0005 and the
   execplan treat meetily's `windows.rs` / `AudioMixerRingBuffer` as a
   read-only reference implementation, but it has not been confirmed to
   exist in this environment. Execplan Step 0 must locate it (e.g. clone or
   find a local copy) or fall back to MS WASAPI docs + the Chronicler design
   spec if unavailable.
2. **Thread-budget drift.** S03's design (§7) requires 3 Rust threads (mic
   capture, loopback capture, mixer/writer) vs. the 2 documented in
   `CLAUDE.md` ("1 capture thread + 1 VAD/chunker thread") — and Phase 1 has
   no VAD thread yet. This drift is logged in `docs/HARNESS_BACKLOG.md` and
   needs architect ratification before `CLAUDE.md` / `docs/ARCHITECTURE.md`
   are updated.
3. **All S03 ACs require a Windows GUI and real audio devices** (mic +
   render-device loopback, VU meters, WAV playback/ffprobe). None of AC1–AC8
   can be verified by an agent alone — manual smoke tests are required per
   `validation.md`.
4. No Rust code exists yet for this story — implementation has not started
   (intentionally; gated below).

## What was not attempted

- Any Rust implementation (audio capture, mixer, WAV writer, VU meter
  events, React UI for the meters).
- Locating/confirming the meetily reference (execplan Step 0).
- Any build, test, or `cargo`/`pnpm` command — this session is docs-only.

## Next step for a cold-start agent

1. **Land S02 first.** Run S02's Windows GUI smoke test (see
   `docs/stories/phase-0-scaffold/S02-fastapi-sidecar-health-sqlite.workpad.md`
   "Next step for a cold-start agent"): verify AC1 (teardown — no orphan
   `chronicler-backend-*.exe`), AC2 (`get_backend_port`), AC3 (Ping renders
   `/health` JSON), AC6 (Backend unavailable UI state). Check off those ACs
   and land PR #2 to `main`.
2. **Rebase** `story/s03-mic-loopback-mixed-wav` onto post-S02 `main`.
3. **Confirm or locate the meetily reference** per execplan Step 0; fall
   back to MS WASAPI loopback docs + the Chronicler design spec if it is
   unavailable.
4. **Execute `execplan.md` steps 1→9**, TDD-first: write `wav_writer` and
   `mixer` unit tests before the implementation (per
   `superpowers:test-driven-development`).
5. **Record evidence** in `S03-mic-loopback-mixed-wav.md`'s Evidence
   section as each AC is verified (most require a human at a Windows GUI
   with real audio devices — see `validation.md`).
6. Transition story status `todo` → `in_progress` only after the pull
   (rebase) and this workpad are updated to reflect the new base.
