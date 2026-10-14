//! What Claude Code hands a hook: its event as JSON on stdin, and the project directory.

use std::env;
use std::io::{self, Read};
use std::path::PathBuf;

use serde_json::Value;

pub(crate) fn read() -> Option<Value> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).ok()?;
    serde_json::from_str(&input).ok()
}

pub(crate) fn cwd(input: &Value) -> PathBuf {
    PathBuf::from(
        input["cwd"]
            .as_str()
            .filter(|cwd| !cwd.is_empty())
            .unwrap_or("."),
    )
}

pub(crate) fn project_dir() -> PathBuf {
    env::var_os("CLAUDE_PROJECT_DIR").map_or_else(|| PathBuf::from("."), PathBuf::from)
}
