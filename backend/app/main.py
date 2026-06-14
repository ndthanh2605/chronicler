from contextlib import asynccontextmanager
from datetime import datetime, timezone
from typing import Any

import aiosqlite
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.db import get_db_path, init_db


@asynccontextmanager
async def lifespan(app: FastAPI):  # type: ignore[type-arg]
    await init_db()
    yield


app = FastAPI(lifespan=lifespan)

# The sidecar is reached by the Tauri webview from a different origin than
# 127.0.0.1:<port> (http://localhost:1420 in `tauri dev`, http://tauri.localhost
# / tauri://localhost in a built app), so the browser enforces CORS on `fetch`.
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:1420",
        "http://tauri.localhost",
        "tauri://localhost",
    ],
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health() -> dict[str, Any]:
    """Return previous last_seen_at (null on first call) and write current timestamp."""
    db_path = get_db_path()
    async with aiosqlite.connect(db_path) as db:
        async with db.execute(
            "SELECT value FROM config WHERE key = 'last_seen_at'"
        ) as cursor:
            row = await cursor.fetchone()
        previous: str | None = row[0] if row else None
        now = datetime.now(timezone.utc).isoformat()
        await db.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('last_seen_at', ?)",
            (now,),
        )
        await db.commit()
    return {"status": "ok", "last_seen_at": previous}
