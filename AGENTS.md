# template.rs

This project's agent instructions are written in Claude Code's format, and they apply to any coding agent. The main instructions are in `.claude/CLAUDE.md`, imported below for agents that support `@` imports. If yours does not, open that file before starting.

@.claude/CLAUDE.md

The rest of `.claude/` holds more detail:

- `.claude/rules/`: rules for one area. Each file's `paths:` lists the files it covers: `rust.md` and `comments.md` for source files, `testing.md` for `tests/` and `.claude/hooks/tests/`, `changelog.md` for `CHANGELOG.md`.
- `.claude/agents/code-reviewer.md`: what a review of a finished change looks for, and the report it ends with.
- `.claude/decisions/`: questions a review raised, each waiting for the user to decide.
- `.claude/skills/release/SKILL.md`: the release procedure, when you are asked to cut a release. Agents other than Claude Code land its release commit with `cargo land --reviewed`.
- `.claude/skills/setup-template/SKILL.md`: first-time setup for a project created from this template. If this file exists and the repository that `git remote get-url origin` points to isn't named `template.rs` (the template or a fork of it), suggest it before other work.

Claude Code formats Rust files and enforces the review in the Landing section of `.claude/CLAUDE.md` through hooks, and its permission rules deny `cargo publish`. Other agents run `cargo fmt` themselves after editing Rust files. They review a finished change by following `.claude/agents/code-reviewer.md`, fix its Fix findings and review again until it comes back clean, and record its Ask findings as decisions. Then they commit on a branch and land with `cargo land --reviewed`, because only Claude Code's hooks record a review that plain `cargo land` accepts. They never run `cargo publish` or `git push`.
