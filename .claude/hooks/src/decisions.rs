//! `SessionStart` hook: list the decisions waiting for the user in `.claude/decisions/`, so
//! Claude can raise them. Each file's first `# ` heading is its question. Prints nothing
//! when there are none.

use std::fs;
use std::process::ExitCode;

use crate::git::DECISIONS;
use crate::input;

pub(crate) fn run() -> ExitCode {
    let Ok(entries) = fs::read_dir(input::project_dir().join(DECISIONS)) else {
        return ExitCode::SUCCESS;
    };
    let mut decisions: Vec<(String, String)> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .map(|path| {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let question = fs::read_to_string(&path)
                .ok()
                .and_then(|text| {
                    text.lines()
                        .find_map(|line| line.strip_prefix("# "))
                        .map(|question| question.trim().to_owned())
                })
                .unwrap_or_default();
            (name, question)
        })
        .collect();
    if decisions.is_empty() {
        return ExitCode::SUCCESS;
    }
    decisions.sort();
    println!(
        "{} decision(s) in {DECISIONS}/ wait for the user. Raise them as the Landing section \
        of .claude/CLAUDE.md says:",
        decisions.len()
    );
    for (name, question) in decisions {
        println!("- {name}: {question}");
    }
    ExitCode::SUCCESS
}
