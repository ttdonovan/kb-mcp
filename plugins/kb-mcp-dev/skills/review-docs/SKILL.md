---
name: review-docs
description: Hand a change to kb-mcp's docs (the book, docs/, README, CONTRIBUTING, CLAUDE.md, the plugins' skills) to the independent read-only docs-reviewer, which checks each claim against the code. Use when asked to review a docs change, or before proposing a commit or pull request that changes docs.
argument-hint: "[branch]"
context: fork
agent: kb-mcp-dev:docs-reviewer
---

Review the branch named here: `$0`. If that is empty or reads literally as a dollar sign and a zero, no branch was given: review the current branch. The repository is in `${CLAUDE_PROJECT_DIR}`. Say which branch you reviewed.

- The base: the branch below this one if it is in a stack (`gh stack view --json` lists them bottom to top), otherwise `main`.
- The change: `git diff <base>...<branch>` and `git log --oneline <base>..<branch>`, plus uncommitted work if reviewing the current branch.
- Checklist and report format: `${CLAUDE_PROJECT_DIR}/book/src/development/reviewing.md`.
- The log of earlier rounds, if it exists: `${CLAUDE_PROJECT_DIR}/.ai/reviews/docs/<branch, with each / replaced by ->.md`. Read it first; "The review log" in the checklist says what to do with it.

Do not modify any files. Return the review report only.
