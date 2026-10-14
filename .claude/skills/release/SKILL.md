---
name: release
description: Prepare a release. Bumps the version, updates the changelog, lands the release commit with `cargo land`, and tags it. Never pushes the tag or publishes.
disable-model-invocation: true
argument-hint: "<version>"
arguments: [version]
allowed-tools: Bash(git status *) Bash(git for-each-ref *) Bash(git ls-tree *) Bash(git switch *) Bash(git add *) Bash(git commit *) Bash(git tag *) Bash(cargo land) PowerShell(git status *) PowerShell(git for-each-ref *) PowerShell(git ls-tree *) PowerShell(git switch *) PowerShell(git add *) PowerShell(git commit *) PowerShell(git tag *) PowerShell(cargo land)
---

Prepare release `$version`.

## Current state

- Uncommitted changes (empty means clean): !`git status --short`
- Latest tags: !`git for-each-ref --sort=-creatordate --count=5 --format="%(refname:short)" refs/tags`
- Open decisions on main (empty means none): !`git ls-tree -r --name-only main -- .claude/decisions`

## Steps

1. Stop and report if the working tree is not clean.
2. If decisions are open, list their questions and ask whether to release anyway. If `.github/assets/body.svg` exists, compare what its alt text in `README.md` lists with what the project now contains. If it is out of date, tell the user the image needs redrawing, and ask the same.
3. Start the release from the newest `main`, not from whatever is checked out: `git switch -c release-$version main`. Then stop and report if `$version` is not greater than the `version` in `Cargo.toml`.
4. Set `version` in `Cargo.toml` to `$version`, then run `cargo check` so `Cargo.lock` picks up the change.
5. In `CHANGELOG.md`, rename `### Unreleased` to `### $version`. If that section is missing or has no entries, stop and ask what to write.
6. Commit with the message `release: v$version`, then land it as the Landing section of `.claude/CLAUDE.md` says. `cargo land` runs every check; if one fails, stop and show its output.
7. Tag the landed commit with `git tag -a v$version -m v$version`. The tag must be annotated, so it records who tagged the release and when.
8. Print the commands that finish the release, and do not run them:
   - `git push origin v$version`
   - `cargo publish`
