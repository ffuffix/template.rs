---
name: setup-template
description: Personalize a repository created from the template.rs template. Offers a full setup (project details, conventions, and template features), a quick setup (project details only), or skipping it, then applies the answers, runs the checks, and deletes itself.
disable-model-invocation: true
---

# Set up a project created from template.rs

Turn this repository into the user's own project, then delete this skill. Other coding agents can follow this file directly.

If the repository that `git remote get-url origin` points to is named `template.rs`, whoever owns it, this is the template or a fork of it: stop and say so.

## How to ask

Ask in rounds grouped by topic, as multiple-choice questions wherever the options are known. Mark the template's current choice as the recommended option, so a user who is happy with the defaults can get through a round quickly. Before changing any file, summarize every answer and confirm once.

When confirming, say that most of the changes are inside `.claude/`, which Claude Code protects: the first write there asks for approval, and choosing to allow edits to this project's `.claude` folder for the session lets the rest of the setup finish without further prompts.

## 0. How much to set up

- **Full setup** (recommended): sections 1 to 3.
- **Quick setup**: section 1 only. Conventions and features stay as the template has them, but the name, README, and changelog still become the user's own.
- **Not now**: change nothing. Setup is suggested again next session.
- **Skip setup**: keep the project exactly as the template made it and stop suggesting setup. Apply only steps 1 to 3 of Finish, and mention that the README, changelog, and `LICENSE` still describe the template, including `/setup-template` itself.

## 1. The project

- Crate name, and a one-sentence description of what the project does.
- Library and binary (the template has both), library only, or binary only. With binary only, tests can exercise it only by running it, so suggest keeping a small library for the logic.
- Whether the crate will be published to crates.io (the template assumes it will). Libraries and command-line tools often are; services, games, and desktop apps usually aren't.
- License: `MIT OR Apache-2.0` (the Rust convention), MIT, Apache-2.0, or none. If one is chosen, the copyright holder's name. The MIT-0 `LICENSE` covers only the template, so it is replaced or removed either way.
- Keep the greeting example as a starting point, or start empty.

## 2. Conventions (full setup)

The template's rules encode one set of conventions. For each area, offer to keep it (the default), adjust it to the user's preference, or remove it.

| Area | Where it lives | What the template does |
| --- | --- | --- |
| Test layout | `tests/`, the Tests section of `.claude/CLAUDE.md`, `.claude/rules/testing.md` | Every test in one binary under `tests/`, using only the public API |
| Rust code | `.claude/rules/rust.md` | Typed error enum, fix code before suppressing a lint, ask before `unsafe` |
| Comments | `.claude/rules/comments.md` | No comment unless the code cannot say it |
| Changelog | `CHANGELOG.md`, `.claude/rules/changelog.md` | `### <version>` headings grouped into Added, Fixed, and Removed |
| Lints | `[lints]` in `Cargo.toml` | Pedantic clippy, `#[expect]` over `#[allow]`, no `todo!`, `unimplemented!`, or `dbg!`, `unsafe` denied |
| Dependencies | Conventions in `.claude/CLAUDE.md` | Ask before adding one |
| Minimum Rust version | `rust-version` in `Cargo.toml`, the `msrv` job in CI | 1.85, checked in CI |

Then ask whether Claude should always follow anything else in this project, such as a commit message style, preferred crates (`thiserror`, `anyhow`, `tokio`), or naming rules. Facts every session needs go in `.claude/CLAUDE.md`. Rules for one part of the tree go in a new file under `.claude/rules/` with a `paths:` filter.

Do not invent conventions the user did not ask for: a new project has no code yet to learn them from. Mention that rules work best when added later, once Claude makes the same mistake twice.

Check one thing yourself: compare the project rules with the user's personal instructions already in your context, such as their user `CLAUDE.md` or `~/.claude/rules/`. Where the two conflict, point it out and ask which should win, because Claude may follow either side of a conflict.

## 3. Template features (full setup)

Each is kept unless the user says otherwise.

- The review workflow: the `code-reviewer` agent, the hooks that keep Claude's turn open until a review passes the change and keep unreviewed work off `main`, `cargo land`, and the decisions for the user in `.claude/decisions/`.
- The `/release` skill.
- The hook that runs rustfmt on every Rust file Claude edits.
- `AGENTS.md`, which points other coding agents to these instructions.
- `SECURITY.md`, and the acknowledgement time it promises (7 days).
- The designed headers on `CONTRIBUTING.md` and `SECURITY.md`, or plain Markdown headings.
- The toolchain pinned in `rust-toolchain.toml`, or the latest stable Rust (check with `rustup check`).
- Dependabot, which opens pull requests for new Rust releases, dependency updates, and new GitHub Actions versions.

