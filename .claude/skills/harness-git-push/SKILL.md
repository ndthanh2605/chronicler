---
name: harness-git-push
description: >
  Push branch to remote and create or update the PR, with a pre-push security
  scan and story packet Evidence linking. Use instead of commit-commands:commit-push-pr
  in this project — this skill scans for sensitive files, creates a PR referencing
  the story packet, and records the PR URL in the story Evidence section.
  Invoke after committing to push work and open or update the PR.
---

# Harness Git Push

## Goal

Push the current branch to origin and ensure a PR is open and linked in the
story packet Evidence section.

## Inputs

- One or more commits on a feature branch
- Story packet file path

## Steps

1. Run `git status` — confirm the working tree is clean (no untracked or
   modified files that should have been committed).

2. Check for sensitive files about to be pushed:

   ```bash
   git diff --name-only origin/main..HEAD 2>/dev/null \
     | grep -E '(\.env([./]|$)|id_rsa|\.(key|pem|p12|crt|secret)$|(^|/)secrets/)'
   ```

   If any match: abort, remove or `.gitignore` the file, re-commit, then
   return to step 1. If `origin/main` does not exist locally, run
   `git fetch origin main` first.

3. Push the branch:

   ```bash
   git push -u origin HEAD
   ```

   If rejected (non-fast-forward): stop and run the `harness-git-pull` skill
   to sync with origin/main first, then return to step 1 of this skill.

4. Check whether a PR already exists:

   ```bash
   gh pr view --json url,state 2>/dev/null
   ```

   - No output or error → create a new PR. Substitute the real story id and
     title — never push literal `US-XXX`:
     ```bash
     gh pr create --title "US-001: actual story title here" --body "$(cat <<'EOF'
     Implements US-001 (real story id).

     Story: docs/stories/US-001-actual-story.md
     Workpad: docs/stories/US-001-actual-story.workpad.md
     EOF
     )"
     ```
   - PR exists → update description if needed:
     ```bash
     gh pr edit --body "updated body"
     ```

5. Copy the PR URL. Open the story packet and record it in `## Evidence`:

   ```
   PR: https://github.com/<owner>/<repo>/pull/<N>
   ```

## Output

Branch pushed to origin. PR open and URL recorded in story packet Evidence.

## Related Skills

- `harness-git-pull`: run this first if push is rejected (non-fast-forward).
- `harness-git-land`: run this after the PR is approved and CI is green.
