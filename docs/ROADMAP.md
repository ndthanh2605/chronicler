# Chronicler — Roadmap

## Context

Chronicler is a private, local-first Windows desktop app for transcribing and
summarizing meetings and videos. Architecture is approved
(`docs/superpowers/specs/2026-05-18-chronicler-design.md`): Tauri 2 + React UI,
FastAPI Python sidecar, faster-whisper for live ASR, Vast.ai (Qwen3-8B) for
correction + summary, on-demand pyannote diarization, SQLite persistence.

Repo state: harness v0 is in place — no application code. This roadmap is the
high-level direction. Agents expand each phase into story packets and execute
under the harness state machine.

**Roadmap philosophy:**

- Each phase is a vertical slice with one user-visible or operator-visible output.
- Each phase is small enough to fit one "initiative" and large enough to be worth
  scaffolding a story bundle around.
- Sequencing is dependency-driven: capture → transcribe → persist → enrich → ship.
- Agents pick a phase, run feature intake, decompose into stories, and execute.
- The roadmap is direction, not contract. Phases may split or merge as friction
  surfaces; record drift in `docs/HARNESS_BACKLOG.md` and ADRs.

---

## Phase 0 — Scaffold

**Output:** Tauri 2 shell launches, embeds React/Vite UI, spawns FastAPI sidecar
on a known port, and a UI button completes a backend round-trip (e.g. `/health`).
SQLite file created on first run. No real audio or ASR yet.

**Why first:** Establishes the three-process topology (Rust ↔ Python ↔ UI) and
the build/dev loop. Every later phase depends on it.

---

## Phase 1 — Audio capture

**Output:** Pressing **Record** captures mic + WASAPI loopback into a single
mixed WAV file under `%APPDATA%\Chronicler\audio\<meeting-id>.wav`. VU meters in
UI show both streams live. **Stop** finalizes the file. No transcription yet.

**Why next:** Audio is the hardest platform-bound piece (WASAPI loopback flag,
ring-buffer alignment). De-risk it before wiring ML on top. Reference meetily's
`windows.rs` and `AudioMixerRingBuffer`.

---

## Phase 2 — Live transcription

**Output:** While recording, 5-second speech chunks (post-VAD) stream from Rust
to FastAPI; `faster-whisper medium.en` transcribes; tokens stream back to UI via
WebSocket and render as rolling text with < 5 s lag.

**Why next:** Closes the realtime loop end-to-end and proves the thread budget
(3 cores for whisper, 2 reserved for OS/UI). Surfaces VAD tuning and back-pressure
before persistence is involved.

---

## Phase 3 — Meeting persistence

**Output:** Meetings, transcripts, and (empty) summaries persist in SQLite per
the approved schema. UI has a meeting list, re-open shows the live-kind
transcript and the WAV path. Status state machine wired:
`recording → processing → completed | error`.

**Why next:** Live capture has no value without recall. Establishes the data
contract that post-processing and diarization will write into.

---

## Phase 4 — Post-processing (Vast.ai)

**Output:** Settings panel accepts Vast.ai endpoint URL + API key (stored in
`settings.json`, never committed). On **Stop**, FastAPI calls the OpenAI-compat
endpoint with a single prompt; response parsed into `corrected_transcript` and
`summary`. Both persisted and rendered. Endpoint unreachable → raw transcript
shown, summary shows actionable "configure GPU endpoint" message.

**Why next:** Delivers the headline value (clean transcript + prose summary)
without local GPU cost. Graceful-degradation path is a hard gate because the
GPU is user-managed and frequently offline.

---

## Phase 5 — On-demand diarization

**Output:** Completed meetings show **"Annotate speakers"**. First click prompts
for HF token (stored in `settings.json`). `whisperx` + `pyannote 3.1` run as a
background task with progress in UI. Speaker-labeled transcript persisted as
`kind='diarized'` and replaces the rendered transcript when ready. Recording is
blocked while diarization runs (thread budget).

**Why next:** Speaker labels are valuable but not on the critical recording
path. Doing it last avoids blocking earlier phases on pyannote/HF-token UX.

---

## Phase 6 — Onboarding & packaging

**Output:** First launch downloads faster-whisper `medium.en` (~500 MB) with a
resumable progress bar. Settings round-trips Vast.ai endpoint, HF token, and
whisper-model fallback (`medium.en` ↔ `small.en`). `tauri build` produces a
signed Windows installer; README documents Vast.ai setup. App is installable on
a clean Windows machine and reaches a working state without dev tooling.

**Why next:** Until this phase exists, only developers can run Chronicler. This
turns it into a shippable artifact.

---

## Phase 7 — Hardening

**Output:** Thread caps enforced via env vars and verified under load. Crash
recovery: incomplete meetings reach `error` cleanly with audio intact. Backend
logs captured to a rotating file under `%APPDATA%`. Performance smoke test
documents realtime factor on the reference machine (Ryzen 5 3600x, 6C/12T).
Test matrix covers the verification plan items in the spec.

**Why last:** Reliability work needs real surface area to harden. Doing it
earlier produces speculative tests; doing it now produces tests that match
observed behavior.

---

## Cross-cutting concerns

These don't get their own phase — they accrue inside whichever phase first
touches them, and graduate into a dedicated story only if scope demands:

- **ADRs** as each phase makes contract-shaping choices (model selection,
  prompt format, ring-buffer alignment window, etc.).
- **Test matrix** rows added per phase, mapped to the spec's verification plan.
- **Validation ladder** (`validate:quick` → `test:release`) appears
  incrementally — first script lands in Phase 0, more rungs accrete as the
  stack solidifies.
- **Harness friction** noted in `docs/HARNESS_BACKLOG.md` whenever an agent
  hits a missing rule, repeated reasoning, or absent validation command.

---

## Critical files (read before designing any phase)

- `docs/superpowers/specs/2026-05-18-chronicler-design.md` — full architecture.
- `docs/HARNESS.md` — operating model and state machine.
- `docs/FEATURE_INTAKE.md` — lane classification before any work.
- `~/Repos/meetily/frontend/src-tauri/src/audio/devices/platform/windows.rs` —
  WASAPI loopback reference (Phase 1).
- `~/Repos/stenoai` — Electron + Python sidecar pattern reference (Phase 0).

---

## How an agent picks up a phase

1. Invoke `harness-intake` skill.
2. Classify input as **New initiative** (a phase = an initiative).
3. Write initiative notes under `docs/stories/` listing candidate stories,
   affected product docs, validation shape, and open decisions.
4. For each story: normal lane unless the risk checklist forces high-risk
   (Phase 1 audio, Phase 4 Vast.ai integration, and Phase 6 packaging are
   likely high-risk candidates).
5. Execute under the state machine; close the phase only when its single
   declared output works end-to-end on the reference machine.

---

## Status of this roadmap

This file is direction, not code. It is "verified" when the first phase to be
picked has a complete initiative note and at least one story packet in
`docs/stories/` that an agent can begin executing without further user input.
