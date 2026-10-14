use std::fs;
use std::path::Path;

use crate::common::{TempDir, run_hook_in};

#[test]
fn lists_every_cargo_command_in_the_ci_workflow() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let workflow = fs::read_to_string(repository.join(".github/workflows/ci.yml"))
        .expect("the CI workflow is readable");

    let output = run_hook_in(&repository, "ci --list", "", &[]);

    assert!(output.status.success(), "{output:?}");
    let checks = String::from_utf8_lossy(&output.stdout);
    let commands = workflow
        .lines()
        .map(|line| line.trim().trim_start_matches("run: "))
        .filter(|line| line.starts_with("cargo "));
    // A check may set variables before `cargo`, and packages with `--allow-dirty` so it
    // can run before the change is committed.
    let checks: Vec<&str> = checks
        .lines()
        .filter_map(|check| check.find("cargo ").map(|start| &check[start..]))
        .map(|check| check.trim_end_matches(" --allow-dirty"))
        .collect();
    for command in commands {
        assert!(
            checks.contains(&command),
            "`cargo ci` lacks `{command}`: {checks:#?}"
        );
    }
}

fn crate_in(directory: &Path, manifest_extra: &str) {
    fs::create_dir_all(directory.join("src")).expect("the crate directory can be created");
    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\nname = \"scratch\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{manifest_extra}"
        ),
    )
    .expect("the manifest can be written");
    fs::write(directory.join("src/lib.rs"), "pub fn scratch() {}\n")
        .expect("the library can be written");
    // The checks run with `--locked`, which needs a lock file to exist.
    fs::write(
        directory.join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"scratch\"\nversion = \"0.1.0\"\n",
    )
    .expect("the lock file can be written");
}

fn ci(directory: &Path, arguments: &str) -> String {
    let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ci-scratch");
    let output = run_hook_in(
        directory,
        &format!("ci {arguments}"),
        "",
        &[("CARGO_TARGET_DIR", target.as_os_str())],
    );
    assert!(output.status.success(), "{output:?}");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn leaves_out_the_package_check_for_a_crate_that_is_not_published() {
    let directory = TempDir::new();
    crate_in(directory.path(), "publish = false\n");

    let checks = ci(directory.path(), "--list");

    assert!(checks.contains("cargo test --locked"), "{checks}");
    assert!(!checks.contains("cargo package"), "{checks}");
}

#[test]
fn skips_the_minimum_version_check_when_that_toolchain_is_missing() {
    let directory = TempDir::new();
    crate_in(
        directory.path(),
        "rust-version = \"1.56.1\"\npublish = false\n",
    );

    let report = ci(directory.path(), "");

    assert!(report.contains("passed   cargo test --locked"), "{report}");
    assert!(
        report.contains("skipped  RUSTUP_TOOLCHAIN=\"1.56.1\" cargo check"),
        "{report}"
    );
}
