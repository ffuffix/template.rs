---
paths:
  - "**/*.rs"
---

# Rust

- The library returns a typed error enum. `main` turns an error into a message on stderr and a non-zero exit code.
- Fix the code before reaching for a lint suppression.
- `unsafe_code` is denied. Ask before overriding it.
- An `unsafe` block's `// SAFETY:` comment names the invariant that makes it sound, not what the block does.
