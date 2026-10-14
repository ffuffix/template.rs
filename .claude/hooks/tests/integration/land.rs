use std::fs;
use std::path::Path;
use std::process::Output;

use crate::common::{Repo, write};

const CLEAN: &str = "Verdict: 0 fix, 0 ask";

/// A repository whose `main` is pushed to a bare `origin` beside it.
fn with_remote() -> Repo {
    let repo = Repo::new();
    let remote = repo.beside("origin.git");
    repo.git(&[
        "init",
        "-q",
        "--bare",
        "-b",
        "main",
        &remote.to_string_lossy(),
    ]);
    repo.git(&["remote", "add", "origin", &remote.to_string_lossy()]);
    repo.git(&["push", "-q", "-u", "origin", "main"]);
    repo
}

/// Pushes a commit that adds `other.txt` to origin's `main` from a second clone.
fn push_from_elsewhere(repo: &Repo) {
    let other = repo.beside("other");
    let remote = repo.beside("origin.git");
    repo.git(&[
        "clone",
        "-q",
        &remote.to_string_lossy(),
        &other.to_string_lossy(),
    ]);
    write(&other, "other.txt", "elsewhere\n");
    repo.commit_all_in(&other, "Elsewhere");
    repo.git_in(&other, &["push", "-q", "origin", "main"]);
}

/// A reviewed, committed change on a `work` branch.
fn reviewed_change(repo: &Repo) {
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");
    repo.reviewer_says(CLEAN);
}

fn land(repo: &Repo) -> Output {
    land_in(repo, repo.path())
}

fn land_in(repo: &Repo, directory: &Path) -> Output {
    repo.hook_in(directory, "land", "")
}

fn refused(output: &Output) -> String {
    assert!(!output.status.success(), "{output:?}");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn landed(output: &Output) -> String {
    assert!(output.status.success(), "{output:?}");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn lands_a_reviewed_change_and_pushes_it() {
    let repo = with_remote();
    reviewed_change(&repo);

    let report = landed(&land(&repo));

    let work = repo.git(&["rev-parse", "work"]);
    assert_eq!(repo.git(&["rev-parse", "main"]), work);
    let remote = repo.beside("origin.git");
    assert_eq!(repo.git_in(&remote, &["rev-parse", "main"]), work);
    assert!(
        report.contains("Landed work on main: 1 commit(s)"),
        "{report}"
    );
    assert!(report.contains("Pushed main to origin."), "{report}");
}

#[test]
fn lands_without_a_remote() {
    let repo = Repo::new();
    reviewed_change(&repo);

    let report = landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"])
    );
    assert!(!report.contains("origin"), "{report}");
}

#[test]
fn refuses_a_change_without_a_clean_review() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");
    let main = repo.git(&["rev-parse", "main"]);

    assert!(refused(&land(&repo)).contains("hasn't passed review"));
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
}

#[test]
fn refuses_uncommitted_work() {
    let repo = Repo::new();
    reviewed_change(&repo);
    repo.write("notes.txt", "draft\n");

    assert!(refused(&land(&repo)).contains("Commit the change first"));
}

#[test]
fn refuses_to_land_from_the_trunk() {
    let repo = Repo::new();

    assert!(refused(&land(&repo)).contains("You're on main"));
}

#[test]
fn refuses_a_branch_with_nothing_to_land() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);

    assert!(refused(&land(&repo)).contains("Nothing to land"));
}

#[test]
fn fast_forwards_main_in_the_checkout_that_has_it() {
    let repo = Repo::new();
    let worktree = repo.beside("worktree");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "work",
        &worktree.to_string_lossy(),
    ]);
    write(&worktree, "src/new.rs", "pub fn new() {}\n");
    repo.commit_all_in(&worktree, "Add new");
    repo.reviewer_says_in(&worktree, CLEAN);

    landed(&land_in(&repo, &worktree));

    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        repo.git_in(&worktree, &["rev-parse", "HEAD"])
    );
    assert!(repo.path().join("src/new.rs").is_file());
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

