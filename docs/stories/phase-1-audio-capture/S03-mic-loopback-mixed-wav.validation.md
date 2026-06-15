# S03 — Validation: mic + WASAPI loopback → mixed WAV

Validation plan for
`docs/stories/phase-1-audio-capture/S03-mic-loopback-mixed-wav.md`. Covers
the manual Windows-GUI smoke matrix, automated Rust coverage, the
reference-machine acceptance run, and the named measurement methods for AC4
(±50 ms sync) and AC6 (header finalize + partial recovery). Every row maps to
an AC and, where one exists, a TM row (TM-005..008).

> Reality check (mirrors S02): the load-bearing behaviors here — real WASAPI
> capture, audible mixed playback, moving VU bars, silent-device clocking —
> can **only be verified by a human at a Windows GUI**. The automated Rust
> tests cover the pure logic (mixer, WAV header) and a shape-level
> integration check; they do **not** prove end-to-end capture.

## 1. Automated coverage (`validate:quick`, AC7)

Wired into the `validate:quick` gate (execplan step 8):

| Test | Module | Asserts | AC |
|---|---|---|---|
| WAV header write | `wav_writer.rs` | RIFF/WAVE/fmt /data correct; 16000 Hz, 1 ch, 16-bit | AC2, AC7 |
| WAV finalize | `wav_writer.rs` | RIFF + data sizes patched correctly for known PCM length | AC2, AC6, AC7 |
| WAV repair (sentinel) | `wav_writer.rs` | sentinel file detected unfinalized; repaired to last whole frame; result is a valid WAV | AC6, AC7 |
| Mixer downmix | `mixer.rs` | two known inputs → expected summed/clamped mono output | AC2, AC7 |
| Mixer tick-injection sync | `mixer.rs` | impulse offset in mixed output ≤ ±50 ms (synthetic streams) | AC4, AC7 |
| Mixer silence synthesis | `mixer.rs` | timestamp gap → silent frames at expected rate, no stall, stays aligned | AC5, AC7 |
| Mixer bounded memory | `mixer.rs` | long synthetic feed does not grow buffers unboundedly | AC4, AC7 |

**Optional Rust integration test** (capture → WAV shape): drive a short
capture and assert the output WAV's sample rate / channels / bit depth via
`ffprobe` (TM-005 shape-level; does not replace manual capture).

## 2. Manual smoke matrix (Windows GUI)

Run on the reference machine (§3). Attach evidence to the story.

| # | Step | Expected | AC | TM |
|---|---|---|---|---|
| M1 | Click **Record**, speak + play a YouTube clip 60 s, **Stop** | A `<meeting-id>.wav` is written under `%APPDATA%\Chronicler\audio\` | AC1, AC2 | TM-005/007 |
| M2 | Open the WAV in Windows Media Player | Plays back; **both** voice and YouTube audibly mixed | AC2 | TM-007 |
| M3 | `ffprobe` the WAV | Codec PCM, **16000 Hz, 1 channel**, 16-bit; duration within **±100 ms** of recording length | AC2 | TM-007 |
| M4 | Watch the UI while recording | **Two** independent VU bars (mic, loopback) update at ≥10 Hz; **screenshot** with both moving | AC3, AC8 | TM-008 |
| M5 | Mute mic / pause system audio mid-record | The corresponding bar drops to silence within **200 ms** | AC3 | TM-008 |
| M6 | Record with **system audio paused** the whole time | Loopback still writes silent PCM at the expected rate; recording does **not** stall; mic audio still present | AC5 | TM-006 |
| M7 | Stop after a normal recording | WAV header finalized — RIFF/data sizes correct (re-check with `ffprobe`/M3) | AC6 | — |
| M8 | Force-kill the app mid-recording, relaunch | The partial WAV is auto-repaired on next launch and plays back to the last whole frame | AC6 | — |
| M9 | ≥5-minute recording | Completes with no overrun/underrun, no audible dropouts, no unbounded memory growth (§3) | AC4 | — |

## 3. Reference-machine acceptance

- **Machine**: Ryzen 5 3600X, 6C/12T (the project's reference hardware,
  `docs/ARCHITECTURE.md` / design plan thread budget).
- During the **≥5-minute** run (M9): record **CPU usage and memory** for the
  Tauri/Rust process (Task Manager / `tracing` counters). Expected: stable
  memory (bounded ring buffer + streaming WAV write — design §5.1/§5.2),
  comfortable CPU headroom (no faster-whisper in Phase 1). No buffer
  overrun/underrun warnings logged.
- Confirms AC4's "survives ≥5 minutes without overrun, underrun, or growing
  memory."

## 4. ±50 ms two-stream sync (AC4 — tick-injection method)

Two layers, same principle:

- **Automated (in-process)**: the `mixer.rs` tick-injection unit test (§1) —
  synthetic mic + loopback streams each carry a known impulse at a known
  timestamp; assert the offset between the two impulses in the mixed output
  is ≤ ±50 ms. Runs in `validate:quick`.
- **Manual (end-to-end, on a real recording)**: inject a **shared physical
  tick** audible to both streams — e.g. a sharp clap/beep that the mic hears
  acoustically and that is simultaneously played through the render device
  (so loopback captures it). In the final mixed WAV, locate the impulse as
  heard via mic vs. via loopback and measure the offset; it must be ≤ ±50 ms
  over a ≥5-minute recording. (Because both land in one mixed mono stream,
  the manual check confirms the *combined* timeline did not drift beyond the
  window.)

## 5. Partial-file recovery (AC6 — forced-kill method)

- WAV-header **unit test** (§1) proves the sentinel → detect → repair logic
  in isolation.
- **Manual** forced-kill check (M8): kill the process mid-recording (the
  header still carries the placeholder sentinel), relaunch, confirm the
  startup recovery scan repaired the file to a valid, playable WAV at the
  last whole frame.

## 6. Coverage map (AC → proof)

| AC | Automated | Manual | TM |
|---|---|---|---|
| AC1 open both / clean stop, no orphan handles | int. test (shape) | M1, M2 + handle check on Stop | TM-005 |
| AC2 valid 16 kHz mono mixed WAV | WAV header/downmix tests | M1, M2, M3 (`ffprobe`) | TM-007 |
| AC3 two VU bars ≥10 Hz, silence ≤200 ms | — | M4, M5 | TM-008 |
| AC4 ≥5 min stable, ±50 ms sync | mixer sync + bounded-mem tests | M9 + §3 + §4 tick injection | — |
| AC5 silent render device still clocked | mixer silence test | M6 | TM-006 |
| AC6 header finalize + partial recovery | WAV finalize + repair tests | M7, M8 (§5) | — |
| AC7 Rust unit tests in `validate:quick` | §1 suite green | — | — |
| AC8 60 s smoke + VU screenshot | — | M1, M4 (screenshot) | TM-008 |

## 7. Human-only verification (cannot be automated)

- Audible confirmation that **both** streams are present and mixed (M2).
- VU bars visually moving + dropping to silence (M4, M5) and the evidence
  screenshot (M4 / AC8).
- Real WASAPI loopback against the actual default render device, including
  the "Stereo Mix"/virtual-cable constraint (M6).
- Forced-kill → relaunch recovery behavior on a real `%APPDATA%` file (M8).
- ≥5-minute stability and the end-to-end physical tick-injection sync
  measurement (M9, §4).
