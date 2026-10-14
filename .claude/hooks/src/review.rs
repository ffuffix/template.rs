//! The review gate. A change must go through the `code-reviewer` agent until it finds
//! nothing to fix, before Claude's turn can end and before the change can leave this
//! machine. One subcommand serves every event it hooks, told apart by `hook_event_name`:
//!
//! - `Stop` holds the turn while the change has no clean review, unless a reviewer is
//!   still running in the background. Once the change has a clean review, it reminds
//!   Claude, once per change, to land it.
//! - `SubagentStart` notes which change the reviewer starts on, so its verdict covers
//!   what it saw even if Claude edits the change while it runs.
//! - `SubagentStop`, and `PreToolUse` on `SubagentHandback`, read the verdict line that
//!   ends the reviewer's report. In auto mode the report arrives through `SubagentHandback`
//!   and the subagent's last message is only a closing note, so a hand-back without the
//!   line goes back to the reviewer, while a last message without one is ignored.
//! - `SessionStart` tells Claude when git is too old for the gate to work.
//! - `PreToolUse` on Bash and PowerShell denies `git commit` on the trunk, so work happens
//!   on a branch that `cargo land` can rebase, and `git push` of an unreviewed change.
//!
//! The gate sends Claude back at most `MAX_ROUNDS` times in a turn. Past that it lets the
//! turn end after asking Claude to report what is left, and stays quiet about that change,
//! so a review that won't come clean can't loop. A turn starts when `stop_hook_active` is
//! false, meaning Claude is stopping on its own rather than because a stop hook sent it back.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::{Value, json};

use crate::git::{self, Change, Repo};
use crate::input;

const REVIEWER: &str = "code-reviewer";
const VERDICT: &str = "Verdict:";
const MAX_ROUNDS: u64 = 3;
const STATE_FILE: &str = "claude-review.json";
const REMEMBERED: usize = 100;

const MISSING_VERDICT: &str = "End the report with the line `Verdict: <n> fix, <m> ask`, \
    counting the findings in each section. The review gate reads it.";
const LAND: &str = "The change passed review but isn't on the trunk yet. Commit it, run \
    `cargo land`, and report as the Landing section of `.claude/CLAUDE.md` says. If the user \
    asked you not to land it, end your turn.";
const PUSH_UNREVIEWED: &str = "This change hasn't passed review, so it can't leave this \
    machine yet. Run the `code-reviewer` agent and fix its Fix findings first; `cargo land` \
    then pushes the reviewed change. If the user asked to push unreviewed work, tell them the \
    review gate stops it and that they can push it themselves.";

#[derive(Default)]
struct State {
    passed: Vec<String>,
    /// The review id each running reviewer started on, by agent id.
    started: Vec<(String, String)>,
    rounds: u64,
    gave_up_on: Option<String>,
    reminded_of: Option<String>,
}

