# Chronicler

Private, local-first Windows desktop app for transcribing and summarizing meetings and videos.

## What it does

- Captures live microphone and system audio (WASAPI loopback)
- Streams real-time transcription during recording
- After stopping: sends transcript to a user-managed GPU for correction + prose summary
- On-demand speaker annotation (diarization) via pyannote

## Status

Harness v0 — no application code yet. See `docs/stories/` for planned work.

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (Rust) |
| UI | React 18 + Vite + TypeScript |
| Backend | FastAPI (Python sidecar) |
| Live transcription | faster-whisper `medium.en` |
| Post-processing | Vast.ai GPU (vLLM + Qwen3-8B) |
| Diarization | whisperx + pyannote 3.1 (on-demand) |
| Persistence | SQLite |

## Setup

### Prerequisites

- Windows 10/11
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) (Rust, Visual Studio Build Tools)
- Python 3.11+
- A [Vast.ai](https://vast.ai) account with a running vLLM instance (for post-processing)

### First run

```powershell
# Install Python dependencies
cd backend && pip install -r requirements.txt

# Download Whisper model (~500 MB)
python scripts/download-models.py

# Start backend
cd backend && uvicorn app.main:app --port 8910

# Start Tauri app
cd frontend && pnpm install && pnpm run tauri:dev
```

### Vast.ai configuration

1. Rent a GPU instance (RTX 4090 or A100 recommended)
2. Deploy with: `vllm serve Qwen/Qwen3-8B --api-key <your-key> --host 0.0.0.0 --port 8000`
3. Open Chronicler → Settings → Post-processing → paste endpoint URL + API key

## Development

See `docs/HARNESS.md` for the agent operating model and `docs/FEATURE_INTAKE.md` before starting any task.
