# Initiative — Phase 1: Audio capture

## Goal

Pressing **Record** captures mic + WASAPI loopback (system audio) into a
single mixed **16 kHz mono PCM WAV** under
`%APPDATA%\Chronicler\audio\<meeting-id>.wav`, with live VU meters for both
streams; **Stop** finalizes the file. No transcription yet.

## Why first

Audio is the hardest platform-bound piece of Chronicler — the WASAPI
loopback flag and two-stream ring-buffer alignment have no precedent in this
codebase. De-risking it before any ML code lands avoids debugging audio
plumbing and model integration at the same time. Phase 0 scaffold is
complete; this initiative establishes the capture foundation every later
phase (live transcription, persistence, post-processing, diarization)
depends on. Reference meetily's `windows.rs` and `AudioMixerRingBuffer` for
the WASAPI loopback and mixing shape.

## Candidate stories

| ID  | Title | Status | Lane | Depends on |
|-----|---|---|---|---|
| S03 | Mic + WASAPI loopback captured into a single mixed WAV with live VU meters | `todo` | high-risk | S02 |

Phase 1 is one vertical slice = one story (S03). If execution surfaces a
natural split (e.g. capture vs. UI metering), record it as drift in
`docs/HARNESS_BACKLOG.md` rather than pre-inventing S04–S06.

## Affected product docs

- `docs/ARCHITECTURE.md` — Tauri/Rust audio-capture ownership (mic +
  loopback open/close, two-stream mixing, continuous WAV write path) needs
  to move from planned to implemented once S03 lands.
- A user-facing setup note is needed: WASAPI loopback requires "Stereo Mix"
  enabled or a virtual audio cable on Windows (a known constraint already
  recorded in `docs/ARCHITECTURE.md`, but not yet surfaced to end users).
- No recording-UI product contract doc exists yet. Flag during S03 whether
  one should be added under `docs/product/` to describe the Record/Stop
  flow and VU meter behavior, or whether the story's ACs are sufficient on
  their own for Phase 1.

## Design decisions (recorded in `docs/decisions/`)

- ADR-0005 — Audio capture library (`windows-rs` vs `cpal` vs vendoring
  meetily's audio module).

Additional design-level decisions are captured in S03's `design.md` rather
than as ADRs, since they are implementation-shape choices scoped to this
story:

- Two-stream synchronization model (mic vs. loopback clock drift handling).
- WAV flush / header-fixup policy (when the RIFF header is finalized).
- Partial-file recovery (auto-truncate vs. surface "incomplete" in the UI).

## Validation shape

- Rust unit tests for the ring-buffer mixer and the WAV header writer,
  wired into `validate:quick`.
- Rust integration test — capture pipeline produces a mixed WAV with the
  expected shape (sample rate, channels, bit depth), checked via `ffprobe`.
- Manual Windows GUI smoke — record 60 seconds of voice plus a YouTube
  clip and confirm both are audible in the resulting WAV; VU meters move
  for both streams; a ≥5-minute recording stays stable; two-stream sync is
  within ±50 ms via tick injection.

Test-matrix rows: TM-005..TM-008 (see `docs/TEST_MATRIX.md`).

## Open decisions

- The three `design.md` sub-decisions above (two-stream sync model, WAV
  flush/header-fixup policy, partial-file recovery) — to be resolved during
  S03 design before implementation begins.
- Whether a recording-UI product contract doc is needed under
  `docs/product/`.
- Whether the meetily reference implementation is available on this
  machine; if not, S03 proceeds from the architecture spec and WASAPI docs
  alone.

## Exit criteria

This initiative closes (and Phase 1 is "verified" per `docs/ROADMAP.md`)
when:

- S03 is at `done`.
- Pressing Record on a clean Windows machine captures mic + system audio
  and Stop produces a valid 16 kHz mono mixed WAV at
  `%APPDATA%\Chronicler\audio\<meeting-id>.wav` that plays back with both
  streams audible.
- Two live VU meters update during recording, one per stream.
- A ≥5-minute recording completes without buffer overrun, underrun, or
  unbounded memory growth.
- All S03 acceptance criteria are checked with evidence attached.
- TM-005..TM-008 are marked `implemented` in `docs/TEST_MATRIX.md`.
- `validate:quick` runs green, including the new Rust unit tests.

When this initiative closes, decompose Phase 2 (Live transcription) under a
new initiative note.
