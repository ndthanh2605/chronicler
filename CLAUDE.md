# Chronicler — Agent Guide

Chronicler is a local-first Windows desktop app that transcribes and summarizes meetings and
videos. It captures microphone and system audio simultaneously, streams live transcription
during recording, then sends the full transcript to a user-managed GPU for correction and
prose summarization. Speaker diarization is available on demand.

No application code has been written yet. Development proceeds iteratively: one story at a
time, each validated before the next begins. The phased roadmap (`docs/ROADMAP.md`) sets
direction — each phase is a vertical slice with one declared user-visible output, picked up
by an agent as a **New initiative** and decomposed into story packets. The harness framework
(`docs/HARNESS.md`) governs how agents plan, implement, and verify each increment.

## Source Of Truth

Read in this order at the start of every session:

1. `README.md` — project status.
2. `docs/HARNESS.md` — development operating model.
3. `docs/FEATURE_INTAKE.md` — before turning any prompt into work.
4. `docs/ROADMAP.md` — phased direction; tells you which phase a task belongs to.
5. `docs/superpowers/specs/2026-05-18-chronicler-design.md` — approved architecture (full).
6. `docs/product/` — current product contracts.
7. `docs/ARCHITECTURE.md` — component ownership and data flows; read before proposing implementation.
8. `docs/stories/` — story packets and backlog.
9. `docs/TEST_MATRIX.md` — behavior-to-proof control panel.
10. `docs/decisions/` — ADRs (why important choices were made).
11. `.claude/skills/` — development workflow skills (see below).

When picking up an **in-progress story**, read the story's workpad sibling first. The workpad
contains prior attempt context and takes priority over the general reading order above.

## Skills

Use these skills for all development operations. Do not use the global `commit-commands` skills.

| Skill | When to invoke |
|---|---|
| `harness-intake` | Start of every non-trivial task — before reading code or planning |
| `harness-git-pull` | Before starting or resuming work on a story branch |
| `harness-git-commit` | Every commit (enforces conventional commit format + security checks) |
| `harness-git-push` | Push branch and open/update PR with story Evidence link |
| `harness-git-land` | Merge approved PR and close the story |

## Architecture

### Stack

| Layer | Technology | Why |
|---|---|---|
| Desktop shell | Tauri 2 (Rust) | Native Windows, WASAPI audio, system tray |
| UI | React 18 + Vite + TypeScript | Lightweight for desktop; no SSR needed |
| Backend sidecar | FastAPI (Python) | ML ecosystem: whisperx, pyannote, async HTTP |
| Live transcription | faster-whisper `medium.en` | ~0.8–1.0× RTF on 6 CPU cores |
| Post-processing | Vast.ai GPU + Qwen3-8B (vLLM) | Correction + summary in one GPU pass |
| Diarization | whisperx + pyannote 3.1 | On-demand only; CPU-heavy, not time-critical |
| Persistence | SQLite via aiosqlite | Simple, local, async |
| Audio storage | WAV files | Retained per meeting for diarization reprocessing |

### Component ownership

**Tauri / Rust** (`frontend/src-tauri/`):
- WASAPI audio capture: mic device + loopback (system audio)
- Voice activity detection; 5-second speech chunk assembly
- HTTP POST of chunks to FastAPI `/transcribe`
- WebSocket connection to FastAPI for transcript stream → forwards tokens to React UI
- Tauri IPC commands exposed to the React front end (start/stop recording, open settings)
- Audio written continuously to `%APPDATA%\Chronicler\audio\<meeting-id>.wav`

**FastAPI sidecar** (`backend/`):
- `/transcribe` endpoint: receives audio chunks, runs faster-whisper, returns transcript segments
- WebSocket endpoint: streams transcript tokens to Tauri
- `/postprocess` endpoint: reads full transcript from SQLite, calls Vast.ai, stores result
- `/diarize` endpoint: reads WAV, runs whisperx + pyannote, stores speaker-annotated JSON
- SQLite read/write via aiosqlite
- Vast.ai client (OpenAI-compatible; endpoint URL + key from user settings)

