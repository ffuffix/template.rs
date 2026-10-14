use serde_json::{Value, json};

use crate::common::{Repo, TempDir};

const CLEAN: &str = "## Ask\n1. `src/lib.rs:3`: keep the cache?\n\nVerdict: 0 fix, 1 ask";
const UNCLEAN: &str = "## Fix\n1. `src/lib.rs:3`: panics on empty input.\n\nVerdict: 1 fix, 0 ask";

fn on_branch() -> Repo {
    let repo = Repo::new();
    repo.git(&["switch", "-q", "-c", "work"]);
    repo
}

fn stop(repo: &Repo, sent_back: bool) -> Option<Value> {
    repo.review_event(json!({ "hook_event_name": "Stop", "stop_hook_active": sent_back }))
}

fn hand_back(repo: &Repo, report: &str) -> Option<Value> {
    repo.review_event(json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "SubagentHandback",
        "agent_type": "code-reviewer",
        "tool_input": { "message": report },
    }))
}

fn bash(repo: &Repo, command: &str) -> Option<Value> {
    repo.review_event(json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": { "command": command },
    }))
}

fn held(response: Option<Value>) -> String {
    let response = response.expect("the hook holds the turn");
    assert_eq!(response["decision"], "block", "{response}");
    response["reason"]
        .as_str()
        .expect("a block has a reason")
        .to_owned()
}

fn denied(response: Option<Value>) -> String {
    let response = response.expect("the hook denies the call");
    let output = &response["hookSpecificOutput"];
    assert_eq!(output["permissionDecision"], "deny", "{response}");
    output["permissionDecisionReason"]
        .as_str()
        .expect("a denial has a reason")
        .to_owned()
}

fn asks_for_review(response: Option<Value>) -> bool {
    held(response).contains("Run the `code-reviewer` agent")
}

fn asks_to_land(response: Option<Value>) -> bool {
    held(response).contains("cargo land")
}

#[test]
fn lets_a_turn_end_without_a_change() {
    let repo = on_branch();

    assert_eq!(stop(&repo, false), None);
}

#[test]
fn holds_a_turn_until_an_edit_is_reviewed() {
    let repo = on_branch();
    repo.write("README.md", "# Project\n\nMore.\n");

    let reason = held(stop(&repo, false));

    assert!(reason.contains("Review round 1 of 3"), "{reason}");
    let base = repo.git(&["rev-parse", "main"]);
    assert!(
        reason.contains(&format!("git diff {}", &base[..12])),
        "{reason}"
    );
}

#[test]
fn counts_untracked_files_as_part_of_the_change() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn counts_commits_on_the_branch_as_part_of_the_change() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn asks_once_to_land_a_change_with_a_clean_review() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    held(stop(&repo, false));

    repo.reviewer_says(CLEAN);

    assert!(asks_to_land(stop(&repo, true)));
    assert_eq!(stop(&repo, true), None);
    assert_eq!(stop(&repo, false), None);
}

#[test]
fn keeps_holding_while_the_review_finds_something_to_fix() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    repo.reviewer_says(UNCLEAN);

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn asks_for_another_review_after_an_edit() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.reviewer_says(CLEAN);

    repo.write("src/new.rs", "pub fn new() -> u8 { 0 }\n");

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn asks_for_another_review_after_an_edit_that_only_changes_whitespace() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub const SEPARATOR: &str = \"a b\";\n");
    repo.reviewer_says(CLEAN);

    repo.write("src/new.rs", "pub const SEPARATOR: &str = \"ab\";\n");

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn passes_only_the_change_the_reviewer_started_on() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    reviewer_event(&repo, "SubagentStart", None);
    repo.write("src/new.rs", "pub fn new() -> u8 { 0 }\n");

    reviewer_event(&repo, "SubagentStop", Some(CLEAN));

    assert!(asks_for_review(stop(&repo, false)));
    repo.write("src/new.rs", "pub fn new() {}\n");
    assert!(asks_to_land(stop(&repo, false)));
}

fn reviewer_event(repo: &Repo, event: &str, report: Option<&str>) {
    repo.review_event(json!({
        "hook_event_name": event,
        "agent_type": "code-reviewer",
        "agent_id": "reviewer-1",
        "last_assistant_message": report,
    }));
}

#[test]
fn reads_the_verdict_from_a_report_handed_back_in_auto_mode() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    assert_eq!(
        hand_back(&repo, "Nothing to report.\n\n**Verdict:** 0 fix, 0 ask"),
        None
    );
    assert!(asks_to_land(stop(&repo, false)));
}

#[test]
fn sends_a_handed_back_report_without_a_verdict_back_to_the_reviewer() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    let reason = denied(hand_back(&repo, "Looks fine to me."));

    assert!(reason.contains("Verdict: <n> fix, <m> ask"), "{reason}");
    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn ignores_a_verdict_from_another_agent() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    repo.review_event(json!({
        "hook_event_name": "SubagentStop",
        "agent_type": "Explore",
        "last_assistant_message": CLEAN,
    }));

    assert!(asks_for_review(stop(&repo, false)));
}

