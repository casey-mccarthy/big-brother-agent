---
description: Run linter, fix issues, run tests, and push to PR
allowed-tools: Bash(cargo:*), Bash(git:*), Bash(gh:*), Read, Edit, Glob, Grep
---

## Context

- Current branch: !`git branch --show-current`
- Git status: !`git status --short`
- Existing PR: !`gh pr view --json number,title 2>/dev/null || echo "No PR exists"`

## Your Task

Perform linting, testing, and PR management:

1. **Format code**: Run `cargo fmt` to fix formatting issues
2. **Run clippy**: Run `cargo clippy -- -D warnings` and fix any errors
3. **Run tests**: Run `cargo test` and fix any failures
4. **Commit changes**: If there are changes, commit with a descriptive message
5. **Push to PR**:
   - If a PR exists for this branch, push the changes
   - If no PR exists, create one with `gh pr create`

Fix any issues you encounter during linting or testing before proceeding to commit.
