import os
from pathlib import Path

import aiosqlite


def get_db_path() -> Path:
    """Return the SQLite DB path, respecting CHRONICLER_DB_PATH for test overrides."""
    if "CHRONICLER_DB_PATH" in os.environ:
        return Path(os.environ["CHRONICLER_DB_PATH"])
    if os.name == "nt":
        appdata = os.environ.get("APPDATA") or str(Path.home() / "AppData" / "Roaming")
        base = Path(appdata) / "Chronicler"
    else:
        base = Path.home() / ".chronicler"
    base.mkdir(parents=True, exist_ok=True)
    return base / "chronicler.db"


async def init_db() -> None:
    async with aiosqlite.connect(get_db_path()) as db:
        await db.execute(
            "CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT)"
        )
        await db.commit()
