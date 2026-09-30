//! `SessionStart` hook: tell Claude when a project created from template.rs has not been set up.
//!
//! Prints nothing in the template.rs repository itself, or once the setup-template skill
//! has deleted itself, so it costs nothing outside a fresh project. Claude Code adds a
//! `SessionStart` hook's stdout to Claude's context.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const TEMPLATE_REPOSITORY: &str = "ffuffix/template.rs";
const MESSAGE: &str = "This repository was created from the template.rs template and has not been set up yet. \
    Before other work, suggest running /setup-template, which asks how to personalize the project.";

pub(crate) fn run() -> ExitCode {
    let project =
        env::var_os("CLAUDE_PROJECT_DIR").map_or_else(|| PathBuf::from("."), PathBuf::from);
    let skill = project
        .join(".claude")
        .join("skills")
        .join("setup-template")
        .join("SKILL.md");
    if skill.is_file() && !origin_url(&project).contains(TEMPLATE_REPOSITORY) {
        println!("{MESSAGE}");
    }
    ExitCode::SUCCESS
}

fn origin_url(project: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["remote", "get-url", "origin"])
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}