#[test]
fn leaves_main_and_its_checkout_alone_when_it_cannot_fast_forward() {
    let repo = Repo::new();
    let worktree = repo.beside("worktree");
    repo.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "work",
        &worktree.to_string_lossy(),
    ]);
    write(&worktree, "README.md", "# Project, from work\n");
    repo.commit_all_in(&worktree, "Rename");
    repo.reviewer_says_in(&worktree, CLEAN);
    repo.write("README.md", "# Project, being edited\n");
    let main = repo.git(&["rev-parse", "main"]);

    assert!(refused(&land_in(&repo, &worktree)).contains("Couldn't fast-forward main"));
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
    assert_eq!(
        fs::read_to_string(repo.path().join("README.md")).expect("the edit is still there"),
        "# Project, being edited\n"
    );
}

#[test]
fn rebases_onto_a_trunk_that_moved() {
    let repo = Repo::new();
    reviewed_change(&repo);
    repo.git(&["switch", "-q", "main"]);
    repo.write("other.txt", "elsewhere\n");
    repo.commit_all("Elsewhere");
    repo.git(&["switch", "-q", "work"]);

    landed(&land(&repo));

    repo.git(&["cat-file", "-e", "main:src/new.rs"]);
    repo.git(&["cat-file", "-e", "main:other.txt"]);
    assert_eq!(repo.git(&["rev-list", "--merges", "main"]), "");
}

#[test]
fn catches_up_with_commits_pushed_from_elsewhere() {
    let repo = with_remote();
    push_from_elsewhere(&repo);
    reviewed_change(&repo);

    landed(&land(&repo));

    repo.git(&["cat-file", "-e", "main:other.txt"]);
    assert_eq!(
        repo.git_in(&repo.beside("origin.git"), &["rev-parse", "main"]),
        repo.git(&["rev-parse", "main"])
    );
}

#[test]
fn stops_without_moving_anything_when_the_rebase_conflicts() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write("README.md", "# Project, from work\n");
    repo.commit_all("Rename on work");
    repo.reviewer_says(CLEAN);
    repo.git(&["switch", "-q", "main"]);
    repo.write("README.md", "# Project, from main\n");
    repo.commit_all("Rename on main");
    repo.git(&["switch", "-q", "work"]);
    let (main, work) = (
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"]),
    );

    assert!(refused(&land(&repo)).contains("conflicts"));
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), work);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

#[test]
fn stops_without_moving_main_when_a_check_fails() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write(
        "Cargo.toml",
        "[package]\nname = \"unformatted\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    repo.write("src/main.rs", "fn main(){}\n");
    repo.commit_all("Add a crate rustfmt rejects");
    repo.reviewer_says(CLEAN);
    let main = repo.git(&["rev-parse", "main"]);

    let output = land(&repo);

    assert!(refused(&output).contains("A check failed"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("failed   cargo fmt --check"));
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
    assert!(fs::read_to_string(repo.path().join("src/main.rs")).is_ok());
}

#[test]
fn refuses_a_detached_head() {
    let repo = Repo::new();
    reviewed_change(&repo);
    repo.git(&["switch", "-q", "--detach"]);

    assert!(refused(&land(&repo)).contains("HEAD is detached"));
}

#[test]
fn refuses_when_main_and_origin_have_diverged() {
    let repo = with_remote();
    push_from_elsewhere(&repo);
    repo.write("local.txt", "here\n");
    repo.commit_all("Here");
    reviewed_change(&repo);
    let main = repo.git(&["rev-parse", "main"]);

    assert!(refused(&land(&repo)).contains("have diverged"));
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
}

#[test]
fn lands_locally_when_origin_is_out_of_reach() {
    let repo = with_remote();
    reviewed_change(&repo);
    fs::rename(repo.beside("origin.git"), repo.beside("moved.git"))
        .expect("the remote can be moved away");

    let report = landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"])
    );
    assert!(report.contains("Couldn't fetch origin"), "{report}");
    assert!(report.contains("Couldn't push main to origin"), "{report}");
}

#[test]
fn lands_on_master_when_there_is_no_main() {
    let repo = Repo::new();
    repo.git(&["branch", "-q", "-m", "main", "master"]);
    reviewed_change(&repo);

    landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "master"]),
        repo.git(&["rev-parse", "work"])
    );
}

