# template.rs

<!--
Loaded into every Claude Code session, so keep it under 200 lines and limited
to facts every session needs.
- Multi-step procedures go in .claude/skills/.
- Rules for one part of the tree go in .claude/rules/ with a `paths:` filter.
- Anything that must always happen goes in .claude/settings.json as a
  permission rule or hook. Prose here is guidance, not enforcement.
- Personal notes go in CLAUDE.local.md, which is gitignored.
HTML comments like this one are stripped before Claude sees the file.
-->

The greeting code in `src/` is a placeholder that will be replaced.

## Checks

`cargo ci` runs every check CI runs, for the crate and for the hooks package: rustfmt, clippy with `-D warnings`, the tests, the docs without warnings, the packaged crate, and a build on the minimum Rust version when that toolchain is installed. A change is done when it passes. If a check fails, show its output rather than summarising it.

A hook runs rustfmt on every `.rs` file Claude edits, so formatting needs no manual step. The hooks are their own package in `.claude/hooks/`, run through `cargo run`. `cargo ci` and `cargo land` are aliases for two of its subcommands, set in `.cargo/config.toml`. Run them from the repository root.

The toolchain is pinned in `rust-toolchain.toml`, but code must also build on the older `rust-version` in `Cargo.toml`. CI always checks this and `cargo ci` only when that toolchain is installed, so do not use language or library features stabilised after that version.

## Tests

Tests never go in `src/`. Every test of the crate lives in the single test binary under `tests/`: unit tests in `tests/unit/`, tests that run the binary in `tests/integration/`, shared helpers in `tests/common/`. The hooks package keeps its tests the same way under `.claude/hooks/tests/`. `.claude/rules/testing.md` has the details and loads when you open a file there.

## Landing a change

Finished work goes onto `main` without the user reading it, so review stands in for them. The hooks and `cargo land` need git 2.39 or newer.

1. Work on a branch. A hook denies commits on `main`.
2. When the change is done, run the `code-reviewer` agent in the foreground on everything since the branch left `main`, and wait for its report. A Stop hook enforces this: your turn can't end while the change has no clean review, and the hook names the commit to review from.
3. Fix every **Fix** finding without asking. The hook has whatever changed reviewed again. After three rounds it stops you, and you tell the user what is left.
4. Never act on an **Ask** finding unasked. Record each one as a decision, below, unless a file in `.claude/decisions/` already covers it.
5. Commit. The message body says what changed, what the review fixed, and which decisions it raised. Then run `cargo land`, never with `--reviewed`, which is for agents without these hooks. It refuses unreviewed work, rebases onto `main`, runs `cargo ci`, moves `main`, and pushes it when there is a remote. If it fails, fix the cause and land again. A fix that changes the diff gets reviewed again.
6. Report in three short parts: what changed; what the review fixed, one line each; and the decisions waiting for the user.

Land every finished change unless the user said not to.

### Decisions

A decision is one Markdown file in `.claude/decisions/`, named for its subject and committed with the change that raised it:

```markdown
# Accept tabs in names?

- **Where:** `src/lib.rs:12`
- **Found:** what the reviewer saw.
- **Now:** what the change does.
- **Instead:** the alternative.
```

Each session starts with a list of the open decisions. Mention them when you report. When the user rules on one, make the change they chose, or none to keep things as they are, and delete the file in that change.

## Conventions

- Every user-visible change adds an entry under `### Unreleased` in `CHANGELOG.md`.
- Ask before adding a dependency.
- Lint levels live in the `[lints]` table in `Cargo.toml`. Change them there rather than with crate-level attributes.
