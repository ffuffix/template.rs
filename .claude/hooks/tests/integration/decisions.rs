use std::process::Output;

use crate::common::{Repo, run_hook};

fn start_session(repo: &Repo) -> Output {
    run_hook(
        "decisions",
        "{}",
        &[("CLAUDE_PROJECT_DIR", repo.path().as_os_str())],
    )
}

#[test]
fn stays_silent_without_decisions() {
    let repo = Repo::new();

    let output = start_session(&repo);

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn lists_each_decision_with_its_question() {
    let repo = Repo::new();
    repo.write(
        ".claude/decisions/keep-cache.md",
        "# Keep the cache?\n\n- **Where:** `src/lib.rs:3`\n",
    );
    repo.write(
        ".claude/decisions/accept-tabs.md",
        "# Accept tabs in names?\n",
    );

    let output = start_session(&repo);

    assert!(output.status.success(), "{output:?}");
    let listed = String::from_utf8_lossy(&output.stdout);
    assert!(listed.contains("2 decision(s)"), "{listed}");
    let tabs = listed
        .find("- accept-tabs.md: Accept tabs in names?")
        .expect("lists the first decision");
    let cache = listed
        .find("- keep-cache.md: Keep the cache?")
        .expect("lists the second decision");
    assert!(tabs < cache, "{listed}");
}
