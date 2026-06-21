# S03 — Mic + WASAPI loopback captured into a single mixed WAV with live VU meters

Pressing Record opens the default mic and the default render device's
WASAPI loopback, mixes both into a single 16 kHz mono PCM WAV, and shows two
live VU meters; Stop finalizes the file cleanly.

## Status

`todo`

## Lane

`high-risk`

Reason: native WASAPI loopback flag, two-stream synchronization, and no
prior audio test harness in this codebase — three independent sources of
platform risk with no precedent to fall back on.

## Depends on

- S02 (FastAPI sidecar + Tauri↔sidecar topology + `%APPDATA%\Chronicler`
  directory creation) — must be landed to `main` before S03 implementation
  code begins, because S03 code branches from post-S02 `main`. This packet
  may be authored before S02 lands; only the *code* is gated.

## Context

- Phase: 1 — Audio capture (`docs/ROADMAP.md`)
- Approved architecture: `docs/superpowers/specs/2026-05-18-chronicler-design.md`
- Initiative: `docs/stories/phase-1-audio-capture/initiative.md`
- Affected product docs: `docs/ARCHITECTURE.md` (Tauri/Rust audio-capture
  ownership); a recording-UI product contract doc may be added — see the
  initiative's open decisions.

This is the first story to record real audio. The Tauri/Rust shell opens
the default Windows microphone and the default render device's WASAPI
loopback simultaneously, mixes the two streams down to a single 16 kHz mono
PCM WAV — the format faster-whisper needs in Phase 2 — and writes it
continuously to `%APPDATA%\Chronicler\audio\<meeting-id>.wav`. Two live VU
meters give the user feedback that both streams are being captured.

This story is high-risk because of the native WASAPI loopback flag, the
need to keep two independently-clocked streams synchronized over long
recordings, and the absence of any prior audio test harness in this repo.
Reference meetily's Windows audio implementation (`windows.rs` and
`AudioMixerRingBuffer`) for the WASAPI loopback and mixing shape.

## Acceptance criteria

- [ ] AC1 — Click **Record** in the React UI → Rust opens the default
  Windows mic and the default render device's WASAPI loopback
  simultaneously; **Stop** terminates both cleanly with no orphan audio
  handles. → maps to `TM-005`.
- [ ] AC2 — A valid 16 kHz mono PCM WAV is written to
  `%APPDATA%\Chronicler\audio\<meeting-id>.wav`; it plays back in Windows
  Media Player and contains both streams audibly mixed. → maps to `TM-007`.
- [ ] AC3 — VU meters in the React UI show two independent level bars (mic
  and loopback) updating at ≥10 Hz while recording, and both drop to
  silence within 200 ms of mute. Levels come from a Tauri **event channel**,
  not from polling the WAV. → maps to `TM-008`.
- [ ] AC4 — Recording survives ≥5 minutes without buffer overrun, underrun,
  or growing memory; the mixer keeps the two streams synchronised to within
  **±50 ms** over the run (measured by injecting a known tick into both
  streams and checking the offset in the final WAV).
- [ ] AC5 — If the default render device has no active stream (silence),
  loopback still writes silent PCM frames at the expected sample rate;
  recording must not stall waiting on the loopback device. → maps to
  `TM-006`.
- [ ] AC6 — Stop finalizes the WAV header (RIFF size, data chunk size)
  correctly; partial files left by a forced kill are either auto-truncated
  on next launch or surfaced in the UI as "incomplete" (the choice is made
  in `design.md`).
- [ ] AC7 — `validate:quick` is extended with Rust unit tests for the
  ring-buffer mixer and the WAV header writer; both green.
- [ ] AC8 — Manual smoke: a 60-second recording of the user's voice plus a
  YouTube clip yields a WAV in which both are present; a screenshot of the
  UI during recording (VU meters moving) is attached to story Evidence.
  → maps to `TM-008`.

## Validation

- `validate:quick` — extended with Rust unit tests for the ring-buffer
  mixer and the WAV header writer (AC7).
- Manual smoke + Rust integration test — AC1 and AC5 (TM-005 / TM-006).
- Manual playback + `ffprobe` — AC2 (TM-007).
- Manual smoke + screenshot — AC3 and AC8 (TM-008).
- Tick-injection sync measurement — AC4 (±50 ms two-stream sync over a
  ≥5-minute recording).
- WAV-header unit test + manual forced-kill check — AC6.

## Out of scope

- Transcription, VAD, 5-second chunking, HTTP POST `/transcribe`, WebSocket
  token streaming — all Phase 2.
- SQLite `meetings` / `transcripts` rows — Phase 3.
- Device-selection UI beyond system defaults.
- Non-Windows audio backends.

## Evidence

To be filled in during execution:

- Workpad: `S03-mic-loopback-mixed-wav.workpad.md` (sibling of this file)
- PR: https://github.com/ndthanh2605/chronicler/pull/4 (docs bundle; supersedes #3, which auto-closed on S02 branch deletion)
- Docs bundle merged: PR #4 at `f6f68dd` on 2026-06-16 (planning artifacts only; story stays `todo` — implementation pending, tracked by a future code PR)
- Implementation (branch `story/s03-mic-loopback-mixed-wav`, 2026-06-20/21):
  - `3b85bfc` — pure audio core (meeting_id, wav_writer, mixer, vu) + 23 TDD unit tests
  - `33dc0f1` — WASAPI capture/loopback (`#[cfg(windows)]`) + AudioController IPC + React Record/Stop/VU UI
  - `a88a917` — windows-gnu cross-compile-check fixes (whole crate typechecks for the Windows target)
- `validate:quick` log: **green on Linux host** — eslint + tsc clean; `cargo test` 24/24 pass pristine (satisfies **AC7**); backend ruff/pyright clean; pytest 5/5. Windows target additionally `cargo check`-clean (`a88a917`).
- **Pending (Windows GUI + real devices required):** AC1–AC6, AC8 manual smoke per `validation.md`.
- Manual smoke screenshots:
  - VU meters moving during recording: <path>
  - WAV playback in Windows Media Player: <path>
  - `ffprobe` output for the mixed WAV: <paste>
  - Sync-offset measurement (tick injection): <paste>

## Notes for the next agent

- **S02 has landed** to `main` (PR #2 → `a6073bf`, 2026-06-15) and this branch
  has been rebased onto post-S02 `main` — the S02-land gate is now CLEARED, so
  S03 implementation code can begin (branch is already on a clean `main` base).
- Read `design.md` and `execplan.md` first — they resolve the two-stream
  sync model, WAV flush/header-fixup policy, and partial-file recovery
  decisions before any Rust is written.
- Locate and confirm the meetily reference (`windows.rs`,
  `AudioMixerRingBuffer`) is available on this machine before coding; if
  not, fall back to the architecture spec and WASAPI documentation.
