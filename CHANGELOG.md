# Changelog

### Unreleased
- Added
  - MIT-0 license for the template, so projects made from it don't have to keep its copyright notice.
  - `/setup-template` reminds you to turn on private vulnerability reporting, which `SECURITY.md` relies on.
- Fixed
  - The binary greets names that are not valid Unicode, showing `�` for the invalid bytes, instead of panicking.
  - Claude Code hooks now run in a project whose path contains spaces, in a checkout with Windows line endings, and when Claude Code runs them through PowerShell.
  - The published crate no longer includes `.claude/`, `.github/`, or the hook tests.
  - `/release` no longer asks for permission to run the hook tests.
  - `/setup-template` is no longer suggested in forks of the template, which contributors use to send changes back.
  - On Windows, the permission rules also cover commands Claude Code runs through PowerShell, and `/release` starts without Git Bash.
  - Claude Code is also denied `cargo publish` when a toolchain or flag comes first, as in `cargo +stable publish`.
  - `/release` creates an annotated tag, so the `git push --follow-tags` it prints pushes the tag too.
  - The `code-reviewer` agent reviews new files that haven't been added to git yet.
  - `/setup-template` no longer leaves rules describing removed code after a library-only, binary-only, or empty setup.
- Changed
  - The Claude Code hooks are a small Rust package run through `cargo` instead of Python scripts, so they work without Python, including on macOS, whose built-in `python3` was too old for them.
  - Claude Code only calls a change done once `cargo doc --no-deps` also finishes without warnings, matching CI.

### 0.1.0
- Added
  - Placeholder `greet` library function and `template_rs` binary.
  - Claude Code setup: `CLAUDE.md`, path-scoped rules, the `/release` skill, the `code-reviewer` agent, and a `cargo fmt` hook.
  - `/setup-template`, which personalizes a project created from the template and is suggested automatically until it has run.
  - Rules for Rust code, tests, comments, and this changelog.
  - Single test binary under `tests/` for unit and integration tests.
  - CI for formatting, lints, tests, docs, and the minimum supported Rust version.
  - Toolchain pinned to Rust 1.98.1.
