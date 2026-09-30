# Release checklist

Run each command from the repository root. Every one must exit 0.

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo test --manifest-path .claude/hooks/Cargo.toml`
5. `cargo doc --no-deps`: it must also finish without warnings, because CI treats them as errors.
6. `cargo package`: builds the crate from its packaged sources, which catches files missing from `include` and path dependencies without a `version`.
