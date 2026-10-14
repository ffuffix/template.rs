//! Running git, and the change that the review gate and `cargo land` both judge.
//!
//! A change is everything that differs from where the branch left the trunk, or origin's
//! newer copy of it: commits, uncommitted edits, and untracked files. Its review id is
//! git's verbatim patch id of that diff without context lines. Patch ids ignore line
//! numbers, so committing the change or rebasing it cleanly keeps the id, while editing
//! its lines, whitespace included, gives a new one. Moving an added line among unchanged
//! ones keeps the id, because the diff carries no context lines. The diff comes from a
//! tree written through a scratch copy of the index, which leaves the real index and the
//! working tree untouched.
//!
//! Files in `.claude/decisions/` count as a change but are left out of the review id: they
//! only record questions for the user, and adding one must not undo a review.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

pub(crate) const DECISIONS: &str = ".claude/decisions";
/// The first git with `patch-id --verbatim`.
const OLDEST_GIT: (u32, u32) = (2, 39);

pub(crate) struct Repo {
    root: PathBuf,
}

pub(crate) struct Change {
    /// The commit the change is measured from, or `None` before the first commit.
    pub(crate) base: Option<String>,
    pub(crate) tree: String,
    pub(crate) exists: bool,
    /// `None` when nothing outside `.claude/decisions/` changed.
    pub(crate) review_id: Option<String>,
}

/// Runs git in `directory` and returns its trimmed stdout, or `None` if it fails.
pub(crate) fn output(directory: &Path, arguments: &[&str]) -> Option<String> {
    run(git(directory, arguments), None).as_deref().map(text)
}

/// The installed git's version when it is older than the review id needs, so the gate and
/// `cargo land` can say why they can't work.
pub(crate) fn too_old() -> Option<String> {
    let mut command = Command::new("git");
    command.arg("--version");
    // The third word, because Apple's git adds more: `git version 2.37.1 (Apple Git-137.1)`.
    let version = text(&run(command, None)?)
        .split_whitespace()
        .nth(2)?
        .to_owned();
    let mut numbers = version.split('.').map(|number| number.parse::<u32>().ok());
    let found = (numbers.next()??, numbers.next()??);
    (found < OLDEST_GIT).then_some(version)
}

/// What the gate and `cargo land` say when git is older than `OLDEST_GIT`.
pub(crate) fn too_old_message(version: &str) -> String {
    let (major, minor) = OLDEST_GIT;
    format!(
        "The review gate and `cargo land` need git {major}.{minor} or newer, for \
        `git patch-id --verbatim`, and this machine has git {version}. Until git is updated, \
        no change can be reviewed or landed."
    )
}

fn git(directory: &Path, arguments: &[&str]) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(directory).args(arguments);
    command
}

