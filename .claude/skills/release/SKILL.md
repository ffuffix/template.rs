---
name: release
description: Prepare a release. Verifies the project, bumps the version, updates the changelog, commits, and tags. Never pushes or publishes.
disable-model-invocation: true
argument-hint: "<version>"
arguments: [version]
allowed-tools: Bash(git status *) Bash(git tag *) Bash(git add *) Bash(git commit *) Bash(cargo package *)
---

Prepare release `$version`.

## Current state

- Uncommitted changes (empty means clean): !`git status --short`
- Latest tags: !`git tag --list --sort=-creatordate | head -n 5`

## Steps

1. Stop and report if the working tree is not clean, or `$version` is not greater than the `version` in `Cargo.toml`.
2. Run every check in [checklist.md](checklist.md). Stop at the first failure and show its output.
3. Set `version` in `Cargo.toml` to `$version`, then run `cargo check` so `Cargo.lock` picks up the change.
4. In `CHANGELOG.md`, rename `### Unreleased` to `### $version`. If that section is missing or has no entries, stop and ask what to write.
5. Commit with the message `release: v$version` and tag the commit `v$version`.
6. Print the commands that finish the release, and do not run them:
   - `git push --follow-tags`
   - `cargo publish`
