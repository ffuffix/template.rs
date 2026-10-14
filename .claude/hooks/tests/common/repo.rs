use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

use super::{TempDir, run_hook_in};

/// A git repository on `main` with one commit, in a scratch directory with room for a
/// remote and worktrees beside it. Git reads none of the user's configuration here, so
/// commit signing or hooks set up on the machine can't change what a test sees.
pub(crate) struct Repo {
    space: TempDir,
    root: PathBuf,
}

impl Repo {
    pub(crate) fn new() -> Self {
        let space = TempDir::new();
        let root = space.path().join("repo");
        fs::create_dir_all(&root).expect("the repository directory can be created");
        let repo = Self { space, root };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.write("README.md", "# Project\n");
        repo.commit_all("Start");
        repo
    }

    pub(crate) fn path(&self) -> &Path {
        &self.root
    }

    /// A path beside the repository, for a remote or a worktree.
    pub(crate) fn beside(&self, name: &str) -> PathBuf {
        self.space.path().join(name)
    }

    pub(crate) fn git(&self, arguments: &[&str]) -> String {
        self.git_in(&self.root, arguments)
    }

    pub(crate) fn git_in(&self, directory: &Path, arguments: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .envs(self.environment())
            .output()
            .expect("git is installed");
        assert!(output.status.success(), "git {arguments:?}: {output:?}");
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    pub(crate) fn write(&self, file: &str, contents: &str) {
        write(&self.root, file, contents);
    }

    pub(crate) fn commit_all(&self, message: &str) {
        self.commit_all_in(&self.root, message);
    }

    pub(crate) fn commit_all_in(&self, directory: &Path, message: &str) {
        self.git_in(directory, &["add", "--all"]);
        self.git_in(directory, &["commit", "-q", "-m", message]);
    }

    /// Runs `hook` from `directory` with `input` on stdin.
    pub(crate) fn hook_in(&self, directory: &Path, hook: &str, input: &str) -> Output {
        self.hook_with(directory, hook, input, &[])
    }

    /// Runs `hook` like `hook_in`, with `extra` added to its environment.
    pub(crate) fn hook_with(
        &self,
        directory: &Path,
        hook: &str,
        input: &str,
        extra: &[(&str, OsString)],
    ) -> Output {
        let own = self.environment();
        let environment: Vec<(&str, &OsStr)> = own
            .iter()
            .chain(extra)
            .map(|(name, value)| (*name, value.as_os_str()))
            .collect();
        run_hook_in(directory, hook, input, &environment)
    }

    /// Runs the `review` hook for `event` in `directory`, and returns what it printed.
    pub(crate) fn review_event_in(&self, directory: &Path, mut event: Value) -> Option<Value> {
        event["cwd"] = json!(directory);
        let output = self.hook_in(directory, "review", &event.to_string());
        assert!(output.status.success(), "{output:?}");
        let printed = String::from_utf8_lossy(&output.stdout);
        (!printed.trim().is_empty())
            .then(|| serde_json::from_str(&printed).expect("the hook prints JSON"))
    }

    pub(crate) fn review_event(&self, event: Value) -> Option<Value> {
        self.review_event_in(&self.root, event)
    }

    /// The reviewer finishing in `directory` with `report` as its last message.
    pub(crate) fn reviewer_says_in(&self, directory: &Path, report: &str) {
        self.review_event_in(
            directory,
            json!({
                "hook_event_name": "SubagentStop",
                "agent_type": "code-reviewer",
                "stop_hook_active": false,
                "last_assistant_message": report,
            }),
        );
    }

    pub(crate) fn reviewer_says(&self, report: &str) {
        self.reviewer_says_in(&self.root, report);
    }

    /// Installs `script` as a stand-in for the command `name`, and returns the environment
    /// that puts it first on the `PATH`.
    #[cfg(unix)]
    pub(crate) fn stand_in(&self, name: &str, script: &str) -> Vec<(&'static str, OsString)> {
        use std::os::unix::fs::PermissionsExt;

        let bin = self.beside(&format!("stand-in-{name}"));
        write(&bin, name, script);
        fs::set_permissions(bin.join(name), fs::Permissions::from_mode(0o755))
            .expect("the stand-in can be made executable");
        let path = std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )))
        .expect("the PATH can be joined");
        vec![("PATH", path)]
    }

    /// Environment for a hook whose git answers `git --version` with `version_line`, while
    /// the real git does everything else.
    #[cfg(unix)]
    pub(crate) fn old_git(&self, version_line: &str) -> Vec<(&'static str, OsString)> {
        let real = Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .expect("sh runs");
        let real = String::from_utf8_lossy(&real.stdout).trim().to_owned();
        let mut environment = self.stand_in(
            "git",
            &format!(
                "#!/bin/sh\n[ \"$1\" = --version ] && {{ echo '{version_line}'; exit 0; }}\nexec \"$REAL_GIT\" \"$@\"\n"
            ),
        );
        environment.push(("REAL_GIT", real.into()));
        environment
    }

    fn environment(&self) -> Vec<(&'static str, OsString)> {
        vec![
            (
                "GIT_CONFIG_GLOBAL",
                self.beside("gitconfig").into_os_string(),
            ),
            ("GIT_CONFIG_NOSYSTEM", "1".into()),
            ("GIT_AUTHOR_NAME", "Test".into()),
            ("GIT_AUTHOR_EMAIL", "test@example.com".into()),
            ("GIT_COMMITTER_NAME", "Test".into()),
            ("GIT_COMMITTER_EMAIL", "test@example.com".into()),
        ]
    }
}

pub(crate) fn write(directory: &Path, file: &str, contents: &str) {
    let path = directory.join(file);
    fs::create_dir_all(path.parent().expect("files have a directory"))
        .expect("the file's directory can be created");
    fs::write(path, contents).expect("the file can be written");
}
