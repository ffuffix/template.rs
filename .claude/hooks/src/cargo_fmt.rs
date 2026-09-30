//! `PostToolUse` hook: run `cargo fmt` on the package that owns an edited Rust file.
//!
//! Resolves the package from the edited file's path rather than `CLAUDE_PROJECT_DIR`,
//! because `CLAUDE_PROJECT_DIR` keeps pointing at the main checkout after Claude
//! enters a worktree. Exit 2 feeds rustfmt errors back to Claude.
//!
//! A `mod` declaration whose file does not exist yet is expected while Claude
//! writes a module tree one file at a time, so that error is not reported.

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;

const MAX_ERROR_LINES: usize = 20;
const CARGO_TIMEOUT: Duration = Duration::from_secs(50);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const UNWRITTEN_MODULE_ERROR: &str = "failed to resolve mod";

pub(crate) fn run() -> ExitCode {
    let Some(file) = edited_rust_file() else {
        return ExitCode::SUCCESS;
    };
    let Some(manifest) = file.parent().and_then(find_manifest) else {
        return ExitCode::SUCCESS;
    };
    let Some(output) = cargo_fmt(&manifest) else {
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
    eprintln!("cargo fmt failed for {}:", name.to_string_lossy());
    for line in errors.trim().lines().take(MAX_ERROR_LINES) {
        eprintln!("  {line}");
    }
    ExitCode::from(2)
}

fn edited_rust_file() -> Option<PathBuf> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).ok()?;
    let payload: Value = serde_json::from_str(&input).ok()?;

    let file = payload["tool_input"]["file_path"]
        .as_str()
        .filter(|file| !file.is_empty())?;
    let cwd = payload["cwd"]
        .as_str()
        .filter(|cwd| !cwd.is_empty())
        .unwrap_or(".");
    let path = Path::new(cwd).join(file);
    (path.extension()? == "rs").then_some(path)
}

fn find_manifest(directory: &Path) -> Option<PathBuf> {
    directory
        .ancestors()
        .map(|ancestor| ancestor.join("Cargo.toml"))
        .find(|manifest| manifest.is_file())
}

fn cargo_fmt(manifest: &Path) -> Option<Output> {
    let mut child = Command::new("cargo")
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let stdout = read_in_background(child.stdout.take()?);
    let stderr = read_in_background(child.stderr.take()?);

    let deadline = Instant::now() + CARGO_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().ok()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            return None;
        }
        thread::sleep(POLL_INTERVAL);
    };

    Some(Output {
        status,
        stdout: stdout.join().ok()?,
        stderr: stderr.join().ok()?,
    })
}

fn read_in_background(mut pipe: impl Read + Send + 'static) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        bytes
    })
}
