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

A change is done when `cargo clippy --all-targets -- -D warnings` and `cargo test` both pass. If either fails, show its output rather than summarising it.

A hook runs `cargo fmt` after every edit to a `.rs` file, so formatting needs no manual step.

The toolchain is pinned in `rust-toolchain.toml`, but code must also build on the older `rust-version` in `Cargo.toml`. CI checks this and a local build does not, so do not use language or library features stabilised after that version.

## Tests

Tests never go in `src/`. Every Rust test lives in the single test binary under `tests/`: unit tests in `tests/unit/`, tests that run the binary in `tests/integration/`, shared helpers in `tests/common/`. `.claude/rules/testing.md` has the details and loads when you open a file there.

## Conventions

- Every user-visible change adds an entry under `### Unreleased` in `CHANGELOG.md`.
- Ask before adding a dependency.
- Lint levels live in the `[lints]` table in `Cargo.toml`. Change them there rather than with crate-level attributes.
