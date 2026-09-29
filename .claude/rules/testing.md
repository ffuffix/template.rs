---
paths:
  - "tests/**"
---

# Tests

All Rust tests compile into one test binary rooted at `tests/main.rs`. Cargo builds every top-level `.rs` file in `tests/` as a separate binary, so never add one. Add a module instead.

| Directory | Holds |
| --- | --- |
| `tests/unit/` | One file per library module, calling its public functions directly |
| `tests/integration/` | Tests that run the compiled binary through `common::run_binary` |
| `tests/common/` | Helpers shared by both. No `#[test]` functions |
| `tests/hooks/` | Python tests for `.claude/hooks/`, run with `python3 -m unittest discover -s tests/hooks` |

- A new test file must be declared in its directory's `mod.rs`. An undeclared file is silently not compiled.
- Tests see only the public API. Do not make an item `pub` to reach it from a test. Test it through the public function that uses it.
