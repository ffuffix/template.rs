use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::common::{TempDir, run_hook};

struct Project {
    root: TempDir,
    skill: PathBuf,
}

impl Project {
    fn new() -> Self {
        let root = TempDir::new();
        git(&root, &["init", "-q"]);
        let skill = root
            .path()
            .join(".claude")
            .join("skills")
            .join("setup-template")
            .join("SKILL.md");
        fs::create_dir_all(skill.parent().expect("the skill has a directory"))
            .expect("the skill directory can be created");
        fs::write(&skill, "# Set up\n").expect("the skill can be written");
        Self { root, skill }
    }

    fn set_origin(&self, url: &str) {
        git(&self.root, &["remote", "add", "origin", url]);
    }

    fn start_session(&self) -> Output {
        run_hook(
            "setup-pending",
            "{}",
            &[("CLAUDE_PROJECT_DIR", self.root.path().as_os_str())],
        )
    }
}

fn git(root: &TempDir, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .args(arguments)
        .status()
        .expect("git is installed");
    assert!(status.success(), "git {arguments:?} failed");
}

#[test]
fn suggests_setup_in_a_project_created_from_the_template() {
    let project = Project::new();
    project.set_origin("https://github.com/example/my-project.git");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("/setup-template"));
}

#[test]
fn suggests_setup_when_the_project_has_no_remote() {
    let project = Project::new();

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("/setup-template"));
}

#[test]
fn stays_silent_in_the_template_repository_itself() {
    let project = Project::new();
    project.set_origin("git@github.com:ffuffix/template.rs.git");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn stays_silent_in_a_fork_of_the_template() {
    let project = Project::new();
    project.set_origin("https://github.com/alice/template.rs.git");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn stays_silent_in_a_clone_of_a_local_copy_of_the_template() {
    let project = Project::new();
    project.set_origin("/home/alice/template.rs/.git");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn stays_silent_whatever_the_case_of_the_remote() {
    let project = Project::new();
    project.set_origin("https://github.com/FFuffix/Template.rs/");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn suggests_setup_when_the_name_only_starts_with_template_rs() {
    let project = Project::new();
    project.set_origin("https://github.com/ffuffix/template.rs-extras.git");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("/setup-template"));
}

#[test]
fn stays_silent_once_setup_has_removed_the_skill() {
    let project = Project::new();
    project.set_origin("https://github.com/example/my-project.git");
    fs::remove_file(&project.skill).expect("the skill can be removed");

    let output = project.start_session();

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}
