# S03 — Execution plan: mic + WASAPI loopback → mixed WAV

Ordered implementation steps for
`docs/stories/phase-1-audio-capture/S03-mic-loopback-mixed-wav.md`. Each step
names the proof that closes it and the AC / TM row it serves. Decision
branches are marked **[branch]**; the resampler-crate branch resolves in
**ADR-0005** (this plan does not pick the crate). Read
`S03-mic-loopback-mixed-wav.design.md` before starting — it resolves the
sync model, WAV write policy, and partial-recovery decisions.

Steps are ordered so the **pure, testable** modules (id, WAV writer, mixer)
land first with unit tests, before the native WASAPI capture that can only be
verified at a Windows GUI.

---

## Step 0 — Preconditions (no Rust yet)

- **S02 landed**: S03 *code* is gated on S02 (PR #2) merging to `main`. This
  branch sits on the S02 tip (`cef8f5d`); **rebase S03 onto post-S02 `main`
  before writing any Rust** (per the story's Notes-for-next-agent).
- **Confirm the meetily reference**: locate
  `meetily/frontend/src-tauri/src/audio/devices/platform/windows.rs` and
  `AudioMixerRingBuffer` on this machine.
  - **[branch]** If present → use it as the concrete shape for loopback
    activation and the ring-buffer alignment window.
  - If **absent** (it was not found on the docs-authoring machine) → fall
    back to the approved spec
    (`docs/superpowers/specs/2026-05-18-chronicler-design.md`) + `windows-rs`
    WASAPI docs (cross-check loopback flags via `microsoft-learn` /
    `context7`).
- **Proof**: rebased branch builds (`pnpm validate:quick` green on inherited
  S02 state); reference located or fallback recorded in the workpad.

## Step 1 — Crates + module skeleton

- Add dependencies: `windows` (WASAPI + Media + COM features), a resampler
  crate **[branch → ADR-0005]** (e.g. `rubato` vs. hand-rolled), and a
  time-ordered id crate (`uuid` v7 or `ulid`).
- Create the empty `audio/` module tree (design §2) with `pub` stubs and wire
  `mod audio;` into `lib.rs`.
- **Proof**: `cargo build` succeeds (no behavior yet). Serves all ACs as the
  foundation.

## Step 2 — `meeting_id.rs` + `wav_writer.rs` (pure, unit-tested)

- `meeting_id.rs`: generate a time-ordered `<meeting-id>` for the filename.
- `wav_writer.rs`: open file, write a 16 kHz/mono/16-bit PCM WAV header with
  **placeholder sentinel** RIFF/data sizes (design §5.2/§5.3); streaming
  `append(frames)`; `finalize()` back-patches RIFF + `data` sizes; a
  standalone `repair(path)` that recomputes `data` size from file length
  rounded down to a frame boundary and patches the header.
- **Proof — Rust unit tests** (AC7; partially satisfies TM-007's "valid WAV"
  shape):
  - header bytes are correct (RIFF/WAVE/fmt /data, 16000 Hz, 1 ch, 16-bit);
  - `finalize()` writes correct RIFF + data sizes for a known PCM length;
  - a sentinel-bearing file is detected as unfinalized and `repair()`
    produces a valid header at the last whole frame (AC6).

## Step 3 — `mixer.rs` ring buffer + downmix (pure, unit-tested)

- Bounded ring buffer; per-stream resample-to-16 kHz then sum to mono
  (design §8); timestamp-based alignment in ~50 ms windows (design §5.1);
  **silence synthesis** when a stream has a timestamp gap (design §5.3 / AC5).
- **Proof — Rust unit tests** (AC4 + AC7):
  - **tick-injection sync**: feed two synthetic streams each carrying a known
    impulse at a known time; assert the impulse offset in the mixed output is
    within ±50 ms (AC4 method, in-process);
  - downmix correctness (two known inputs → expected summed/clamped output);
  - a gap in one stream yields silent frames at the expected rate, no stall,
    timeline stays aligned (AC5 mechanism);
  - bounded memory: long synthetic feed does not grow buffers unboundedly.

## Step 4 — `wasapi/mod.rs` + `wasapi/capture.rs` (mic)

- Shared COM/`IAudioClient` activation + `GetMixFormat` negotiation in
  `wasapi/mod.rs`; default-mic event-driven capture loop in `capture.rs`
  feeding timestamped frames to the mixer.
- **Proof** — `cargo build`; **manual Windows GUI** capture check + optional
  Rust **integration test** asserting captured→WAV shape via `ffprobe`
  (TM-005, AC1 partial).

## Step 5 — `wasapi/loopback.rs` (system audio, incl. silence path)

- Default render device activated with `AUDCLNT_STREAMFLAGS_LOOPBACK`;
  event-driven loop; **must not block** when the render device is silent —
  no packets → mixer synthesizes silence (design §3/§5.3).
- **Proof** — **manual** smoke: system audio appears in the mixed WAV; with
  system audio paused, loopback still produces silent PCM at the expected
  rate and recording does not stall (TM-006, AC5).

## Step 6 — `AudioController` start/stop + IPC wiring in `lib.rs`

- Implement `AudioController::start()` (open both devices, spawn the three
  threads, begin streaming WAV write) and `stop()` (signal → join → drain →
  `wav_writer.finalize()` → release all WASAPI handles), mirroring S02's
  `Guard`/`kill_backend` teardown and `Mutex<Option<…>>` state (design §6).
- Register `start_recording` / `stop_recording` Tauri IPC commands in
  `invoke_handler`; run the **partial-recovery scan** (design §5.3) at
  `setup`; run teardown on `WindowEvent::Destroyed`.
- Add minimal React Record/Stop controls that invoke the commands.
- **Proof** — `cargo build` + **manual**: Record opens both streams, Stop
  terminates both with no orphan audio handles; forced-kill then relaunch
  auto-repairs the partial WAV (TM-005, AC1 + AC6).

## Step 7 — `vu.rs` + Tauri events + React VU bars

- Compute per-stream peak/RMS on the mixer thread (before downmix); emit
  async Tauri events at ≥10 Hz; React renders two independent level bars.
- **Proof** — **manual** smoke + screenshot: two bars move independently and
  drop to silence within 200 ms of mute; levels arrive via the event channel,
  not WAV polling (TM-008, AC3 + AC8).

## Step 8 — Wire Rust unit tests into `validate:quick`

- Extend the `validate:quick` script so the mixer (step 3) and WAV-writer
  (step 2) unit tests run as part of the quick gate.
- **Proof** — `pnpm validate:quick` runs the new Rust tests and is green
  (AC7).

## Step 9 — Full manual smoke + evidence

- 60-second recording of voice + a YouTube clip → playable mixed WAV with
  both audible; VU screenshot; `ffprobe` confirms 16 kHz mono and duration
  within ±100 ms; ≥5-minute stability run; tick-injection ±50 ms check on a
  real recording.
- **Proof** — manual smoke matrix in
  `S03-mic-loopback-mixed-wav.validation.md` executed; evidence attached to
  the story (TM-005..008, AC1/AC2/AC4/AC8).

---

## Step → AC / TM coverage

| Step | Proof | AC | TM |
|---|---|---|---|
| 0 | rebase + reference located | gating | — |
| 1 | `cargo build` | foundation | — |
| 2 | Rust unit tests (WAV header/repair) | AC2 (partial), AC6, AC7 | TM-007 (partial) |
| 3 | Rust unit tests (mixer/sync/silence) | AC4, AC5 (mech), AC7 | — |
| 4 | build + manual + int. test | AC1 (partial) | TM-005 |
| 5 | manual smoke | AC5 | TM-006 |
| 6 | build + manual | AC1, AC6 | TM-005 |
| 7 | manual + screenshot | AC3, AC8 | TM-008 |
| 8 | `validate:quick` green | AC7 | — |
| 9 | full manual smoke | AC1, AC2, AC4, AC8 | TM-005..008 |

> AC4 (tick-injection sync) and AC6 (header unit test + forced-kill check)
> are validated by named methods, not by a dedicated TM row — see
> `validation.md`.

## Decision branches summary

- **Step 0** — meetily reference present vs. spec/docs fallback.
- **Step 1** — resampler crate (`rubato` vs. hand-rolled) → **resolved in
  ADR-0005**, not in this plan.
