# template.rs

This project's agent instructions are written in Claude Code's format, and they apply to any coding agent. The main instructions are in `.claude/CLAUDE.md`, imported below for agents that support `@` imports. If yours does not, open that file before starting.

@.claude/CLAUDE.md

The rest of `.claude/` holds more detail:

- `.claude/rules/`: rules for one area. Each file's `paths:` lists the files it covers: `rust.md` and `comments.md` for source files, `testing.md` for `tests/`, `changelog.md` for `CHANGELOG.md`.
- `.claude/agents/code-reviewer.md`: what a review of a finished change looks for.
- `.claude/skills/release/SKILL.md`: the release procedure, when you are asked to cut a release.
- `.claude/skills/setup-template/SKILL.md`: first-time setup for a project created from this template. If this file exists and `git remote get-url origin` does not point to `ffuffix/template.rs`, suggest it before other work.

Claude Code formats Rust files through a hook and is blocked from running `cargo publish`. Other agents run `cargo fmt` themselves after editing Rust files, and never run `cargo publish` or `git push` unless asked.
