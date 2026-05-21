#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== ESLint ==="
(cd "$ROOT/frontend" && pnpm run lint)

echo "=== TypeScript typecheck ==="
(cd "$ROOT/frontend" && pnpm run typecheck)

echo "=== Rust unit tests ==="
(cd "$ROOT/frontend/src-tauri" && cargo test)

echo ""
echo "=== validate:quick passed ==="
