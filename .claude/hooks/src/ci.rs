//! `cargo ci`: every check the CI workflow runs, run locally, for the crate at the
//! repository root and for this hooks package. `cargo land` runs it before moving the
//! trunk, and `cargo ci --list` prints the commands without running them.
//!
//! The list mirrors `.github/workflows/ci.yml`, and a test fails when the workflow runs a
//! cargo command that is missing here. The packaged-crate check runs only for a crate that
//! can be published. The minimum Rust version check runs only when that toolchain is
//! already installed, because installing one is a large download the user should choose.

use std::env;
use std::fmt;
use std::path::Path;
use std::process::{Command, ExitCode, Output, Stdio};

use crate::cargo;
use crate::git::Repo;

const HOOKS_MANIFEST: &str = ".claude/hooks/Cargo.toml";
const SHOWN_LINES: usize = 40;

struct Check {
    arguments: Vec<&'static str>,
    environment: Vec<(&'static str, String)>,
    toolchain: Option<String>,
}

impl Check {
    fn new(arguments: &[&'static str]) -> Self {
        Self {
            arguments: arguments.to_vec(),
            environment: Vec::new(),
            toolchain: None,
        }
    }

    fn with_env(mut self, name: &'static str, value: &str) -> Self {
        self.environment.push((name, value.to_owned()));
        self
    }

    fn on_toolchain(mut self, toolchain: &str) -> Self {
        self.toolchain = Some(toolchain.to_owned());
        self.with_env("RUSTUP_TOOLCHAIN", toolchain)
    }
}

impl fmt::Display for Check {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.environment {
            write!(formatter, "{name}={value:?} ")?;
        }
        write!(formatter, "cargo {}", self.arguments.join(" "))
    }
}

pub(crate) fn run() -> ExitCode {
    let directory = env::current_dir().unwrap_or_else(|_| ".".into());
    let root = Repo::open(&directory).map_or(directory, |repo| repo.root().to_owned());
    if env::args_os().nth(2).is_some_and(|flag| flag == "--list") {
        for check in checks(&root) {
            println!("{check}");
        }
        return ExitCode::SUCCESS;
    }
    if all_pass(&root) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs every check in `root`, printing one line for each, and stops at the first failure
/// with the end of its output.
pub(crate) fn all_pass(root: &Path) -> bool {
    for check in checks(root) {
        if let Some(toolchain) = &check.toolchain
            && !installed(toolchain)
        {
            println!(
                "skipped  {check}: Rust {toolchain} isn't installed. \
                `rustup toolchain install {toolchain} --profile minimal` adds it."
            );
            continue;
        }
        let mut command = Command::new("cargo");
        command
            .args(&check.arguments)
            .current_dir(root)
            .stdin(Stdio::null());
        command.envs(check.environment.iter().map(|(name, value)| (name, value)));
        match command.output() {
            Ok(output) if output.status.success() => println!("passed   {check}"),
            Ok(output) => {
                println!("failed   {check}");
                show_end(&output);
                return false;
            }
            Err(error) => {
                println!("failed   {check}: {error}");
                return false;
            }
        }
    }
    true
}

fn checks(root: &Path) -> Vec<Check> {
    let mut checks = Vec::new();
    let manifest = root.join("Cargo.toml");
    if manifest.is_file() {
        checks.extend([
            Check::new(&["fmt", "--check"]),
            Check::new(&[
                "clippy",
                "--locked",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]),
            Check::new(&["test", "--locked"]),
            Check::new(&["doc", "--locked", "--no-deps"]).with_env("RUSTDOCFLAGS", "-D warnings"),
        ]);
        let package = cargo::package(&manifest);
        // `publish` is null when any registry may publish it, and `[]` for `publish = false`.
        let publishable = package.as_ref().is_some_and(|package| {
            package["publish"]
                .as_array()
                .is_none_or(|registries| !registries.is_empty())
        });
        if publishable {
            checks.push(Check::new(&["package", "--locked", "--allow-dirty"]));
        }
        if let Some(version) = package
            .as_ref()
            .and_then(|package| package["rust_version"].as_str())
        {
            checks.push(Check::new(&["check", "--locked", "--all-targets"]).on_toolchain(version));
        }
    }
    if root.join(HOOKS_MANIFEST).is_file() {
        checks.extend([
            Check::new(&["fmt", "--check", "--manifest-path", HOOKS_MANIFEST]),
            Check::new(&[
                "clippy",
                "--locked",
                "--all-targets",
                "--manifest-path",
                HOOKS_MANIFEST,
                "--",
                "-D",
                "warnings",
            ]),
            Check::new(&["test", "--locked", "--manifest-path", HOOKS_MANIFEST]),
        ]);
    }
    checks
}

fn installed(toolchain: &str) -> bool {
    Command::new("rustup")
        .args(["run", toolchain, "rustc", "--version"])
        .env("RUSTUP_AUTO_INSTALL", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn show_end(output: &Output) {
    for stream in [&output.stderr, &output.stdout] {
        let text = String::from_utf8_lossy(stream);
        let lines: Vec<&str> = text.trim_end().lines().collect();
        for line in &lines[lines.len().saturating_sub(SHOWN_LINES)..] {
            println!("  {line}");
        }
    }
}
