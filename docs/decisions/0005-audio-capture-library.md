# ADR-0005 — Audio capture library

- **Status:** Accepted
- **Date:** 2026-06-15
- **Initiative:** `docs/stories/phase-1-audio-capture/initiative.md`
- **Stories:** `S03`

## Context

Phase 1 needs simultaneous WASAPI mic + render-device loopback capture on
Windows, mixed to 16 kHz mono PCM (the format faster-whisper needs in
Phase 2). Loopback requires the WASAPI loopback flag
(`AUDCLNT_STREAMFLAGS_LOOPBACK`) on the render device; shared-mode WASAPI
delivers device-native format (commonly 44.1/48 kHz float), so resampling is
unavoidable. Choice of capture library shapes how much control we have over
the loopback flag and format negotiation.

## Alternatives considered

**A. Raw `windows-rs` WASAPI** — direct `IAudioClient`/`IAudioCaptureClient`,
full control of the loopback flag, event-mode timing, and format
negotiation; more code.

**B. `cpal`** — cross-platform abstraction; less boilerplate but hides
sample-rate negotiation and historically awkward loopback support on
Windows.

**C. Vendor meetily's audio code** — fastest start, but imports an
unfamiliar dependency surface and license/maintenance burden; better used as
a *reference* than vendored.

## Decision

**A — use `windows-rs` WASAPI directly**, treating meetily's `windows.rs`/
`AudioMixerRingBuffer` as a read-only reference, not a vendored dependency.
Pair with the sample-rate strategy: **capture native, then resample +
downmix to 16 kHz mono in the mixer** (per-stream resampler before mix); the
resampler crate (e.g. `rubato`) is selected at implementation in execplan
Step 1. Rationale: full control of the loopback flag and event-mode timing;
avoids hiding format negotiation behind cpal; keeps the dependency surface
understood.

## Consequences

- More WASAPI boilerplate (COM init, activation, `GetBuffer`/`ReleaseBuffer`);
  we own resampling/downmix correctness and its unit tests.
- Windows-only (acceptable — Chronicler is Windows-only).
- Design-level decisions (two-stream sync, WAV flush, partial recovery) live
  in `S03-mic-loopback-mixed-wav.design.md`.

## Verification

Validated by S03 ACs (AC1 simultaneous open/clean stop; AC2 valid mixed
16 kHz mono WAV; AC5 loopback silence) and TM-005..007.

## References

- meetily `frontend/src-tauri/src/audio/devices/platform/windows.rs` +
  `AudioMixerRingBuffer` (reference; confirm availability per execplan
  Step 0).
- WASAPI loopback recording (MS docs).
- `docs/superpowers/specs/2026-05-18-chronicler-design.md` — Chronicler
  design spec.
