# Changelog

### Unreleased
- Added
  - MIT-0 license for the template, so projects made from it don't have to keep its copyright notice.
  - `/setup-template` reminds you to turn on private vulnerability reporting, which `SECURITY.md` relies on.
  - CI checks that the crate still builds from the files it would be published with.
  - Dependabot updates for the Rust toolchain, Cargo dependencies, and GitHub Actions.
  - `/setup-template` asks whether the crate will be published to crates.io, and drops the packaging checks if not.
  - A review gate for Claude Code. Claude can't finish a change until the `code-reviewer` agent has passed it, fixes what the review says is wrong without asking, and leaves the judgment calls for you in `.claude/decisions/`. It needs git 2.39 or newer.
  - `cargo land`, which puts a reviewed change on `main`: it rebases the branch, runs every CI check, moves `main`, and pushes it if there is a remote. Other coding agents land with `cargo land --reviewed` after reviewing by the same instructions.
  - `cargo ci`, which runs every check CI runs, including the minimum Rust version when that toolchain is installed.
  - Clippy warns about `todo!`, `unimplemented!`, and `dbg!`.
- Fixed
  - The binary greets names that are not valid Unicode, showing `�` for the invalid bytes, instead of panicking.
  - Claude Code hooks now run in a project whose path contains spaces, in a checkout with Windows line endings, and when Claude Code runs them through PowerShell.
  - The published crate no longer includes `.claude/`, `.github/`, or the hook tests.
  - `/release` no longer asks for permission to run the hook tests.
  - `/setup-template` is no longer suggested in forks of the template, which contributors use to send changes back.
  - On Windows, the permission rules also cover commands Claude Code runs through PowerShell, and `/release` starts without Git Bash.
  - Claude Code is also denied `cargo publish` when a toolchain or flag comes first, as in `cargo +stable publish`.
  - `/release` creates an annotated tag, which records who tagged the release and when.
  - The `code-reviewer` agent reviews new files that haven't been added to git yet.
  - `/setup-template` no longer leaves rules describing removed code after a library-only, binary-only, or empty setup.
- Changed
  - The Claude Code hooks are a small Rust package run through `cargo` instead of Python scripts, so they work without Python, including on macOS, whose built-in `python3` was too old for them.
  - Claude Code only calls a change done once `cargo doc --no-deps` also finishes without warnings, matching CI.
  - The formatting hook formats only the file Claude edited, so edits stay fast in large projects and other files are left alone.
  - The `code-reviewer` agent sorts its findings into ones to fix now and ones for you to decide, looks for tech debt a change creates, and leaves running the checks to `cargo land`.
  - Claude commits on a branch rather than on `main`, and may make that branch in the same command, as in `git switch -c work && git commit`. `/release` lands the release commit through `cargo land` before tagging it.
  - `/release` starts from the newest `main`, and asks before releasing while decisions are open or the README image is out of date.

### 0.1.0
- Added
  - Placeholder `greet` library function and `template_rs` binary.
  - Claude Code setup: `CLAUDE.md`, path-scoped rules, the `/release` skill, the `code-reviewer` agent, and a `cargo fmt` hook.
  - `/setup-template`, which personalizes a project created from the template and is suggested automatically until it has run.
  - Rules for Rust code, tests, comments, and this changelog.
  - Single test binary under `tests/` for unit and integration tests.
  - CI for formatting, lints, tests, docs, and the minimum supported Rust version.
  - Toolchain pinned to Rust 1.98.1.
