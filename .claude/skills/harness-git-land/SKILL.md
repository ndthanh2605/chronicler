---
name: harness-git-land
description: >
  Safely merge an approved PR and close out the story packet. Use instead of
  manually merging in this project — this skill verifies approval and CI, detects
  the project's merge convention, transitions the story status to done, updates
  TEST_MATRIX.md, and handles branch-protected main by opening a follow-up close PR.
  Invoke when a story is in merging status and the PR has human approval.
---

# Harness Git Land

## Goal

Merge a PR that has been approved by a human reviewer, verify the merge
succeeded, and close out the story packet. Handle branch-protected `main`.

## Prerequisites

- Story status is `merging` (human approved PR and set status)
- PR has at least one approved review
- CI is green on the latest commit

## Steps

1. Confirm PR approval and CI status with a single pass/fail check:

   ```bash
   gh pr view --json state,reviews,statusCheckRollup --jq '
     if .state != "OPEN" then "FAIL: state is \(.state)"
     elif ([.reviews[]? | select(.state=="APPROVED")] | length) < 1
       then "FAIL: no approved review"
     elif ([.statusCheckRollup[]? | select(.conclusion!=null and .conclusion!="SUCCESS")] | length) > 0
       then "FAIL: failing checks: \([.statusCheckRollup[] | select(.conclusion!=null and .conclusion!="SUCCESS") | .name] | join(","))"
     else "PASS"
     end
   '
   ```

   If output is not `PASS`: stop. Record the failure in workpad Notes.

2. Determine the project's merge convention from recent merged PRs:

   ```bash
   gh pr list --state merged --limit 5 --json title,mergeCommit \
     --jq '[.[] | select(.mergeCommit != null) | .mergeCommit.oid] | length'
   ```

   If the number is high relative to PR count and squash is the team's
   stated convention, use `--squash`. If unsure, ask the human. Default
   when the project has no history: `--squash`.

3. Merge the PR and delete the branch:

   ```bash
   gh pr merge --squash --delete-branch
   ```

   (Replace `--squash` with `--merge` if the project convention requires it.)

4. Switch to main and pull the merged changes:

   ```bash
   git checkout main
   git pull origin main
   ```

5. Verify the merge is present in main:

   ```bash
   git log --oneline -5
   ```

   Confirm the story's commit (or squash commit) appears in the log.

6. Update the story packet:

   - Set `## Status` to `done`.
   - Add to `## Evidence`:
     ```
     Merged: <PR URL> at <short-sha> on YYYY-MM-DD
     ```

7. Update `docs/TEST_MATRIX.md`: set the story's row status to `implemented`
   and add the merge SHA as the evidence reference.

8. Decide how to ship the doc-close commit based on whether `main` is
   branch-protected. Detect:

   ```bash
   gh api "repos/{owner}/{repo}/branches/main/protection" \
     --jq '.required_status_checks // .required_pull_request_reviews // empty' 2>/dev/null
   ```

   - **No output → main is unprotected.** Commit directly:

     ```bash
     git add docs/stories/US-XXX.md docs/TEST_MATRIX.md
     git commit -m "$(cat <<'EOF'
     docs(US-XXX): mark done, record merge evidence

     Why: story is complete and merged; closing out packet and test matrix.

     Co-Authored-By: Claude <noreply@anthropic.com>
     EOF
     )"
     git push origin main
     ```

   - **Output non-empty → main is protected.** Open a follow-up close PR:

     ```bash
     git checkout -b docs/US-XXX-close
     git add docs/stories/US-XXX.md docs/TEST_MATRIX.md
     git commit -m "$(cat <<'EOF'
     docs(US-XXX): mark done, record merge evidence

     Why: closing out packet and test matrix after the implementation PR
     merged. Branch protection on main requires this as a separate PR.

     Co-Authored-By: Claude <noreply@anthropic.com>
     EOF
     )"
     git push -u origin HEAD
     CLOSE_PR_URL=$(gh pr create --title "docs(US-XXX): close story" --body "Closes story US-XXX after merge. See <impl-PR-URL>." | tail -1)
     gh pr merge --auto --squash --delete-branch || true
     ```

     Record `CLOSE_PR_URL` in the story Evidence under a `Close PR:` line.

## Output

PR merged. Feature branch deleted. Story status `done`. PR URL and merge SHA
recorded in Evidence. Test matrix row set to `implemented`. Doc-close commit
is on main (unprotected) or in an auto-merging follow-up PR (protected).

## Related Skills

- `harness-git-push`: if the branch needed a final update before merge, run push first.
- `harness-git-pull`: if main has advanced since the branch was last synced.