#[test]
fn gives_up_after_three_rounds_and_stays_quiet_about_that_change() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    assert!(held(stop(&repo, false)).contains("round 1"));
    assert!(held(stop(&repo, true)).contains("round 2"));
    assert!(held(stop(&repo, true)).contains("round 3"));
    let reason = held(stop(&repo, true));
    assert!(reason.contains("after 3 rounds"), "{reason}");
    assert_eq!(stop(&repo, true), None);
    assert_eq!(stop(&repo, false), None);
}

#[test]
fn lets_claude_wait_for_a_reviewer_running_in_the_background() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");

    let waiting = repo.review_event(json!({
        "hook_event_name": "Stop",
        "stop_hook_active": false,
        "background_tasks": [
            { "id": "a1", "type": "subagent", "status": "running", "agent_type": "code-reviewer" },
        ],
    }));

    assert_eq!(waiting, None);
    assert!(held(stop(&repo, true)).contains("round 1"));
}

#[test]
fn counts_rounds_again_in_a_new_turn() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    held(stop(&repo, false));
    held(stop(&repo, true));

    assert!(held(stop(&repo, false)).contains("round 1"));
}

#[test]
fn keeps_a_review_through_a_commit_and_a_clean_rebase() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.reviewer_says(CLEAN);
    repo.commit_all("Add new");
    repo.git(&["switch", "-q", "main"]);
    repo.write("other.txt", "elsewhere\n");
    repo.commit_all("Elsewhere");
    repo.git(&["switch", "-q", "work"]);

    repo.git(&["rebase", "-q", "main"]);

    assert!(asks_to_land(stop(&repo, false)));
}

#[test]
fn lets_decisions_land_without_review() {
    let repo = on_branch();

    repo.write(".claude/decisions/cache.md", "# Keep the cache?\n");

    assert!(asks_to_land(stop(&repo, false)));
}

#[test]
fn keeps_a_review_when_a_decision_is_added_after_it() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.reviewer_says(CLEAN);

    repo.write(".claude/decisions/cache.md", "# Keep the cache?\n");

    assert!(asks_to_land(stop(&repo, false)));
}

#[test]
fn denies_a_commit_on_the_trunk() {
    let repo = Repo::new();
    repo.write("README.md", "# Project\n\nMore.\n");

    let reason = denied(bash(&repo, "git commit -am 'More'"));

    assert!(reason.contains("git switch -c"), "{reason}");
}

#[test]
fn allows_a_commit_on_a_branch() {
    let repo = on_branch();
    repo.write("README.md", "# Project\n\nMore.\n");

    assert_eq!(bash(&repo, "git commit -am 'More'"), None);
}

#[test]
fn denies_pushing_a_change_until_it_is_reviewed() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");

    assert!(denied(bash(&repo, "git push -u origin work")).contains("hasn't passed review"));
    repo.reviewer_says(CLEAN);
    assert_eq!(bash(&repo, "git push -u origin work"), None);
}

#[test]
fn stays_out_of_a_directory_that_is_not_a_repository() {
    let repo = Repo::new();
    let elsewhere = TempDir::new();

    let response = repo.review_event_in(
        elsewhere.path(),
        json!({ "hook_event_name": "Stop", "stop_hook_active": false }),
    );

    assert_eq!(response, None);
}

#[test]
fn denies_a_commit_on_the_trunk_while_a_tag_has_its_name() {
    let repo = Repo::new();
    repo.git(&["tag", "main"]);
    repo.write("README.md", "# Project\n\nMore.\n");

    assert!(denied(bash(&repo, "git commit -am 'More'")).contains("git switch -c"));
}

#[test]
fn reacts_only_to_git_commands_that_run() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    repo.commit_all("Add new");

    assert_eq!(bash(&repo, "echo 'then git push it'"), None);
    assert_eq!(bash(&repo, "cat <<'EOF'\nRun `git push` later.\nEOF"), None);
    assert!(denied(bash(&repo, "cargo ci && git -C . push")).contains("hasn't passed review"));
    assert!(denied(bash(&repo, "FOO=1 git push")).contains("hasn't passed review"));
}

#[test]
fn starts_counting_rounds_afresh_even_when_a_turn_ends_early() {
    let repo = on_branch();
    repo.write("src/new.rs", "pub fn new() {}\n");
    held(stop(&repo, false));
    held(stop(&repo, true));
    held(stop(&repo, true));
    assert!(held(stop(&repo, true)).contains("after 3 rounds"));

    assert_eq!(stop(&repo, false), None);
    repo.write("src/new.rs", "pub fn new() -> u8 { 0 }\n");

    assert!(held(stop(&repo, true)).contains("round 1"));
}

#[cfg(unix)]
#[test]
fn says_at_session_start_when_git_is_too_old() {
    let repo = Repo::new();
    let old_git = repo.old_git("git version 2.37.1 (Apple Git-137.1)");
    let event = json!({ "hook_event_name": "SessionStart", "cwd": repo.path() });

    let output = repo.hook_with(repo.path(), "review", &event.to_string(), &old_git);

    let printed: Value = serde_json::from_slice(&output.stdout).expect("the hook prints JSON");
    let context = printed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("the hook adds context");
    assert!(context.contains("git 2.37.1"), "{context}");
}

#[test]
fn says_nothing_at_session_start_with_a_recent_git() {
    let repo = Repo::new();

    assert_eq!(
        repo.review_event(json!({ "hook_event_name": "SessionStart" })),
        None
    );
}
