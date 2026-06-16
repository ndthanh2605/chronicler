# S03 — Design: mic + WASAPI loopback → single mixed 16 kHz mono WAV

Engineering design for story
`docs/stories/phase-1-audio-capture/S03-mic-loopback-mixed-wav.md`.
Resolves the sub-decisions that must be settled before any Rust is written,
and fixes the component layout, data flow, and thread model. Section
anchors track the story's acceptance criteria (AC1–AC8).

> Companion docs: `S03-mic-loopback-mixed-wav.execplan.md` (ordered steps +
> proofs) and `S03-mic-loopback-mixed-wav.validation.md` (smoke matrix +
> automated coverage). The library decision (resampler + WASAPI bindings)
> is recorded separately in **ADR-0005**; this design states the resampling
> *strategy* and points at the ADR for the *crate* choice.

## 1. Scope

In scope (Phase 1):

- Open the default Windows mic and the default render device's WASAPI
  loopback simultaneously on **Record**; tear both down cleanly on **Stop**
  (AC1).
- Mix both streams to a single **16 kHz mono PCM** WAV at
  `%APPDATA%\Chronicler\audio\<meeting-id>.wav` (AC2).
- Emit two independent live VU levels (mic, loopback) to the React UI over a
  Tauri **event channel** at ≥10 Hz (AC3).
- Stay stable and synchronized within ±50 ms over a ≥5-minute run (AC4).
- Keep the loopback timeline advancing when the render device is silent
  (AC5).
- Finalize the WAV header on Stop and auto-repair partials on next launch
  (AC6).

Out of scope: transcription, VAD, chunking, HTTP POST of chunks, WebSocket
token streaming (all Phase 2); the meetings DB (Phase 3); device-picker UI
(default devices only this phase).

## 2. Module layout

New Rust module tree under `frontend/src-tauri/src/audio/`:

```
audio/
  mod.rs          # AudioController: start/stop; owns thread handles + stop signal; public re-exports
  wasapi/
    mod.rs        # IAudioClient activation + shared-mode format negotiation (shared helpers)
    capture.rs    # default-mic capture loop (event-driven GetBuffer/ReleaseBuffer)
    loopback.rs   # default render-device loopback loop (AUDCLNT_STREAMFLAGS_LOOPBACK)
  mixer.rs        # ring buffer; two-stream timestamp alignment; per-stream resample + downmix → 16 kHz mono
  wav_writer.rs   # streaming WAV append + RIFF/data header fixup on Stop; placeholder sentinel on start
  vu.rs           # peak/RMS metering → Tauri events
  meeting_id.rs   # time-ordered id (UUID v7 / ULID) for <meeting-id>.wav
```

`audio/mod.rs` is wired into `frontend/src-tauri/src/lib.rs` exactly as S02
wired the sidecar lifecycle: an `AudioController` held in module-level state,
started/stopped from Tauri IPC commands, and torn down on the window
`Destroyed` event (see §6).

## 3. Capture layer (AC1, AC5)

- **API**: `windows-rs` WASAPI (`IMMDeviceEnumerator` → default capture +
  default render endpoints; `IAudioClient` / `IAudioCaptureClient`).
- **Mode**: **shared mode**, **event-driven** (`AUDCLNT_STREAMFLAGS_EVENTCALLBACK`).
  Each capture loop waits on its event handle, then drains the device with a
  `GetBuffer` / `ReleaseBuffer` cycle and forwards timestamped frames to the
  mixer.
- **Loopback**: `loopback.rs` activates the **default render device** with
  `AUDCLNT_STREAMFLAGS_LOOPBACK` to capture system audio. This is the
  high-risk native flag the story calls out; mirror meetily's
  `windows.rs` shape (see execplan step 0 — confirm the reference is present
  on this machine first).
- **One thread per stream**: a dedicated mic-capture thread and a dedicated
  loopback-capture thread (see §7).
- **Silent render device (AC5)**: in shared-mode loopback, WASAPI delivers
  **no packets** while nothing is playing. The capture loop must not block
  the pipeline waiting on it — the loop wakes on its event/timeout, and the
  **mixer** synthesizes silence to keep the loopback timeline advancing (see
  §5.3). Recording must never stall on an idle render device.
- **Native format**: each device reports its mix format via
  `GetMixFormat` — commonly 44.1 kHz or 48 kHz, float32, 1–2 channels. We
  **accept the device-native format** and convert in the mixer (see §8); we
  do not try to force 16 kHz at the device.

### Known constraint (surface to users)

WASAPI loopback on Windows requires "Stereo Mix" enabled or a virtual audio
cable on some configurations (recorded in `docs/ARCHITECTURE.md` → Known
constraints). Phase 1 must surface this to end users — flagged in the
initiative's affected-product-docs list; a user-facing setup note is a
follow-up, not code in this story.

## 4. Data flow

