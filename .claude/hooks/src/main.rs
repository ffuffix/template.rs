//! Claude Code hooks for this repository, one per subcommand. `.claude/settings.json`
//! runs them through `cargo run`, so they need nothing beyond the Rust toolchain.

mod rustfmt;
mod setup_pending;

use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;

fn main() -> ExitCode {
    match env::args_os().nth(1).as_deref().and_then(OsStr::to_str) {
        Some("rustfmt") => rustfmt::run(),
        Some("setup-pending") => setup_pending::run(),
        _ => {
            eprintln!("usage: claude-hooks <rustfmt | setup-pending>");
            ExitCode::FAILURE
        }
    }
}
