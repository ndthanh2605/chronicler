# ADR-0004 — Python type checker: pyright

- **Status:** Accepted
- **Date:** 2026-05-24
- **Initiative:** `docs/stories/phase-0-scaffold/initiative.md`
- **Stories:** `S02-fastapi-sidecar-health-sqlite` (AC7)

## Context

S02 AC7 requires `validate:quick` to run a Python type checker against `backend/`.
Two credible options exist for a FastAPI + aiosqlite project: mypy and pyright.

## Alternatives considered

**A. mypy** — the reference implementation; strict mode is very thorough; slower on
large codebases; integrates less smoothly with VS Code out of the box without the
Pylance extension.

**B. pyright** — written in TypeScript; fast (sub-second incremental checks); ships
inside Pylance (the default VS Code Python extension); first-class support for
FastAPI's dependency injection typing and `Annotated` patterns; `basic` mode is
appropriate for Phase 0 bootstrapping.

## Decision

**B** — pyright in `typeCheckingMode = "basic"`.

## Consequences

- `validate:quick` runs `pyright app` from `backend/`.
- `pyproject.toml` contains the `[tool.pyright]` section with `pythonVersion = "3.11"`.
- If the project later needs stricter checks, the mode can be raised to `"standard"`
  or `"strict"` without changing tooling.
- mypy is not used; do not introduce it alongside pyright (split type-check configs
  cause false confidence).

## References

- pyright docs: https://github.com/microsoft/pyright
- FastAPI typing guide: https://fastapi.tiangolo.com/python-types/