impl State {
    fn load(path: &Path) -> Self {
        let Some(state) = fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        else {
            return Self::default();
        };
        let text = |key: &str| state[key].as_str().map(str::to_owned);
        Self {
            passed: state["passed"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            started: state["started"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|pair| {
                    Some((pair[0].as_str()?.to_owned(), pair[1].as_str()?.to_owned()))
                })
                .collect(),
            rounds: state["rounds"].as_u64().unwrap_or(0),
            gave_up_on: text("gave_up_on"),
            reminded_of: text("reminded_of"),
        }
    }

    fn save(&self, path: &Path) {
        let state = json!({
            "passed": self.passed,
            "started": self.started,
            "rounds": self.rounds,
            "gave_up_on": self.gave_up_on,
            "reminded_of": self.reminded_of,
        });
        let _ = fs::write(path, state.to_string());
    }

    fn pass(&mut self, id: String) {
        if !self.passed.contains(&id) {
            self.passed.push(id);
        }
        let forgotten = self.passed.len().saturating_sub(REMEMBERED);
        self.passed.drain(..forgotten);
    }

    fn start(&mut self, agent: &str, id: String) {
        self.started.retain(|(running, _)| running != agent);
        self.started.push((agent.to_owned(), id));
        let forgotten = self.started.len().saturating_sub(REMEMBERED);
        self.started.drain(..forgotten);
    }

    fn finish(&mut self, agent: &str) -> Option<String> {
        let position = self
            .started
            .iter()
            .position(|(running, _)| running == agent)?;
        Some(self.started.remove(position).1)
    }
}

pub(crate) fn run() -> ExitCode {
    let Some(input) = input::read() else {
        return ExitCode::SUCCESS;
    };
    let Some(repo) = Repo::open(&input::cwd(&input)) else {
        return ExitCode::SUCCESS;
    };
    let from_reviewer = input["agent_type"].as_str() == Some(REVIEWER);
    let agent = input["agent_id"].as_str();
    let response = match (
        input["hook_event_name"].as_str(),
        input["tool_name"].as_str(),
    ) {
        (Some("SessionStart"), _) => git::too_old().map(|version| {
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": format!(
                        "{} Tell the user before other work.",
                        git::too_old_message(&version)
                    ),
                }
            })
        }),
        (Some("Stop"), _) if !reviewer_running(&input) => {
            stop(&repo, input["stop_hook_active"] == true)
        }
        (Some("SubagentStart"), _) if from_reviewer => {
            start(&repo, agent);
            None
        }
        (Some("SubagentStop"), _) if from_reviewer => {
            let report = input["last_assistant_message"].as_str().unwrap_or_default();
            record(&repo, agent, report);
            None
        }
        (Some("PreToolUse"), Some("SubagentHandback")) if from_reviewer => {
            let report = input["tool_input"]["message"].as_str().unwrap_or_default();
            (!record(&repo, agent, report)).then(|| deny(MISSING_VERDICT))
        }
        (Some("PreToolUse"), Some("Bash" | "PowerShell")) => command(
            &repo,
            input["tool_input"]["command"].as_str().unwrap_or_default(),
        ),
        _ => None,
    };
    if let Some(response) = response {
        println!("{response}");
    }
    ExitCode::SUCCESS
}

/// Whether a review of the change with this id came back with nothing to fix.
pub(crate) fn passed(repo: &Repo, id: &str) -> bool {
    repo.private_file(STATE_FILE)
        .is_some_and(|path| State::load(&path).passed.iter().any(|passed| passed == id))
}

/// A reviewer running in the background wakes Claude with its report when it finishes,
/// so a stop while it runs is Claude waiting for it, not skipping review.
fn reviewer_running(input: &Value) -> bool {
    input["background_tasks"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|task| task["agent_type"] == REVIEWER)
}

fn stop(repo: &Repo, sent_back: bool) -> Option<Value> {
    let path = repo.private_file(STATE_FILE)?;
    let mut state = State::load(&path);
    // Saved before any early return, or a later stop in this turn would count rounds
    // left over from an earlier one.
    if !sent_back && state.rounds > 0 {
        state.rounds = 0;
        state.save(&path);
    }
    let change = repo.change()?;
    if !change.exists {
        return None;
    }
    let reason = match &change.review_id {
        Some(id) if !state.passed.contains(id) => {
            if state.gave_up_on.as_ref() == Some(id) {
                return None;
            }
            if state.rounds >= MAX_ROUNDS {
                state.gave_up_on = Some(id.clone());
                format!(
                    "This change still has no clean review after {MAX_ROUNDS} rounds, so stop \
                    here. Tell the user which Fix findings are left and why they resist a fix, \
                    or that no review came back. Don't land the change: `cargo land` refuses it \
                    until a review comes back clean."
                )
            } else {
                state.rounds += 1;
                review_request(&change, state.rounds)
            }
        }
        _ => {
            if state.reminded_of.as_ref() == Some(&change.tree) {
                return None;
            }
            state.reminded_of = Some(change.tree);
            LAND.to_owned()
        }
    };
    state.save(&path);
    Some(json!({ "decision": "block", "reason": reason }))
}

fn review_request(change: &Change, round: u64) -> String {
    let scope = match &change.base {
        Some(base) => {
            let short = base.get(..12).unwrap_or(base);
            format!("everything since {short}: `git diff {short}` plus untracked files")
        }
        None => "every file, since the repository has no commits yet".to_owned(),
    };
    format!(
        "Review round {round} of {MAX_ROUNDS}: this change hasn't passed review. Run the \
        `code-reviewer` agent in the foreground on {scope}, and wait for its report. Fix every Fix finding \
        without asking, then end your turn again, and the gate checks whatever changed. \
        Handle Ask findings as the Landing section of `.claude/CLAUDE.md` says."
    )
}