## 4. Apply

**Name.** It appears in `Cargo.toml`, in `use template_rs::` in `src/main.rs` and `tests/unit/`, in `CARGO_BIN_EXE_template_rs` in `tests/common/mod.rs` (the binary takes the package name), and in the titles of `.claude/CLAUDE.md` and `AGENTS.md`. Run `cargo check` afterwards so `Cargo.lock` follows.

**Package metadata.** Set `description`, `license`, and `repository` in `Cargo.toml`, deriving the repository URL from `git remote get-url origin`. Replace the template's `LICENSE` with the chosen license's text from its official source: `LICENSE-MIT` and `LICENSE-APACHE` for the dual license, otherwise `LICENSE`. With no license, delete `LICENSE` and the `license` field.

**Not published.** Set `publish = false` in `Cargo.toml` and remove its `include` list, then remove what only matters for crates.io: the `Package` step in `.github/workflows/ci.yml`, the packaged crate in the sentence about `cargo ci` in `.github/CONTRIBUTING.md` and in the Checks section of `.claude/CLAUDE.md`, and the `cargo publish` line in the last step of `.claude/skills/release/SKILL.md`. `cargo ci` skips its packaging check by itself once the crate isn't published. Keep the `cargo publish` deny rules, which cost nothing.

**Library or binary only.** For library only, delete `src/main.rs`, `tests/integration/`, `tests/common/`, and their `mod` lines in `tests/main.rs`. For binary only, move the logic into `src/main.rs`, delete `src/lib.rs`, `tests/unit/`, and its `mod` line, and remove the rule that tests see only the public API from `.claude/rules/testing.md` and the `pub` item check from `.claude/agents/code-reviewer.md`, because a binary has no public API. Either way, reword the error-handling line in `.claude/rules/rust.md`, which describes both the library and `main`, and update every description of the test layout that names a removed directory.

**Starting empty.** Remove `greet` and `GreetError` along with the tests that use them, leave `src/main.rs` as an empty `fn main() {}` if the binary stays, and delete helpers in `tests/common/` that become unused, because clippy runs with `-D warnings` and unused code fails it. Remove the placeholder line from `.claude/CLAUDE.md`.

**README.** Replace it with a short Markdown README: the project name as the title, the description, a getting started block with `cargo run` and `cargo test`, and a link to `.github/CONTRIBUTING.md`. Delete the template-only images `header.svg`, `body.svg`, `footer.svg`, and `use_template.svg` from `.github/assets/`, and the sentence about `body.svg` in step 2 of `.claude/skills/release/SKILL.md`.

**Changelog.** Replace the template's history with an empty `### Unreleased` section under the `# Changelog` title, in the format the user chose.

**Conventions.** A rule is only changed when everything that repeats it agrees. After adjusting or removing one, search for its file name and topic, since `AGENTS.md`, `.github/CONTRIBUTING.md`, and `.claude/agents/code-reviewer.md` mention several of them.

- Standard Rust test layout: move each file in `tests/unit/` into a `#[cfg(test)] mod tests` at the bottom of the source file it tests, move each integration test to `tests/<name>.rs` with `mod common;` at the top, and delete `tests/main.rs` and `tests/unit/`. Rewrite the Tests section of `.claude/CLAUDE.md`, `.claude/rules/testing.md`, the **Tests** bullet in `.github/CONTRIBUTING.md`, and the reviewer's line about tests under `tests/`.
- Another changelog format, such as Keep a Changelog: rewrite `.claude/rules/changelog.md`, `CHANGELOG.md`, the **Changelog** bullet in `.github/CONTRIBUTING.md`, and the changelog step of `.claude/skills/release/SKILL.md`.
- No changelog: delete `CHANGELOG.md` and `.claude/rules/changelog.md`, remove `/CHANGELOG.md` from `include` in `Cargo.toml` if it is still there, and remove the changelog line from `.claude/CLAUDE.md`, the changelog step of the release skill, the changelog check in the code reviewer, and the **Changelog** bullet in `.github/CONTRIBUTING.md`.
- Lints: edit `[lints]` in `Cargo.toml`. If `unsafe_code` is no longer denied, update the `unsafe` lines in `.claude/rules/rust.md`.
- Minimum Rust version: change `rust-version` in `Cargo.toml` and `RUSTUP_TOOLCHAIN` in the `msrv` job of `.github/workflows/ci.yml` together. If it matches the pinned toolchain, the `msrv` job checks nothing extra, so offer to remove it and the sentence about it in `.claude/CLAUDE.md`.

