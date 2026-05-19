---
name: harness-git-pull
description: >
  Sync current branch with origin/main before editing, and record the result
  in the story workpad sibling. Use instead of a plain git pull in this project
  — this skill includes conflict resolution guidance and writes the sync result
  (including HEAD SHA) to the workpad Notes section as required by the harness.
  Invoke before starting or continuing any implementation work on a story branch.
---

# Harness Git Pull

## Goal

Bring the current branch up to date with `origin/main` before starting or
continuing work. Record the sync result in the story workpad sibling file.

## Inputs

- A checked-out branch (may be behind origin/main)
- An open story packet and its workpad sibling
  (`docs/stories/US-XXX.workpad.md` for Normal-lane stories,
  `docs/stories/US-XXX/workpad.md` inside the folder for High-Risk).

## Steps

1. Fetch the latest remote state without changing the working tree:

   ```bash
   git fetch origin
   ```

2. Check how far behind the branch is:

   ```bash
   git log HEAD..origin/main --oneline
   ```

   If no output: already up to date. Record in workpad Notes and stop.

3. Default convention is `merge`. If the project explicitly uses rebase
   (check `git config pull.rebase` or recent merged PRs), use:

   ```bash
   git rebase origin/main
   ```

   Otherwise:

   ```bash
   git merge origin/main
   ```

4. If conflicts appear:

   - Run `git status` to list conflicted files.
   - Open each conflicted file and resolve it explicitly. Do not blindly
     accept `--ours` or `--theirs` without understanding what each side does.
   - Run `git add <resolved-file>` for each resolved file.
   - Run `git merge --continue` (or `git rebase --continue`).

5. Get the resulting HEAD short SHA:

   ```bash
   git rev-parse --short HEAD
   ```

6. Record in the **workpad sibling** Notes section:

   ```
   YYYY-MM-DD: Synced with origin/main. Result: clean | conflicts-resolved. HEAD: <short-sha>
   ```

   Do not edit the story packet for sync notes — keep the packet a stable
   contract.

## Output

Branch is up to date with origin/main. Workpad Notes records the sync result
and resulting HEAD SHA.

## Related Skills

- `harness-git-push`: run this skill when push is rejected due to non-fast-forward.
- `harness-git-commit`: run after pull to commit any conflict-resolution changes.