```
mic device  ──► capture.rs  ──┐  timestamped PCM frames (native fmt)
                              ├─► mixer.rs (ring buffer)
render dev  ──► loopback.rs ──┘     │  per-stream resample → 16 kHz mono, time-align, sum
                                    ├─► wav_writer.rs  (streaming append → <meeting-id>.wav)
                                    └─► vu.rs          (per-stream peak/RMS → Tauri events ≥10 Hz → React)

Stop: AudioController signals threads → drain ring buffer → final WAV write
      → wav_writer patches RIFF/data sizes (header fixup).
```

Both capture threads push frames (tagged with stream id + capture
timestamp) into the shared mixer. The mixer produces one 16 kHz mono PCM
stream that feeds **both** the WAV writer (single-pass, live) and the VU
meter computation. VU is computed **on the mixer thread** from the
per-stream level *before* downmix (so mic and loopback report independent
bars) and emitted as async Tauri events — no WAV polling (AC3).

## 5. Resolved sub-decisions

### 5.1 Two-stream sync model — **shared ring buffer + real-time mix**

**Options**

- (A) Shared ring buffer on a common timeline, downmix in real time, single
  live WAV write.
- (B) Per-stream buffers held in full, offline mix on Stop.

**Decision: (A) shared ring buffer + real-time mix.**

**Rationale (tied to ACs)**

- AC4 requires ≥5-minute runs with **no growing memory**. (B) buffers whole
  streams → memory grows with duration → conflicts AC4. (A) is bounded by
  the ring-buffer size regardless of run length.
- (A) enables a single-pass live WAV write (§5.2) and live VU (AC3) from the
  same mixed stream. (B) cannot drive live VU from mixed output.
- Mirrors meetily's `AudioMixerRingBuffer` (the story's named reference).

**Time alignment (how ±50 ms is hit — AC4)**

Mic and loopback are **independently-clocked hardware**; sample-counting
alone drifts and will not hold ±50 ms over 5 minutes. The mixer aligns by
**capture timestamps**, not by concatenating sample counts:

- Each captured packet carries a device/QPC timestamp from the WASAPI
  `GetBuffer` cycle (device position + performance-counter time).
