# Chronicler — Agent Entry Point

`AGENTS.md` is the cross-tool agent entry point (Codex, generic OpenAI-style
agents). The **authoritative agent guide for Chronicler is `CLAUDE.md`**.
Read it first; this file only adds GitNexus-tool guidance for once-application-
code-exists tasks.

## Read order (every session)

Follow the source-of-truth ordering in `CLAUDE.md` § "Source Of Truth". Short
form:

1. `README.md` — status.
2. `docs/HARNESS.md` — operating model.
3. `docs/FEATURE_INTAKE.md` — risk lane before any work.
4. `docs/ROADMAP.md` — phase context.
5. `docs/superpowers/specs/2026-05-18-chronicler-design.md` — approved architecture.
6. `docs/ARCHITECTURE.md` — component ownership.
7. `docs/stories/` — current story packets.
8. `docs/TEST_MATRIX.md` — behavior-to-proof.
9. `docs/decisions/` — ADRs.

When picking up an in-progress story, the workpad sibling overrides this order.

## Scope of the GitNexus rules below

The block between the `gitnexus:start` / `gitnexus:end` markers is **regenerated
by `npx gitnexus analyze`** — do not edit inside the markers; edits will be
clobbered on the next regeneration. The block describes discipline for
**editing application code**.

Current repo state (Phase 0 — Scaffold, per `docs/ROADMAP.md`): **no application
code exists yet.** The GitNexus index reports 105 nodes / 101 edges, but those
are derived from this repo's markdown docs, not code. The "MUST run impact
analysis before editing any symbol" rule has no symbols to apply to until
Story S01 (`docs/stories/phase-0-scaffold/S01-tauri-react-shell.md`) lands.

When the rules apply:

| Task | GitNexus rules apply? |
|---|---|
| Editing `docs/`, harness files, story packets, ADRs | No — follow `CLAUDE.md` + harness skills only |
| Editing application code under `frontend/` or `backend/` (post-S01) | Yes — full impact / context / rename discipline |
| Refactoring application code | Yes — `gitnexus_rename`, never find-and-replace |
| Investigating a bug in application code | Yes — `gitnexus_query` / `gitnexus_context` before grep |

## Skill / tool overlap with the harness

The harness git skills (`harness-git-pull`, `harness-git-commit`,
`harness-git-push`, `harness-git-land`, `harness-intake`) are the authoritative
git workflow. The GitNexus block's "run `gitnexus_detect_changes()` before
committing" rule slots **inside** `harness-git-commit` (between staging and
the commit), not in place of it. When in doubt, harness skills win.

## Keeping this file healthy

- The block below is auto-generated. Re-run `npx gitnexus analyze` after
  application-code commits to keep node/edge counts accurate.
- If GitNexus rules in the block ever contradict `CLAUDE.md` or
  `docs/HARNESS.md`, the harness wins. Log the friction in
  `docs/HARNESS_BACKLOG.md` so the regenerator template can be patched.

---

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **chronicler** (105 symbols, 101 relationships, 0 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## When Debugging

1. `gitnexus_query({query: "<error or symptom>"})` — find execution flows related to the issue
2. `gitnexus_context({name: "<suspect function>"})` — see all callers, callees, and process participation
3. `READ gitnexus://repo/chronicler/process/{processName}` — trace the full execution flow step by step
4. For regressions: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})` — see what your branch changed

## When Refactoring

- **Renaming**: MUST use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` first. Review the preview — graph edits are safe, text_search edits need manual review. Then run with `dry_run: false`.
- **Extracting/Splitting**: MUST run `gitnexus_context({name: "target"})` to see all incoming/outgoing refs, then `gitnexus_impact({target: "target", direction: "upstream"})` to find all external callers before moving code.
- After any refactor: run `gitnexus_detect_changes({scope: "all"})` to verify only expected files changed.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Tools Quick Reference

| Tool | When to use | Command |
|------|-------------|---------|
| `query` | Find code by concept | `gitnexus_query({query: "auth validation"})` |
| `context` | 360-degree view of one symbol | `gitnexus_context({name: "validateUser"})` |
| `impact` | Blast radius before editing | `gitnexus_impact({target: "X", direction: "upstream"})` |
| `detect_changes` | Pre-commit scope check | `gitnexus_detect_changes({scope: "staged"})` |
| `rename` | Safe multi-file rename | `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` |
| `cypher` | Custom graph queries | `gitnexus_cypher({query: "MATCH ..."})` |

## Impact Risk Levels

| Depth | Meaning | Action |
|-------|---------|--------|
| d=1 | WILL BREAK — direct callers/importers | MUST update these |
| d=2 | LIKELY AFFECTED — indirect deps | Should test |
| d=3 | MAY NEED TESTING — transitive | Test if critical path |

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/chronicler/context` | Codebase overview, check index freshness |
| `gitnexus://repo/chronicler/clusters` | All functional areas |
| `gitnexus://repo/chronicler/processes` | All execution flows |
| `gitnexus://repo/chronicler/process/{name}` | Step-by-step execution trace |

## Self-Check Before Finishing

Before completing any code modification task, verify:
1. `gitnexus_impact` was run for all modified symbols
2. No HIGH/CRITICAL risk warnings were ignored
3. `gitnexus_detect_changes()` confirms changes match expected scope
4. All d=1 (WILL BREAK) dependents were updated

## Keeping the Index Fresh

After committing code changes, the GitNexus index becomes stale. Re-run analyze to update it:

```bash
npx gitnexus analyze
```

If the index previously included embeddings, preserve them by adding `--embeddings`:

```bash
npx gitnexus analyze --embeddings
```

To check whether embeddings exist, inspect `.gitnexus/meta.json` — the `stats.embeddings` field shows the count (0 means no embeddings). **Running analyze without `--embeddings` will delete any previously generated embeddings.**

> Claude Code users: A PostToolUse hook handles this automatically after `git commit` and `git merge`.

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->