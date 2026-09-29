---
name: setup-template
description: Personalize a repository created from the template.rs template. Asks what the project is and which template features to keep, then renames the crate, rewrites the README and changelog, adds a license, removes unwanted features, and deletes itself.
disable-model-invocation: true
---

# Set up a project created from template.rs

Turn this repository into the user's own project, then delete this skill. Other coding agents can follow this file directly.

If `git remote get-url origin` points to `ffuffix/template.rs`, this is the template itself: stop and say so.

## 1. Ask

Ask everything in as few rounds as possible, offering the current value as the default where there is one.

**The project**

- Crate name, and a one-sentence description of what the project does.
- Library and binary (the template has both), library only, or binary only. With binary only, tests can exercise it only by running it, so suggest keeping a small library for the logic.
- License: `MIT OR Apache-2.0` (the Rust convention), MIT, Apache-2.0, or none. If one is chosen, the copyright holder's name.
- Keep the greeting example as a starting point, or start empty.

**Template features.** Each is kept unless the user says otherwise.

- The `code-reviewer` agent and the `/release` skill.
- The hook that runs `cargo fmt` after every edit.
- `AGENTS.md`, which points other coding agents to these instructions.
- `SECURITY.md`, and the acknowledgement time it promises (7 days).
- The designed headers on `CONTRIBUTING.md` and `SECURITY.md`, or plain Markdown headings.
- The toolchain pinned in `rust-toolchain.toml`, or the latest stable Rust (check with `rustup check`).

## 2. Apply

**Name.** It appears in `Cargo.toml`, in `use template_rs::` in `src/main.rs` and `tests/unit/`, in `CARGO_BIN_EXE_template_rs` in `tests/common/mod.rs` (the binary takes the package name), and in the titles of `.claude/CLAUDE.md` and `AGENTS.md`. Run `cargo check` afterwards so `Cargo.lock` follows.

**Package metadata.** Set `description`, `license`, and `repository` in `Cargo.toml`, deriving the repository URL from `git remote get-url origin`. Add the license text from its official source: `LICENSE-MIT` and `LICENSE-APACHE` for the dual license, otherwise `LICENSE`.

**Library or binary only.** For library only, delete `src/main.rs`, `tests/integration/`, `tests/common/`, and their `mod` lines in `tests/main.rs`. For binary only, move the logic into `src/main.rs`, delete `src/lib.rs`, `tests/unit/`, and its `mod` line.

**Starting empty.** Remove `greet` and `GreetError` along with the tests that use them, and delete helpers in `tests/common/` that become unused, because clippy runs with `-D warnings` and unused code fails it. Remove the placeholder line from `.claude/CLAUDE.md`.

**README.** Replace it with a short Markdown README: the project name as the title, the description, a getting started block with `cargo run` and `cargo test`, and a link to `.github/CONTRIBUTING.md`. Delete the template-only images `header.svg`, `body.svg`, `footer.svg`, and `use_template.svg` from `.github/assets/`.

**Changelog.** Replace the template's history with an empty `### Unreleased` section under the `# Changelog` title.

**Removing features.** A feature is only gone when nothing mentions it. After removing one, search the repository for its name and fix every remaining mention.

- `code-reviewer` and `/release`: `.claude/agents/code-reviewer.md` and `.claude/skills/release/`, plus their lines in `AGENTS.md`.
- The formatting hook: its `PostToolUse` entry in `.claude/settings.json`, `.claude/hooks/cargo-fmt.py`, `tests/hooks/test_cargo_fmt.py`, and the formatting hook mentions in `.claude/CLAUDE.md`, `AGENTS.md`, and `.github/CONTRIBUTING.md`. In `.claude/CLAUDE.md`, replace the hook line with an instruction to run `cargo fmt` after editing Rust files.
- `AGENTS.md`: the file, and the sentence about other assistants in `.github/CONTRIBUTING.md`.
- `SECURITY.md`: `.github/SECURITY.md` and `.github/assets/header_security.svg`.
- Designed headers: replace the image block at the top of `.github/CONTRIBUTING.md` with `# Contributing` and of `.github/SECURITY.md` with `# Security policy`, then delete `header_contributing.svg` and `header_security.svg`. If `.github/assets/` ends up empty, delete it.
- Latest stable toolchain: update `channel` in `rust-toolchain.toml` and the version in `.github/CONTRIBUTING.md` if it names one.

## 3. Finish

1. Delete the setup itself: `.claude/skills/setup-template/`, `.claude/hooks/setup-pending.py`, `tests/hooks/test_setup_pending.py`, the `SessionStart` entry in `.claude/settings.json`, and the `setup-template` line in `AGENTS.md`.
2. If `.claude/hooks/` is now empty, also delete it, the empty `hooks` block in `.claude/settings.json`, `tests/hooks/` and its row in `.claude/rules/testing.md`, the `Hook tests` step in `.github/workflows/ci.yml`, step 4 of `.claude/skills/release/checklist.md`, and the Python and hook test mentions in `.github/CONTRIBUTING.md`.
3. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`, plus `python3 -m unittest discover -s tests/hooks` if `tests/hooks/` still exists. Fix anything that fails.
4. Search for `template_rs`, `template.rs`, and `ffuffix`, and resolve any leftovers.
5. Summarize what changed and what was removed, and suggest a commit message. Commit only if the user asks.