#[test]
fn lands_on_the_default_branch_of_origin() {
    let repo = with_remote();
    repo.git(&["switch", "-q", "-c", "trunk"]);
    repo.git(&["push", "-q", "origin", "trunk"]);
    repo.git(&["remote", "set-head", "origin", "trunk"]);
    let main = repo.git(&["rev-parse", "main"]);
    reviewed_change(&repo);

    landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "trunk"]),
        repo.git(&["rev-parse", "work"])
    );
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
}

#[cfg(unix)]
#[test]
fn refuses_to_move_main_when_another_land_moved_it_first() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "elsewhere"]);
    repo.write("other.txt", "elsewhere\n");
    repo.commit_all("Elsewhere");
    let elsewhere = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["switch", "-q", "main"]);
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write("Cargo.toml", "[package]\nname = \"stand-in\"\n");
    repo.commit_all("Add a crate");
    repo.reviewer_says(CLEAN);
    // A stand-in for cargo whose formatting check lands another change on main, the way a
    // second session could while this land runs its checks.
    let mut environment = repo.stand_in(
        "cargo",
        "#!/bin/sh\n\
        [ \"$1\" = metadata ] && { echo '{\"packages\":[]}'; exit 0; }\n\
        [ \"$1\" = fmt ] && git update-ref refs/heads/main \"$LANDED_ELSEWHERE\"\n\
        exit 0\n",
    );
    environment.push(("LANDED_ELSEWHERE", elsewhere.clone().into()));

    let output = repo.hook_with(repo.path(), "land", "", &environment);

    assert!(refused(&output).contains("main moved while landing"));
    assert_eq!(repo.git(&["rev-parse", "main"]), elsewhere);
}

#[test]
fn lands_a_branch_started_from_a_newer_origin() {
    let repo = with_remote();
    push_from_elsewhere(&repo);
    repo.git(&["fetch", "-q", "origin"]);
    repo.git(&["switch", "-q", "-c", "work", "origin/main"]);
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");
    repo.reviewer_says(CLEAN);

    landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"])
    );
}

#[test]
fn lands_decisions_without_a_review() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write(".claude/decisions/cache.md", "# Keep the cache?\n");
    repo.commit_all("Ask about the cache");

    landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"])
    );
}

#[test]
fn lands_on_the_branch_when_a_tag_has_the_same_name() {
    let repo = with_remote();
    repo.git(&["tag", "main"]);
    repo.write("second.txt", "after the tag\n");
    repo.commit_all("Move past the tag");
    repo.git(&["push", "-q", "origin", "refs/heads/main"]);
    reviewed_change(&repo);

    let report = landed(&land(&repo));

    assert_eq!(
        repo.git(&["rev-parse", "refs/heads/main"]),
        repo.git(&["rev-parse", "work"])
    );
    assert!(report.contains("Pushed main to origin."), "{report}");
}

#[test]
fn lands_a_change_reviewed_by_another_agent_when_told_so() {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");

    assert!(refused(&land(&repo)).contains("hasn't passed review"));
    landed(&repo.hook_in(repo.path(), "land --reviewed", ""));

    assert_eq!(
        repo.git(&["rev-parse", "main"]),
        repo.git(&["rev-parse", "work"])
    );
}

#[test]
fn catches_up_with_origin_when_origin_has_a_tag_named_like_the_trunk() {
    let repo = with_remote();
    push_from_elsewhere(&repo);
    let other = repo.beside("other");
    repo.git_in(&other, &["tag", "main", "HEAD~1"]);
    repo.git_in(&other, &["push", "-q", "origin", "refs/tags/main"]);
    reviewed_change(&repo);

    landed(&land(&repo));

    repo.git(&["cat-file", "-e", "refs/heads/main:other.txt"]);
}

#[cfg(unix)]
#[test]
fn says_when_git_is_too_old() {
    let repo = Repo::new();
    reviewed_change(&repo);
    let old_git = repo.old_git("git version 2.30.2");

    let output = repo.hook_with(repo.path(), "land", "", &old_git);

    assert!(refused(&output).contains("need git 2.39 or newer"));
}

#[test]
fn refuses_an_argument_it_does_not_know() {
    let repo = Repo::new();
    reviewed_change(&repo);
    let main = repo.git(&["rev-parse", "main"]);

    assert!(
        refused(&repo.hook_in(repo.path(), "land --dry-run", "")).contains("usage: cargo land")
    );
    assert_eq!(repo.git(&["rev-parse", "main"]), main);
}
