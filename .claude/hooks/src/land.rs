//! `cargo land`: put a reviewed change on the trunk. It refuses work that is uncommitted,
//! on the trunk itself, or without a clean review recorded by the review gate.
//! `cargo land --reviewed` skips that last check, for coding agents other than Claude Code,
//! which review by following `.claude/agents/code-reviewer.md` without the gate's hooks. It then catches the trunk up with
//! `origin` if there is one, rebases the branch onto it, runs `cargo ci`, moves the trunk
//! to the branch, and pushes it.
//!
//! The trunk is often checked out in another worktree: the main checkout, while Claude
//! works in a worktree of its own. Moving the ref there would leave that checkout's files
//! behind, so that checkout fast-forwards itself instead.

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use crate::git::{self, Repo};
use crate::{ci, review};

pub(crate) fn run() -> ExitCode {
    match land() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}

fn land() -> Result<String, String> {
    if let Some(version) = git::too_old() {
        return Err(git::too_old_message(&version));
    }
    let arguments: Vec<OsString> = env::args_os().skip(2).collect();
    let reviewed_elsewhere = match arguments.as_slice() {
        [] => false,
        [flag] if flag == "--reviewed" => true,
        _ => {
            return Err(
                "usage: cargo land [--reviewed]\n`--reviewed` is for coding agents \
                other than Claude Code, after a clean review by \
                `.claude/agents/code-reviewer.md`."
                    .to_owned(),
            );
        }
    };
    let directory = env::current_dir().map_err(|error| error.to_string())?;
    let repo = Repo::open(&directory).ok_or("Not in a git repository.")?;
    let trunk = repo.trunk().ok_or(
        "Found no trunk to land on: neither origin's default branch, main, nor master exists \
        locally.",
    )?;
    let branch = repo
        .branch()
        .ok_or("HEAD is detached. Switch to the branch that holds the change.")?;
    if branch == trunk {
        return Err(format!(
            "You're on {trunk}. Move the work to a branch with `git switch -c <name>`, then \
            land that."
        ));
    }
    if !repo
        .git(&["status", "--porcelain"])
        .unwrap_or_default()
        .is_empty()
    {
        return Err("Commit the change first: `git status` shows uncommitted work.".to_owned());
    }
    check_reviewed(&repo, &branch, &trunk, reviewed_elsewhere)?;

    let remote = repo
        .git(&["remote"])
        .is_some_and(|remotes| remotes.lines().any(|remote| remote == "origin"));
    let mut notes = Vec::new();
    if remote {
        catch_up(&repo, &trunk, &mut notes)?;
    }
    // Everything from here works against this commit, so a land by another session in the
    // meantime makes moving the trunk fail rather than drop that session's commits.
    let old = repo.tip(&trunk).ok_or("Couldn't read the trunk.")?;
    rebase(&repo, &trunk, &old)?;
    check_reviewed(&repo, &branch, &trunk, reviewed_elsewhere)?;
    if !ci::all_pass(repo.root()) {
        return Err(format!(
            "A check failed, so {trunk} didn't move. Fix it and land again; a fix that changes \
            the diff needs another review."
        ));
    }

    let head = repo.commit("HEAD").ok_or("Couldn't read HEAD.")?;
    move_trunk(&repo, &trunk, &old, &head)?;
    if remote {
        let refspec = format!("refs/heads/{trunk}:refs/heads/{trunk}");
        notes.push(match repo.git(&["push", "--quiet", "origin", &refspec]) {
            Some(_) => format!("Pushed {trunk} to origin."),
            None => format!(
                "Couldn't push {trunk} to origin, but it has landed locally. Run \
                `git push origin refs/heads/{trunk}` once you can."
            ),
        });
    }
    let commits = repo
        .git(&["rev-list", "--count", &format!("{old}..{head}")])
        .unwrap_or_default();
    let short = head.get(..12).unwrap_or(&head);
    notes.insert(
        0,
        format!("Landed {branch} on {trunk}: {commits} commit(s), now at {short}."),
    );
    Ok(notes.join("\n"))
}

fn check_reviewed(
    repo: &Repo,
    branch: &str,
    trunk: &str,
    reviewed_elsewhere: bool,
) -> Result<(), String> {
    let change = repo.change().ok_or("Couldn't read the change from git.")?;
    if !change.exists {
        return Err(format!(
            "Nothing to land: {branch} has no changes from {trunk}."
        ));
    }
    match change.review_id {
        Some(id) if !reviewed_elsewhere && !review::passed(repo, &id) => Err(
            "This change hasn't passed review. Run the `code-reviewer` agent, fix its Fix \
            findings, and land once a review comes back clean."
                .to_owned(),
        ),
        _ => Ok(()),
    }
}

/// Fast-forwards the trunk to `origin`'s when it is behind, so the push that follows
/// won't be rejected.
fn catch_up(repo: &Repo, trunk: &str, notes: &mut Vec<String>) -> Result<(), String> {
    // The full ref, because a bare name would fetch a tag of that name first.
    let branch = format!("refs/heads/{trunk}");
    if repo.git(&["fetch", "--quiet", "origin", &branch]).is_none() {
        notes.push(format!(
            "Couldn't fetch origin, so {trunk} may be missing commits pushed from elsewhere."
        ));
        return Ok(());
    }
    let (Some(theirs), Some(ours)) = (
        repo.commit(&format!("refs/remotes/origin/{trunk}")),
        repo.tip(trunk),
    ) else {
        return Ok(());
    };
    if repo.is_ancestor(&theirs, &ours) {
        Ok(())
    } else if repo.is_ancestor(&ours, &theirs) {
        move_trunk(repo, trunk, &ours, &theirs)
    } else {
        Err(format!(
            "{trunk} and origin/{trunk} have diverged. Reconcile them, then land again."
        ))
    }
}

fn rebase(repo: &Repo, trunk: &str, onto: &str) -> Result<(), String> {
    if repo.is_ancestor(onto, "HEAD") || repo.git(&["rebase", "--quiet", onto]).is_some() {
        return Ok(());
    }
    repo.git(&["rebase", "--abort"]);
    Err(format!(
        "Rebasing onto {trunk} conflicts, so nothing moved. Run `git rebase {trunk}`, resolve \
        the conflicts, and land once the result has passed review."
    ))
}

/// Moves the trunk from `from` to `to`, failing if it no longer points at `from`.
fn move_trunk(repo: &Repo, trunk: &str, from: &str, to: &str) -> Result<(), String> {
    let moved_elsewhere = || {
        format!("{trunk} moved while landing, probably landed from another session. Land again.")
    };
    if repo.tip(trunk).as_deref() != Some(from) {
        return Err(moved_elsewhere());
    }
    if let Some(checkout) = repo.checkout_of(trunk) {
        return git::output(&checkout, &["merge", "--ff-only", "--quiet", to])
            .map(drop)
            .ok_or_else(|| {
                format!(
                    "Couldn't fast-forward {trunk} in its checkout at {}. Commit or stash the \
                    changes there, then land again.",
                    checkout.display()
                )
            });
    }
    repo.git(&["update-ref", &format!("refs/heads/{trunk}"), to, from])
        .map(drop)
        .ok_or_else(moved_elsewhere)
}
