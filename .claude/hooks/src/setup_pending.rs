//! `SessionStart` hook: tell Claude when a project created from template.rs has not been set up.
//!
//! Prints nothing in the template.rs repository itself, or once the setup-template skill
//! has deleted itself, so it costs nothing outside a fresh project. Claude Code adds a
//! `SessionStart` hook's stdout to Claude's context.
//!
//! The template is recognized by its repository name alone, whoever owns it, because a
//! fork keeps the name and its owner wants to contribute back, not set up a new project.

use std::path::Path;
use std::process::ExitCode;

use crate::{git, input};

const TEMPLATE_NAME: &str = "template.rs";
const MESSAGE: &str = "This repository was created from the template.rs template and has not been set up yet. \
    Before other work, suggest running /setup-template, which asks how to personalize the project.";

pub(crate) fn run() -> ExitCode {
    let project = input::project_dir();
    let skill = project
        .join(".claude")
        .join("skills")
        .join("setup-template")
        .join("SKILL.md");
    if skill.is_file() && !is_template(&origin_url(&project)) {
        println!("{MESSAGE}");
    }
    ExitCode::SUCCESS
}

fn is_template(origin: &str) -> bool {
    let origin = origin.trim_end_matches('/');
    let origin = origin
        .strip_suffix(".git")
        .unwrap_or(origin)
        .trim_end_matches('/');
    origin
        .rsplit(['/', ':'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case(TEMPLATE_NAME))
}

fn origin_url(project: &Path) -> String {
    git::output(project, &["remote", "get-url", "origin"]).unwrap_or_default()
}
