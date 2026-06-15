"""Entry point for the Chronicler backend sidecar (PyInstaller target)."""
import argparse

import uvicorn

from app.main import app

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Chronicler backend server")
    parser.add_argument("--port", type=int, default=8000)
    args = parser.parse_args()
    uvicorn.run(app, host="127.0.0.1", port=args.port)
