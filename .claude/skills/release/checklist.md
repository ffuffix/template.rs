# Release checklist

Run each command from the repository root. Every one must exit 0.

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo test --manifest-path .claude/hooks/Cargo.toml`
5. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
6. `cargo package`: builds the crate from its packaged sources, which catches files missing from `include` and path dependencies without a `version`.
