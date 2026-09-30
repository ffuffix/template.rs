use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

const HOOKS: &str = env!("CARGO_MANIFEST_DIR");

fn hook_commands() -> Vec<String> {
    let settings = Path::new(HOOKS).join("..").join("settings.json");
    let settings = fs::read_to_string(settings).expect(".claude/settings.json is readable");
    let settings: Value = serde_json::from_str(&settings).expect(".claude/settings.json is JSON");
    settings["hooks"]
        .as_object()
        .expect("settings.json has hooks")
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|matcher| matcher["hooks"].as_array())
        .flatten()
        .filter_map(|hook| hook["command"].as_str())
        .map(str::to_owned)
        .collect()
}

fn installed(shell: &str) -> bool {
    Command::new(shell)
        .arg("-c")
        .arg("exit 0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn project_with_spaces(shell: &str) -> PathBuf {
    let project = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("hook-commands")
        .join(shell)
        .join("my project");
    let hooks = project.join(".claude").join("hooks");
    copy_dir(&Path::new(HOOKS).join("src"), &hooks.join("src"));
    for file in ["Cargo.toml", "Cargo.lock"] {
        fs::copy(Path::new(HOOKS).join(file), hooks.join(file))
            .expect("the hooks package is copied");
    }
    project
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("the copy's directory can be created");
    for entry in fs::read_dir(from).expect("the hooks source is readable") {
        let source = entry.expect("the hooks source is readable").path();
        let destination = to.join(source.file_name().expect("entries have names"));
        if source.is_dir() {
            copy_dir(&source, &destination);
        } else {
            fs::copy(&source, &destination).expect("the hooks source is copied");
        }
    }
}

fn assert_hook_commands_run(shell: &str, flag: &str) {
    let project = project_with_spaces(shell);
    let target = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("hook-commands")
        .join("target");
    for template in hook_commands() {
        let command = template.replace("${CLAUDE_PROJECT_DIR}", &project.to_string_lossy());
        let output = Command::new(shell)
            .arg(flag)
            .arg(&command)
            .env("CLAUDE_PROJECT_DIR", &project)
            .env("CARGO_TARGET_DIR", &target)
            .stdin(Stdio::null())
            .output()
            .expect("the shell runs");

        assert!(
            output.status.success(),
            "{template} through {shell}: {output:?}"
        );
    }
}

#[test]
fn run_through_sh_from_a_path_with_spaces() {
    if installed("sh") {
        assert_hook_commands_run("sh", "-c");
    }
}

#[test]
fn run_through_powershell_from_a_path_with_spaces() {
    if installed("pwsh") {
        assert_hook_commands_run("pwsh", "-Command");
    }
}
