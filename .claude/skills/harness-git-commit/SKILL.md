---
name: harness-git-commit
description: >
  Produce a well-formed conventional commit with security checks and rationale.
  Use instead of commit-commands:commit in this project — this skill enforces
  harness-specific staging discipline, conventional commit format, required Why:
  rationale, and a post-commit verification step. Invoke whenever committing
  changes in the harness repository.
---

# Harness Git Commit

## Goal

Stage explicitly chosen changes and commit them with a conventional commit
message that explains both what changed and why.

## Inputs

- One or more changed files
- Knowledge of the story or task being implemented

## Steps

1. Run `git status` to review all changed files. Confirm only intended files
   will be committed. Never include `.env*`, `id_rsa*`, `*.key`, `*.pem`,
   `*.p12`, `*.crt`, `*.secret`, or anything under `secrets/`.

2. **Stage explicitly.** Use `git add <file>` for each intended file.
   Never use `git add -A` or `git add .` — both pick up unintended changes
   and have caused secret leaks in the past.

3. Run `git diff --staged` and review the exact changes that will be
   committed. If anything unintended is present, unstage it with
   `git restore --staged <file>` and return to step 2.

4. Draft the commit message using conventional commit format:

   ```
   type(scope): short summary under 72 chars

   Why: one or two sentences explaining the reason for this change.

   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

   Valid types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`.

   The `Why:` line is required. "fix bug" or "add feature" with no rationale
   is not acceptable output. Replace `Claude` with the actual running model
   name (e.g., `Claude Sonnet 4.6`) if exact attribution is preferred.

5. Commit using a heredoc to preserve formatting:

   ```bash
   git commit -m "$(cat <<'EOF'
   type(scope): summary

   Why: reason.

   Co-Authored-By: Claude <noreply@anthropic.com>
   EOF
   )"
   ```

6. Run `git log -1 --oneline` to verify the commit was created correctly.

## Output

A single commit with: type, scope, summary, `Why:` rationale, Co-Authored-By
trailer. Only explicitly-staged files are included.

## Next

Run the `harness-git-push` skill to push the branch and update the PR.
