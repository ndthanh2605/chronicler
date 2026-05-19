# Chronicler — Design Plan

## Context

Build a private, local-only Windows desktop app for transcribing and summarizing meetings and
videos. The app captures live microphone and system audio (screen + loopback), provides
real-time transcription during recording, and produces a speaker-annotated full-prose summary
after the session ends. Everything runs on-device — no cloud, no external APIs.

Reference repos studied: meetily (Tauri 2 + WASAPI + whisper-rs), stenoai (Electron + Python).
Approach chosen: **B** — fresh repo, harness overlay from day one, meetily as architecture
blueprint (not a fork).

---

## Project Identity

| | |
|---|---|
| **Name** | Chronicler |
| **Repo** | `~/Projects/chronicler` |
| **Planning codename** | synthetic-beaver |
| **Platform** | Windows (primary), WSL2 for LLM server |

---

## Stack

| Layer | Technology | Rationale |
|---|---|---|
| Desktop shell | Tauri 2 (Rust) | Native Windows, WASAPI audio, system tray |
| UI | React 18 + Vite + TypeScript | Lighter than Next.js for desktop; no SSR needed |
| Backend | FastAPI (Python, sidecar) | ML ecosystem: whisperx, pyannote, Vast.ai client |
| Transcription — live | faster-whisper `medium.en` | ~0.8–1.0× RTF on 6 cores; fits realtime budget |
| Post-processing | Vast.ai GPU + Qwen3-8B (vLLM) | Transcript correction + prose summary in one GPU pass |
| Diarization | whisperx + pyannote 3.1 | On-demand only; avoids blocking recording or post flow |
| Persistence | SQLite via aiosqlite | Meetings, transcripts, summaries |
| Audio storage | WAV files (`%APPDATA%\Chronicler\audio\`) | Retained per-meeting for diarization reprocessing |

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                       Chronicler (Tauri 2)                       │
│                                                                  │
│  ┌──────────────────────┐     ┌────────────────────────────┐    │
│  │   React + Vite UI    │←───→│  Rust Core                 │    │
│  │  · Live transcript   │     │  · WASAPI mic capture      │    │
│  │  · Speaker labels    │     │  · WASAPI loopback         │    │
│  │  · Prose summary     │     │  · VAD silence filter      │    │
│  │  · Meeting history   │     │  · Audio chunk forwarder   │    │
│  └──────────┬───────────┘     └─────────────┬──────────────┘    │
│             │ Tauri events                   │ HTTP POST         │
└─────────────┼───────────────────────────────┼───────────────────┘
              │ HTTP / WebSocket              │ 5s audio chunks
              ↓                               ↓
┌──────────────────────────────────────────────────────────────────┐
│                    FastAPI Backend (sidecar)                      │
│                                                                  │
│  ┌──────────────────┐  ┌────────────────────┐  ┌─────────────┐  │
│  │  faster-whisper  │  │  whisperx +        │  │   SQLite    │  │
│  │  medium.en(live) │  │  pyannote 3.1      │  │  (meetings, │  │
│  └──────────────────┘  │  (on-demand only)  │  │  transcripts│  │
│                        └────────────────────┘  │  summaries) │  │
│                                    │ HTTPS      └─────────────┘  │
└────────────────────────────────────┼─────────────────────────────┘
                                     ↓
                           ┌──────────────────────┐
                           │  Vast.ai GPU instance │
                           │  vLLM + Qwen3-8B      │
                           │  OpenAI-compat API    │
                           │  (user-managed)       │
                           └──────────────────────┘
```

---

## Session Lifecycle

### Phase 1 — Recording (live, during the meeting)
1. User clicks **Record** → Rust starts WASAPI capture on both mic and loopback device
2. VAD filters silence; passes 5-second speech chunks to FastAPI via HTTP POST
3. FastAPI runs `faster-whisper medium.en` on each chunk (~1–3 s lag on CPU)
4. Transcript tokens stream back to UI via WebSocket; displayed as rolling text
5. Raw audio written continuously to `%APPDATA%\Chronicler\audio\<meeting-id>.wav`

### Phase 2 — Post-processing (triggered on Stop, one GPU pass)
1. FastAPI sends raw live transcript to Vast.ai instance (OpenAI-compatible API)
2. Qwen3-8B prompt: correct transcription errors → then write full-prose summary
3. Response contains two sections: corrected transcript + prose summary
4. Both stored in SQLite (`kind='final'` and summary row), displayed in UI
5. Meeting status updated to `completed`

**Vast.ai instance model**: Qwen3-8B via vLLM, user manages instance lifecycle.
API endpoint URL + key stored in `%APPDATA%\Chronicler\settings.json` (never in repo).
If endpoint unreachable: raw live transcript shown as-is, summary skipped with actionable error.

### Phase 3 — On-demand diarization (user-triggered)
1. User clicks **"Annotate speakers"** on any completed meeting
2. FastAPI runs `whisperx` + `pyannote 3.1` on the stored WAV file
3. Progress shown in UI (background task; can take minutes on CPU)
4. Speaker-labeled transcript stored (`kind='diarized'`), replaces final transcript in view
5. If pyannote HF token not yet set: prompt for it at this point, not on first launch

---

## Thread Budget

Hardware: Ryzen 5 3600x, 6C/12T. Hard caps via env vars:

| Process | Threads | When active |
|---|---|---|
| faster-whisper (live) | 3 | During recording |
| pyannote diarization | 4 | On-demand only |
| OS + Tauri UI | 2 | Always reserved |

Post-processing is offloaded to Vast.ai — no local thread cost for transcript correction
or summarization. Diarization is user-triggered and never overlaps with recording.

---

## Model Downloads (~750 MB on first run)

| Model | Size | Used for | HF token? |
|---|---|---|---|
| faster-whisper `medium.en` | ~500 MB | Live transcription | No |
| pyannote 3.1 | ~250 MB | Diarization (on-demand) | Yes (deferred) |

**Onboarding strategy:**
- First launch downloads faster-whisper `medium.en` automatically with progress bar
- pyannote deferred — prompted only when user first clicks "Annotate speakers"
- Vast.ai endpoint URL + API key entered in Settings (no download needed)
- All downloads resumable on failure

**Vast.ai setup (user responsibility, documented in README):**
1. Rent a GPU instance (RTX 4090 or A100 recommended, ~$0.15–0.50/hr)
2. Deploy Qwen3-8B via vLLM with `--api-key` flag (OpenAI-compatible)
3. Paste instance URL + key into Chronicler Settings → Post-processing

---

## Vast.ai Integration

- FastAPI calls the user-configured endpoint (`POST /v1/chat/completions`, OpenAI-compat)
- Single prompt per meeting: correct transcript errors + produce prose summary
- Response parsed into two sections: `corrected_transcript` and `summary`
- Health-check: `GET /v1/models` before each post-processing call
- If unreachable: raw live transcript shown as-is, summary shows "GPU endpoint not available"
- Endpoint URL + API key in `%APPDATA%\Chronicler\settings.json` (never committed)
- User manages instance lifecycle on Vast.ai — Chronicler does not auto-start/stop instances

**Recommended Vast.ai instance spec:**
- GPU: RTX 4090 or A100 (Qwen3-8B at fp16 fits in 16 GB VRAM)
- Image: `vllm/vllm-openai:latest`
- Start cmd: `vllm serve Qwen/Qwen3-8B --api-key <key> --host 0.0.0.0 --port 8000`

---

## Storage Layout

```
%APPDATA%\Chronicler\
├── meetings.db          # SQLite: all structured data
├── audio\               # WAV files, one per meeting
│   └── <meeting-id>.wav
├── models\              # Whisper model cache (managed by faster-whisper)
└── settings.json        # Vast.ai endpoint URL + key, HF token, preferences
```

### SQLite schema

```sql
CREATE TABLE meetings (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    started_at  DATETIME NOT NULL,
    ended_at    DATETIME,
    audio_path  TEXT,
    status      TEXT NOT NULL  -- 'recording' | 'processing' | 'completed' | 'error'
);

CREATE TABLE transcripts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    meeting_id  TEXT NOT NULL REFERENCES meetings(id),
    text        TEXT NOT NULL,
    kind        TEXT NOT NULL,  -- 'live' | 'final' | 'diarized'
    created_at  DATETIME NOT NULL
);

CREATE TABLE summaries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    meeting_id  TEXT NOT NULL REFERENCES meetings(id),
    text        TEXT NOT NULL,
    model       TEXT NOT NULL,
    created_at  DATETIME NOT NULL
);
```

---

## Harness Overlay Structure

New standalone repo at `~/Projects/chronicler`, initialized with harness files:

```
chronicler/
├── CLAUDE.md                          # Harness agent guide (adapted for Chronicler)
├── README.md
├── docs/
│   ├── HARNESS.md
│   ├── FEATURE_INTAKE.md
│   ├── ARCHITECTURE.md                # Full design (this plan, post-approved)
│   ├── TEST_MATRIX.md
│   ├── HARNESS_BACKLOG.md
│   ├── stories/                       # Story packets per feature
│   ├── decisions/                     # ADRs
│   └── product/                       # Product contracts
├── .claude/
│   └── skills/                        # harness-git-* skills copied from harness-experimental
├── frontend/                          # Tauri 2 + React + Vite
│   ├── src/                           # React UI components
│   └── src-tauri/                     # Rust core (WASAPI, audio pipeline)
├── backend/                           # FastAPI sidecar
│   ├── app/
│   │   ├── main.py
│   │   ├── transcription.py           # faster-whisper integration
│   │   ├── diarization.py             # whisperx + pyannote (on-demand)
│   │   ├── postprocess.py             # Vast.ai OpenAI-compat client (correct + summarize)
│   │   └── db.py                      # SQLite / aiosqlite
│   └── requirements.txt
└── scripts/
    └── download-models.py             # Whisper medium.en prefetch for first launch
```

---

## Key Implementation Risks

1. **WASAPI loopback on Windows** — requires `AUDCLNT_STREAMFLAGS_LOOPBACK`. Reference
   `meetily/frontend/src-tauri/src/audio/devices/platform/windows.rs` directly; do not
   re-invent this.

2. **pyannote HF token UX** — must be prompted only when user first requests diarization.
   Token stored in `settings.json`, never in the repo.

3. **Vast.ai endpoint unreachable** — app must degrade gracefully: raw live transcript
   shown, summary section displays a clear "configure GPU endpoint in Settings" prompt.

4. **faster-whisper medium.en lagging under load** — fallback to `small.en` configurable
   in Settings. Default `medium.en` covers Indian/Chinese English accent quality.

5. **Ring buffer audio sync (mic + loopback)** — two WASAPI streams at possibly different
   sample rates. Use meetily's `AudioMixerRingBuffer` pattern (50ms alignment windows,
   48kHz normalization).

