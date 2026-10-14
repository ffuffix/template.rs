//! Claude Code hooks for this repository, and the `cargo ci` and `cargo land` commands, one
//! per subcommand. `.claude/settings.json` and the aliases in `.cargo/config.toml` run them
//! through `cargo run`, so they need nothing beyond the Rust toolchain.

mod cargo;
mod ci;
mod decisions;
mod git;
mod input;
mod land;
mod review;
mod rustfmt;
mod setup_pending;

use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;

fn main() -> ExitCode {
    match env::args_os().nth(1).as_deref().and_then(OsStr::to_str) {
        Some("ci") => ci::run(),
        Some("decisions") => decisions::run(),
        Some("land") => land::run(),
        Some("review") => review::run(),
        Some("rustfmt") => rustfmt::run(),
        Some("setup-pending") => setup_pending::run(),
        _ => {
            eprintln!(
                "usage: claude-hooks <ci [--list] | decisions | land [--reviewed] | review | rustfmt | setup-pending>"
            );
            ExitCode::FAILURE
        }
    }
}
