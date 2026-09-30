---
paths:
  - "tests/**"
  - ".claude/hooks/tests/**"
---

# Tests

All Rust tests compile into one test binary rooted at `tests/main.rs`. Cargo builds every top-level `.rs` file in `tests/` as a separate binary, so never add one. Add a module instead.

| Directory | Holds |
| --- | --- |
| `tests/unit/` | One file per library module, calling its public functions directly |
| `tests/integration/` | Tests that run the compiled binary through `common::run_binary` |
| `tests/common/` | Helpers shared by both. No `#[test]` functions |

The Claude Code hooks in `.claude/hooks/` are a separate package with the same layout: its tests in `.claude/hooks/tests/integration/` run the hooks, with helpers in `.claude/hooks/tests/common/`. Run them with `cargo test --manifest-path .claude/hooks/Cargo.toml`.

- A new test file must be declared in its directory's `mod.rs`. An undeclared file is silently not compiled.
- Tests see only the public API. Do not make an item `pub` to reach it from a test. Test it through the public function that uses it.
