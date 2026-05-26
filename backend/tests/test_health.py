"""Tests for AC4 (DB creation) and AC5 (last_seen_at SELECT-before-INSERT)."""
from pathlib import Path

import aiosqlite
import pytest
import pytest_asyncio
from httpx import ASGITransport, AsyncClient


@pytest.fixture
def db_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    path = tmp_path / "test.db"
    monkeypatch.setenv("CHRONICLER_DB_PATH", str(path))
    return path


@pytest_asyncio.fixture
async def client(db_file: Path) -> AsyncClient:  # type: ignore[misc]
    # ASGITransport does not emit ASGI lifespan events, so call init_db explicitly.
    from app.db import init_db
    from app.main import app

    await init_db()
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as c:
        yield c  # type: ignore[misc]


# ── AC4: DB creation ──────────────────────────────────────────────────────────


@pytest.mark.asyncio
async def test_db_created_on_first_init(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    path = tmp_path / "fresh.db"
    monkeypatch.setenv("CHRONICLER_DB_PATH", str(path))
    from app.db import init_db

    assert not path.exists()
    await init_db()
    assert path.exists()


@pytest.mark.asyncio
async def test_db_reinit_does_not_truncate(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    path = tmp_path / "persist.db"
    monkeypatch.setenv("CHRONICLER_DB_PATH", str(path))
    from app.db import init_db

    await init_db()
    async with aiosqlite.connect(path) as db:
        await db.execute("INSERT INTO config (key, value) VALUES ('sentinel', 'alive')")
        await db.commit()

    await init_db()  # second init must not truncate

    async with aiosqlite.connect(path) as db:
        async with db.execute("SELECT value FROM config WHERE key = 'sentinel'") as cur:
            row = await cur.fetchone()
    assert row is not None
    assert row[0] == "alive"


# ── AC5: last_seen_at round-trip ─────────────────────────────────────────────


@pytest.mark.asyncio
async def test_health_first_call_returns_null(client: AsyncClient) -> None:
    resp = await client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert data["last_seen_at"] is None


@pytest.mark.asyncio
async def test_health_returns_previous_timestamp(client: AsyncClient, db_file: Path) -> None:
    """Second call must return exactly the timestamp written by the first call."""
    await client.get("/health")

    # Read the value that was persisted by call 1
    async with aiosqlite.connect(db_file) as db:
        async with db.execute(
            "SELECT value FROM config WHERE key = 'last_seen_at'"
        ) as cur:
            row = await cur.fetchone()
    assert row is not None
    t1 = row[0]

    resp2 = await client.get("/health")
    assert resp2.json()["last_seen_at"] == t1


@pytest.mark.asyncio
async def test_health_status_is_always_ok(client: AsyncClient) -> None:
    for _ in range(3):
        resp = await client.get("/health")
        assert resp.json()["status"] == "ok"
