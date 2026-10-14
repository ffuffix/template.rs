//! `PostToolUse` hook: run rustfmt on an edited Rust file, with its package's edition.
//!
//! Formats only the edited file, not the whole package as `cargo fmt` would, so each
//! edit costs the same however large the project grows, and files Claude didn't touch
//! stay as they are. Exit 2 feeds rustfmt errors back to Claude.
//!
//! Resolves the package from the edited file's path rather than `CLAUDE_PROJECT_DIR`,
//! because `CLAUDE_PROJECT_DIR` keeps pointing at the main checkout after Claude
//! enters a worktree.
//!
//! A `mod` declaration whose file does not exist yet is expected while Claude
//! writes a module tree one file at a time, so that error is not reported.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output, Stdio};

use crate::{cargo, input};

const MAX_ERROR_LINES: usize = 20;
const UNWRITTEN_MODULE_ERROR: &str = "failed to resolve mod";

pub(crate) fn run() -> ExitCode {
    let Some(file) = edited_rust_file() else {
        return ExitCode::SUCCESS;
    };
    let Some(edition) = file
        .parent()
        .and_then(find_manifest)
        .and_then(|manifest| cargo::package(&manifest))
        .and_then(|package| package["edition"].as_str().map(str::to_owned))
    else {
        return ExitCode::SUCCESS;
    };
    let Some(output) = rustfmt(&file, &edition) else {
        return ExitCode::SUCCESS;
    };

    let errors = if output.stderr.is_empty() {
        &output.stdout
    } else {
        &output.stderr
    };
    let errors = String::from_utf8_lossy(errors);
    if output.status.success() || errors.contains(UNWRITTEN_MODULE_ERROR) {
        return ExitCode::SUCCESS;
    }

    let name = file.file_name().unwrap_or(file.as_os_str());
    eprintln!("rustfmt failed for {}:", name.to_string_lossy());
    for line in errors.trim().lines().take(MAX_ERROR_LINES) {
        eprintln!("  {line}");
    }
    ExitCode::from(2)
}

fn edited_rust_file() -> Option<PathBuf> {
    let payload = input::read()?;
    let file = payload["tool_input"]["file_path"]
        .as_str()
        .filter(|file| !file.is_empty())?;
    let path = input::cwd(&payload).join(file);
    (path.extension()? == "rs").then_some(path)
}

fn find_manifest(directory: &Path) -> Option<PathBuf> {
    directory
        .ancestors()
        .map(|ancestor| ancestor.join("Cargo.toml"))
        .find(|manifest| manifest.is_file())
}

fn rustfmt(file: &Path, edition: &str) -> Option<Output> {
    Command::new("rustfmt")
        .args(["--edition", edition])
        .arg(file)
        .stdin(Stdio::null())
        .output()
        .ok()
}
