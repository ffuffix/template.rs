<p align="center">
  <img src="assets/header_contributing.svg" alt="Contributing" width="100%">
</p>

Thanks for taking the time to contribute. This guide covers what a change needs before it can be merged.

## Before you start

For anything bigger than a small fix, open an issue first so we can agree on the approach before you write the code. Keep each pull request to one change.

## Setup

Install Rust through [rustup](https://rustup.rs). The toolchain is pinned in `rust-toolchain.toml`, and the first `cargo` command installs it.

## Making a change

- **Tests.** New behaviour needs a test. All tests live under `tests/`: `tests/unit/` for the library's public API and `tests/integration/` for the binary. Declare a new test file in its directory's `mod.rs`, or it is silently not compiled.
- **Changelog.** Describe user-visible changes under `### Unreleased` in `CHANGELOG.md`, grouped as `Added`, `Fixed`, and `Removed`, then optional categories such as `Deprecated`.
- **Rust version.** Code must build on the `rust-version` in `Cargo.toml`, which is older than the pinned toolchain. CI checks this.

## Before opening a pull request

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same checks, plus the docs build, the same three checks for the Claude Code hooks in `.claude/hooks/`, and a build on the minimum Rust version. In the pull request, say what changed and why, and link the issue if there is one.

## Using Claude Code

This project is developed with [Claude Code](https://code.claude.com/docs), and you are welcome to use it too. The `.claude/` folder teaches it the conventions above, formats the Rust files it edits, and blocks it from publishing the crate. The first time you open the folder, Claude Code asks you to trust it, because `.claude/settings.json` pre-approves cargo commands and runs that formatting hook. Other assistants that read `AGENTS.md` are pointed to the same instructions.

Whatever tools you use, you are responsible for what you submit. Read and understand every change before you open a pull request.