**Removing features.** A feature is only gone when nothing mentions it. After removing one, search the repository for its name and fix every remaining mention.

- The review workflow:
  - `.claude/agents/code-reviewer.md` and `.claude/decisions/`.
  - `review.rs`, `land.rs`, and `decisions.rs` in `.claude/hooks/src/`, with their `mod` lines, match arms, and usage entries in `main.rs`, and their tests in `.claude/hooks/tests/integration/` with their `mod` lines. Then delete whatever clippy reports as unused in `git.rs` and in `.claude/hooks/tests/common/`.
  - In `.claude/settings.json`: the `review` hooks under `SessionStart`, `PreToolUse`, `SubagentStart`, `SubagentStop`, and `Stop`, the `decisions` command under `SessionStart`, the `cargo land`, `git switch`, `git add`, and `git commit` allow rules, and the `cargo land --reviewed` deny rules.
  - The `land` alias in `.cargo/config.toml`, the Landing section of `.claude/CLAUDE.md` and the `cargo land` mention above it, `cargo land` in the `//!` headers of `.claude/hooks/src/main.rs` and `ci.rs`, the `//!` header of `git.rs`, rewritten for what is left, and the review mentions in `AGENTS.md` and `.github/CONTRIBUTING.md`, including the git version the latter asks for.
  - In `.claude/skills/release/SKILL.md`: the open decisions line and step, the branch step, and `cargo land` in its description, `allowed-tools`, and steps. Drop `git switch *` and `git ls-tree *` from its `allowed-tools`, have it run `cargo ci` before committing, and stop at the first failure. The release is then committed on the current branch, and the last step prints `git push --follow-tags` in place of `git push origin v$version`.
- `/release`: `.claude/skills/release/` and its line in `AGENTS.md`.
- The formatting hook: its `PostToolUse` entry in `.claude/settings.json`, `.claude/hooks/src/rustfmt.rs` with its `mod` line, match arm, and usage entry in `.claude/hooks/src/main.rs`, `.claude/hooks/tests/integration/rustfmt.rs` with its `mod` line, and the formatting hook mentions in `.claude/CLAUDE.md`, `AGENTS.md`, and `.github/CONTRIBUTING.md`. In `.claude/CLAUDE.md`, replace the hook line with an instruction to run `cargo fmt` after editing Rust files.
- `AGENTS.md`: the file, and the sentence about other assistants in `.github/CONTRIBUTING.md`.
- `SECURITY.md`: `.github/SECURITY.md` and `.github/assets/header_security.svg`.
- Designed headers: replace the image block at the top of `.github/CONTRIBUTING.md` with `# Contributing` and of `.github/SECURITY.md` with `# Security policy`, then delete `header_contributing.svg` and `header_security.svg`. If `.github/assets/` ends up empty, delete it.
- Latest stable toolchain: update `channel` in `rust-toolchain.toml` and the version in `.github/CONTRIBUTING.md` if it names one.
- Dependabot: `.github/dependabot.yml`.

## 5. Finish

1. Delete the files in `.claude/decisions/`, which are the template's own open questions, not the project's. Then delete the setup itself: `.claude/skills/setup-template/`, `.claude/hooks/src/setup_pending.rs` with its `mod` line, match arm, and usage entry in `.claude/hooks/src/main.rs`, `.claude/hooks/tests/integration/setup_pending.rs` with its `mod` line, the `setup-pending` command under `SessionStart` in `.claude/settings.json`, and the `setup-template` line in `AGENTS.md`.
2. In `.claude/settings.json`, delete any hook group left with an empty `hooks` list, then any event left with no groups, and the `hooks` block itself if it ends up empty. Keep `.claude/hooks/` either way, because it also holds `cargo ci`.
3. Run `cargo fmt`, then `cargo ci`. Fix anything that fails.
4. Search for `template_rs`, `template.rs`, and `ffuffix`, and resolve any leftovers.
5. Summarize what changed and what was removed. If the review workflow stays, land the setup as the Landing section of `.claude/CLAUDE.md` says, unless the user wants to look first. Otherwise suggest a commit message, and commit only if the user asks. If `SECURITY.md` stays, remind the user to turn on private vulnerability reporting in the repository's settings, because the **Report a vulnerability** button it points to only appears once that is on.