fn start(repo: &Repo, agent: Option<&str>) {
    if let Some(agent) = agent
        && let Some(id) = repo.change().and_then(|change| change.review_id)
        && let Some(path) = repo.private_file(STATE_FILE)
    {
        let mut state = State::load(&path);
        state.start(agent, id);
        state.save(&path);
    }
}

/// Records a clean review of the change the reviewer started on, or of the current change
/// when its start went unrecorded. Returns whether `report` ends with a verdict line at all.
fn record(repo: &Repo, agent: Option<&str>, report: &str) -> bool {
    let Some(fixes) = fixes(report) else {
        return false;
    };
    let Some(path) = repo.private_file(STATE_FILE) else {
        return true;
    };
    let mut state = State::load(&path);
    let reviewed = agent
        .and_then(|agent| state.finish(agent))
        .or_else(|| repo.change().and_then(|change| change.review_id));
    if fixes == 0
        && let Some(id) = reviewed
    {
        state.pass(id);
    }
    state.save(&path);
    true
}

/// The Fix count from a line such as `Verdict: 2 fix, 1 ask`, which may be wrapped in
/// Markdown emphasis.
fn fixes(report: &str) -> Option<u64> {
    let decoration = ['*', '_', '`', '.', ' '];
    let line = report.lines().rev().find_map(|line| {
        line.trim()
            .trim_matches(decoration)
            .strip_prefix(VERDICT)
            .map(|counts| counts.trim_start_matches(decoration))
    })?;
    let mut counts = line.split(',').map(|part| {
        let mut words = part.split_whitespace();
        Some((words.next()?.parse::<u64>().ok()?, words.next()?))
    });
    match (counts.next()??, counts.next()??, counts.next()) {
        ((fixes, "fix"), (_, "ask"), None) => Some(fixes),
        _ => None,
    }
}

fn command(repo: &Repo, command: &str) -> Option<Value> {
    let git = git_commands(&commands(command));
    // `git switch -c work && git commit` commits on the new branch, though the hook sees the
    // checkout as it was before the command, so only a commit before any new branch counts.
    let commits_before_branching = git
        .iter()
        .take_while(|(subcommand, arguments)| {
            let flags: &[&str] = match *subcommand {
                "switch" => &["-c", "-C", "--create", "--force-create"],
                "checkout" => &["-b", "-B"],
                _ => &[],
            };
            !arguments.iter().any(|argument| flags.contains(argument))
        })
        .any(|(subcommand, _)| *subcommand == "commit");
    if commits_before_branching
        && let Some(trunk) = repo.trunk()
        && repo.branch().as_ref() == Some(&trunk)
    {
        return Some(deny(&format!(
            "Commits go on a branch, not on {trunk}: `cargo land` moves {trunk} once the \
            change has passed review. Run `git switch -c <name>` first; uncommitted edits \
            come along."
        )));
    }
    if git.iter().any(|(subcommand, _)| *subcommand == "push") {
        let id = repo.change()?.review_id?;
        if !passed(repo, &id) {
            return Some(deny(PUSH_UNREVIEWED));
        }
    }
    None
}

/// The words of each command in `command`, in order and without leading variable
/// assignments, as opposed to text that only mentions a command, such as a string or a line
/// of a heredoc. Claude Code's `if` filter runs the hook whenever it can't tell, so the
/// checks here can't rely on it.
fn commands(command: &str) -> Vec<Vec<&str>> {
    command
        .split(['\n', ';', '&', '|', '(', ')'])
        .map(|part| {
            part.split_whitespace()
                .skip_while(|word| word.contains('=') && !word.starts_with('-'))
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .collect()
}

/// Each `git <subcommand> <arguments>` among `commands`, in order.
fn git_commands<'a>(commands: &[Vec<&'a str>]) -> Vec<(&'a str, Vec<&'a str>)> {
    commands
        .iter()
        .filter_map(|words| {
            let mut words = words.iter().copied();
            if words.next() != Some("git") {
                return None;
            }
            while let Some(word) = words.next() {
                match word {
                    "-C" | "-c" | "--git-dir" | "--work-tree" => {
                        words.next();
                    }
                    option if option.starts_with('-') => {}
                    subcommand => return Some((subcommand, words.collect())),
                }
            }
            None
        })
        .collect()
}

fn deny(reason: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    })
}
