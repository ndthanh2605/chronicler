# Architecture — Chronicler

The architecture reference for this project lives in two places:

- `CLAUDE.md` → Architecture section — stack, component ownership, data flows, thread budget,
  known constraints. Read this first.
- `docs/superpowers/specs/2026-05-18-chronicler-design.md` — full design plan with rationale,
  verification plan, and harness overlay structure.

This file holds extended detail that supplements `CLAUDE.md`.

## SQLite Schema

```sql
CREATE TABLE meetings (
    id          TEXT PRIMARY KEY,   -- UUID
    title       TEXT,
    started_at  INTEGER NOT NULL,   -- Unix ms
    stopped_at  INTEGER,
    status      TEXT NOT NULL       -- recording | processing | done | error
);

CREATE TABLE transcripts (
    id          TEXT PRIMARY KEY,
    meeting_id  TEXT NOT NULL REFERENCES meetings(id),
    speaker     TEXT,               -- null until diarization runs
    start_ms    INTEGER NOT NULL,
    end_ms      INTEGER NOT NULL,
    text        TEXT NOT NULL
);

CREATE TABLE summaries (
    id                   TEXT PRIMARY KEY,
    meeting_id           TEXT NOT NULL REFERENCES meetings(id),
    corrected_transcript TEXT NOT NULL,
    prose_summary        TEXT NOT NULL,
    model                TEXT NOT NULL,
    created_at           INTEGER NOT NULL
);

CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
    -- keys: vastai_endpoint, vastai_api_key, hf_token, audio_dir
);
```

## Vast.ai Integration

FastAPI reads `vastai_endpoint` and `vastai_api_key` from the `config` table.

Request format: OpenAI-compatible chat completion (the `model` field is ignored by `vllm serve`).
Prompt: system instruction + full raw transcript text. The assistant returns JSON with two fields:
`corrected_transcript` and `prose_summary`.

FastAPI must return HTTP 503 with a human-readable message when `vastai_endpoint` is not
configured. Do not raise an unhandled exception or return a 500.

## Audio File Convention

- Path: `%APPDATA%\Chronicler\audio\<meeting-id>.wav`
- Format: 16 kHz mono PCM (required by both faster-whisper and whisperx)
- Rust writes the WAV header on recording start and appends raw frames continuously
- Files are retained indefinitely — they are required for diarization reprocessing on demand
- The `audio_dir` config key allows overriding the default location via Settings