**React UI** (`frontend/src/`):
- Calls Tauri IPC commands for recording control
- Displays rolling live transcript (tokens from WebSocket via Tauri)
- Shows post-processing results and diarization output from SQLite
- Settings screen: Vast.ai endpoint URL + API key, stored to SQLite config table

**SQLite** (`%APPDATA%\Chronicler\chronicler.db`):
- `meetings(id, title, started_at, stopped_at, status)`
- `transcripts(id, meeting_id, speaker, start_ms, end_ms, text)`
- `summaries(id, meeting_id, corrected_transcript, prose_summary, model, created_at)`
- `config(key, value)` — user settings (Vast.ai endpoint, pyannote token, etc.)

### Data flows

```
Recording:
  WASAPI → Rust VAD → HTTP POST /transcribe → faster-whisper
  → WebSocket tokens → Tauri → React UI

  Rust → WAV file (continuous write)
  FastAPI → SQLite transcripts (per segment)

Post-processing (on Stop):
  FastAPI reads full transcript from SQLite
  → HTTP POST to Vast.ai (Qwen3-8B)
  → corrected_transcript + prose_summary → SQLite summaries

Diarization (on demand):
  FastAPI reads WAV → whisperx alignment → pyannote 3.1
  → speaker-annotated JSON → SQLite transcripts (speaker field updated)
```

### Thread budget

| Process | Threads |
|---|---|
| Rust | 1 capture thread + 1 VAD/chunker thread |
| FastAPI | 1 async event loop + 2 whisper workers (ProcessPoolExecutor) |
| WebSocket | async (within FastAPI event loop) |

### Known constraints

- WASAPI loopback requires "Stereo Mix" enabled or a virtual audio cable on Windows. Document
  this clearly in user-facing setup instructions.
- pyannote 3.1 requires a HuggingFace auth token accepted once at model download.
- Vast.ai endpoint must be user-configured before post-processing works. FastAPI must degrade
  gracefully (return a clear error, not crash) when the endpoint is not set.
- Diarization is CPU-heavy and blocks the FastAPI event loop if run synchronously — must use
  ProcessPoolExecutor or a background thread.

## Task Loop

For every task:

1. **Orient first.**
   - Invoke `harness-intake`. Do not proceed until it produces a cleared Intake Report.
   - If picking up an in-progress story: read the workpad sibling first to understand what was
     previously attempted and why it stopped — this context takes priority over everything else.
   - Read the story packet, affected product docs, and `docs/TEST_MATRIX.md`.

2. Identify the input type: new feature, bug fix, maintenance, or harness improvement.

3. Work only inside the selected lane: tiny, normal, or high-risk.

4. Use the correct skill for every git operation (pull → commit → push → land).

5. Update the workpad sibling continuously — not just at the end. Record decisions made,
   what was attempted, and enough context for the next agent to resume cold.

6. Before finishing, verify:
   - Did product truth change? → update `docs/product/`.
   - Did validation expectations change? → update `docs/TEST_MATRIX.md`.
   - Did architecture rules change? → update `docs/ARCHITECTURE.md` and add a decision record.
   - Did we discover a repeated failure pattern? → add to `docs/HARNESS_BACKLOG.md`.
   - Is the workpad complete enough for a cold-start agent to resume without re-investigating?
   - Is the story status at the correct state-machine gate?

7. Update routine harness files directly; add structural proposals to `docs/HARNESS_BACKLOG.md`.

## Harness Change Policy

Agents may update directly:
- Story status and evidence.
- `docs/TEST_MATRIX.md` rows.
- Links from story packets to product docs.
- Validation notes and reports.
- Small clarifications tied to the current task.

Agents must ask for human confirmation before:
- Changing architecture direction or component ownership.
- Removing validation requirements.
- Changing the source-of-truth hierarchy.
- Changing risk classification rules.

## Done Definition

A task is done only when:

- The requested change is completed, or the blocker is documented with enough detail for the
  next agent to resume without re-investigating from scratch.
- Relevant product docs, stories, and `docs/TEST_MATRIX.md` entries are current.
- Validation commands were run when they exist.
- Missing harness capabilities were added to `docs/HARNESS_BACKLOG.md`.
- The workpad sibling is complete: what was done, what was not attempted, what the next step is.
- The final response states what changed and what was not attempted.