6. **Vast.ai prompt structure** — single prompt must reliably produce two distinct sections.
   Use structured output or a delimited format (e.g. `<transcript>...</transcript>` +
   `<summary>...</summary>`) to make parsing deterministic.

---

## Verification Plan

### End-to-end smoke test
1. Launch app → FastAPI sidecar starts → no errors in console
2. Click **Record** → audio level meters show both mic and loopback activity
3. Speak for 30 seconds → live transcript appears with < 5 s lag
4. Click **Stop** → corrected transcript and prose summary appear (requires Vast.ai endpoint)
5. Click **"Annotate speakers"** → progress bar, speaker labels appear in transcript

### Component tests
- `POST /transcribe` with a test WAV → assert non-empty transcript text
- `POST /process` with a test transcript → assert `corrected_transcript` and `summary` fields
- Vast.ai health check: `GET /v1/models` → responds within 5 s
- SQLite: after a completed meeting, assert rows in all three tables
- VAD filter: silent audio chunks must not be forwarded to FastAPI
- Vast.ai unreachable: assert raw transcript shown and summary shows graceful error message

---

## Process Note (plan-mode / brainstorming skill conflict)

The brainstorming skill's terminal state is: write spec to
`docs/superpowers/specs/YYYY-MM-DD-chronicler-design.md`, commit, invoke `writing-plans`.
Plan mode only permits edits to this file and forbids commits.

Resolution after `ExitPlanMode`:
1. `mkdir -p ~/Projects/chronicler` and initialize harness overlay
2. Move this design to `docs/superpowers/specs/2026-05-18-chronicler-design.md`
3. Commit
4. Invoke `writing-plans` skill to produce the story-level implementation plan