- The mixer maintains a common output timeline and places each resampled
  packet into the ring buffer at the slot its timestamp maps to, using
  **~50 ms alignment windows** (meetily's alignment-window pattern).
- Drift between the two device clocks is corrected per window rather than
  allowed to accumulate, keeping the streams within ±50 ms across the run.

### 5.2 WAV write policy — **streaming append with header fixup**

**Options**

- (A) Streaming append: write a header with placeholder sizes up front,
  append mixed PCM continuously, back-patch RIFF/data sizes on Stop.
- (B) Buffer all mixed PCM in memory, write the whole file on Stop.

**Decision: (A) streaming append.**

**Rationale**

- Bounded memory (AC4): PCM hits disk continuously; nothing accumulates.
- A forced-kill partial is *mostly valid* — header carries placeholder
  sizes but the PCM payload is intact and recoverable (AC6, §5.3).
- Matches the architecture's stated convention: "Rust writes the WAV header
  on recording start and appends raw frames continuously"
  (`docs/ARCHITECTURE.md` → Audio File Convention).

### 5.3 Partial-file recovery — **auto-repair on next launch (no DB)**

**Constraint**: Phase 1 has **no meetings DB** (that is Phase 3), so a UI
"incomplete" state has nowhere to persist. The story (AC6) explicitly leaves
the choice to this design.

**Options**

- (A) On next launch, scan the audio dir for WAVs whose header was never
  finalized and auto-repair the header.
- (B) Surface "incomplete" in the UI — requires a place to record per-file
  status, i.e. the DB Phase 1 does not have.

**Decision: (A) auto-repair on next launch.**

**Mechanism (sentinel + repair)**

- On Record start, `wav_writer` writes the RIFF and `data` chunk size fields
  as a **placeholder sentinel** (e.g. `0xFFFFFFFF`) rather than a real
  size.
- On clean Stop, those fields are patched to the true sizes (header
  finalized).
- On next launch, `AudioController` scans
  `%APPDATA%\Chronicler\audio\` for any `*.wav` still carrying the sentinel
  → that file is **unfinalized**. Recompute the real `data` size from the
  file length, **rounded down to the nearest whole sample frame**, patch
  the RIFF/data fields, and the file becomes a valid, playable WAV.

**Phase-3 follow-up (drift)**: once the meetings DB exists, surfacing
recovered/incomplete files in the meeting list is the better UX. Noted as
drift in §9; not actioned here.

## 6. Lifecycle — mirroring S02's teardown discipline (AC1)

S02 (`frontend/src-tauri/src/lib.rs`) established the proven pattern this
design reuses:

- **Module-level state** (S02: `BACKEND_CHILD: Mutex<Option<CommandChild>>`,
  `BACKEND_PORT: OnceLock<u16>`). S03 adds an `AudioController` held in a
  `Mutex<Option<…>>` owning the capture/mixer thread `JoinHandle`s and a
  shared stop signal (e.g. an `AtomicBool` / channel).
- **IPC commands** registered in the `invoke_handler` (S02 registered
  `get_backend_port`). S03 registers `start_recording` and `stop_recording`
  (returning the `<meeting-id>` / WAV path), plus the recovery scan at setup.
- **Deterministic teardown** (S02: `kill_backend` on `WindowEvent::Destroyed`
  with `taskkill /F /T` to kill orphan children). S03's Stop path signals
  the threads, joins them, drains the ring buffer, finalizes the WAV header,
  and **releases every WASAPI handle** (`IAudioClient::Stop` + COM release) —
  the S03 analogue of "no orphan audio handles." The same teardown runs on
  the window `Destroyed` event so closing the window mid-recording leaves no
  orphan handles and a recoverable partial WAV.
- **Non-fatal start failures**: as S02 kept spawn failures non-fatal, a
  device-open failure (no mic, loopback blocked) must return a clear error
  to the UI rather than crash the app.

## 7. Thread model — reconciliation + drift flag

`CLAUDE.md` (the canonical source `docs/ARCHITECTURE.md` defers to) states the
Rust thread budget as **"1 capture thread + 1 VAD/chunker thread"** (2 threads).

S03 actually uses **3 Rust threads**:

| Thread | Role |
|---|---|
| mic capture | event-driven WASAPI mic `GetBuffer`/`ReleaseBuffer` loop |
| loopback capture | event-driven WASAPI loopback loop |
| mixer / writer | ring-buffer align + resample + downmix + streaming WAV append + VU computation |

- **VU** is computed on the mixer thread and emitted via **async** Tauri
  events — no dedicated VU thread.
- There is **no VAD/chunker thread** in Phase 1 — that arrives in Phase 2.

So the deviation is both **count** (3 vs 2) and **composition** (two capture
threads + a mixer/writer thread, no VAD thread yet). This is within the
overall machine thread budget (Ryzen 5 3600X, 6C/12T) and the
recording-time budget has ample headroom (faster-whisper is not running in
Phase 1). **This is a documented drift from the stated Rust thread budget
and should be ratified by the architect** before/at the next ARCHITECTURE.md
update. This design does not edit ARCHITECTURE.md.

## 8. Sample-rate strategy (feeds ADR-0005 Consequences)

- WASAPI yields the **device-native** shared-mode format — typically
  44.1 kHz or 48 kHz, float32. Shared-mode WASAPI **cannot** be forced to a
  16 kHz device format (the audio engine fixes the mix format); exclusive
  mode could request 16 kHz but is fragile, device-dependent, and would
  block other apps from the device — unacceptable for a loopback of *system*
  audio.
- Therefore: **capture native, then resample + downmix to 16 kHz mono in the
  mixer.** Each stream gets a per-stream resampler ahead of the sum, so
  mic@48k and loopback@44.1k both land on the common 16 kHz mono timeline
  before mixing. 16 kHz mono is the format faster-whisper/whisperx need in
  Phase 2 (`docs/ARCHITECTURE.md` → Audio File Convention).
- **Note vs meetily**: meetily normalizes internally to 48 kHz; we do **not**
  import that figure — our normalization target is **16 kHz** by design.
- The **resampler crate** choice (e.g. `rubato` vs. a hand-rolled
  polyphase/linear resampler) is a decision branch resolved in **ADR-0005**;
  this design fixes only the strategy (per-stream resample-before-mix to
  16 kHz mono).

## 9. Drift / follow-ups (for the architect; not actioned here)

- **Thread budget** (§7): 3 Rust threads vs the documented 2, different
  composition. Needs ratification + an ARCHITECTURE.md update when S03 lands.
- **Partial-file UI** (§5.3): Phase-3 meetings DB should surface
  recovered/incomplete WAVs in the meeting list.
- **User-facing setup note**: WASAPI loopback "Stereo Mix" / virtual-cable
  requirement should be surfaced to end users (per initiative).

## 10. Acceptance-criteria → design map

| AC | Where satisfied |
|---|---|
| AC1 open both / clean stop, no orphan handles | §3, §6 |
| AC2 valid 16 kHz mono mixed WAV | §4, §5.2, §8 |
| AC3 two VU bars ≥10 Hz via Tauri events | §4 (vu.rs), §7 |
| AC4 ≥5 min stable, ±50 ms sync | §5.1, §5.2 |
| AC5 silent render device still clocked | §3, §5.3 |
| AC6 header finalize + partial recovery | §5.2, §5.3 |
| AC7 Rust unit tests in validate:quick | mixer.rs + wav_writer.rs (see execplan/validation) |
| AC8 60 s manual smoke + VU screenshot | see validation.md |
