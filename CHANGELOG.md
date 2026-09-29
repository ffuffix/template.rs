# Changelog

### Unreleased
- Fixed
  - The binary greets names that are not valid Unicode, showing `�` for the invalid bytes, instead of panicking.
  - Claude Code hooks now run in a project whose path contains spaces.
  - The published crate no longer includes `.claude/`, `.github/`, or the hook tests.
  - `/release` no longer asks for permission to run the hook tests.

### 0.1.0
- Added
  - Placeholder `greet` library function and `template_rs` binary.
  - Claude Code setup: `CLAUDE.md`, path-scoped rules, the `/release` skill, the `code-reviewer` agent, and a `cargo fmt` hook.
  - `/setup-template`, which personalizes a project created from the template and is suggested automatically until it has run.
  - Rules for Rust code, tests, comments, and this changelog.
  - Single test binary under `tests/` for unit and integration tests.
  - CI for formatting, lints, tests, docs, and the minimum supported Rust version.
  - Toolchain pinned to Rust 1.98.1.