fn run(mut command: Command, input: Option<&[u8]>) -> Option<Vec<u8>> {
    let stdin = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let mut child = command
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(input) = input {
        child.stdin.take()?.write_all(input).ok()?;
    }
    let output = child.wait_with_output().ok()?;
    output.status.success().then_some(output.stdout)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

impl Repo {
    pub(crate) fn open(directory: &Path) -> Option<Self> {
        output(directory, &["rev-parse", "--show-toplevel"]).map(|root| Self {
            root: PathBuf::from(root),
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn git(&self, arguments: &[&str]) -> Option<String> {
        output(&self.root, arguments)
    }

    /// A file in this worktree's git directory, where no commit or diff can see it.
    pub(crate) fn private_file(&self, name: &str) -> Option<PathBuf> {
        self.git(&["rev-parse", "--git-path", name])
            .map(|path| self.root.join(path))
    }

    pub(crate) fn branch(&self) -> Option<String> {
        // `--short` would answer `heads/main` when a tag is also named `main`.
        self.git(&["symbolic-ref", "--quiet", "HEAD"])?
            .strip_prefix("refs/heads/")
            .map(str::to_owned)
    }

    /// The branch changes land on: origin's default branch, else `main`, else `master`,
    /// whichever exists locally first.
    pub(crate) fn trunk(&self) -> Option<String> {
        let remote_default = self
            .git(&[
                "symbolic-ref",
                "--quiet",
                "--short",
                "refs/remotes/origin/HEAD",
            ])
            .and_then(|name| name.strip_prefix("origin/").map(str::to_owned));
        remote_default
            .into_iter()
            .chain(["main".to_owned(), "master".to_owned()])
            .find(|name| self.tip(name).is_some())
    }

    pub(crate) fn commit(&self, revision: &str) -> Option<String> {
        self.git(&[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{revision}^{{commit}}"),
        ])
    }

    /// The commit `branch` points at. Spelled out as `refs/heads/`, because git resolves a
    /// bare name to a tag of that name first.
    pub(crate) fn tip(&self, branch: &str) -> Option<String> {
        self.commit(&format!("refs/heads/{branch}"))
    }

    pub(crate) fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool {
        self.git(&["merge-base", "--is-ancestor", ancestor, descendant])
            .is_some()
    }

    /// The worktree that has `branch` checked out, if any.
    pub(crate) fn checkout_of(&self, branch: &str) -> Option<PathBuf> {
        let worktrees = self.git(&["worktree", "list", "--porcelain"])?;
        let wanted = format!("branch refs/heads/{branch}");
        worktrees
            .split("\n\n")
            .find(|entry| entry.lines().any(|line| line == wanted))?
            .lines()
            .find_map(|line| line.strip_prefix("worktree "))
            .map(PathBuf::from)
    }

    pub(crate) fn change(&self) -> Option<Change> {
        let tree = self.working_tree()?;
        let base = self.base();
        let base_tree = match &base {
            Some(base) => self.git(&["rev-parse", &format!("{base}^{{tree}}")])?,
            None => run(
                git(&self.root, &["hash-object", "-t", "tree", "--stdin"]),
                Some(b""),
            )
            .as_deref()
            .map(text)?,
        };
        let exists = !self
            .git(&["diff-tree", "--name-only", &base_tree, &tree])?
            .is_empty();
        let exclude = format!(":(exclude){DECISIONS}");
        let diff = run(
            git(
                &self.root,
                &[
                    "diff-tree",
                    "-p",
                    "--unified=0",
                    "--binary",
                    "--full-index",
                    &base_tree,
                    &tree,
                    "--",
                    ".",
                    &exclude,
                ],
            ),
            None,
        )?;
        let review_id = if diff.is_empty() {
            None
        } else {
            let id = run(git(&self.root, &["patch-id", "--verbatim"]), Some(&diff))
                .as_deref()
                .map(text)?;
            id.split_whitespace().next().map(str::to_owned)
        };
        Some(Change {
            base,
            tree,
            exists,
            review_id,
        })
    }

    fn base(&self) -> Option<String> {
        let head = self.commit("HEAD")?;
        let fork = self
            .trunk()
            .and_then(|trunk| self.trunk_tip(&trunk))
            .and_then(|tip| self.git(&["merge-base", "HEAD", &tip]));
        Some(fork.unwrap_or(head))
    }

    /// Origin's copy of the trunk when it is ahead of the local one, else the local one.
    /// A branch may start from either, and `cargo land` catches the trunk up first.
    fn trunk_tip(&self, trunk: &str) -> Option<String> {
        let ours = self.tip(trunk)?;
        match self.commit(&format!("refs/remotes/origin/{trunk}")) {
            Some(theirs) if self.is_ancestor(&ours, &theirs) => Some(theirs),
            _ => Some(ours),
        }
    }

    fn working_tree(&self) -> Option<String> {
        let index = self.private_file("index")?;
        // Per process, because hooks for one tool call can run at the same time.
        let scratch = self.private_file(&format!("claude-review-{}.index", process::id()))?;
        if index.is_file() {
            fs::copy(&index, &scratch).ok()?;
        }
        let tree = self
            .with_index(&scratch, &["add", "--all"])
            .and_then(|_| self.with_index(&scratch, &["write-tree"]));
        let _ = fs::remove_file(&scratch);
        tree
    }

    fn with_index(&self, index: &Path, arguments: &[&str]) -> Option<String> {
        let mut command = git(&self.root, arguments);
        command.env("GIT_INDEX_FILE", index);
        run(command, None).as_deref().map(text)
    }
}
