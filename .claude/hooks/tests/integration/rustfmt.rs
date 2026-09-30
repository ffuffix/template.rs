use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::common::{TempDir, run_hook};

const UNFORMATTED: &str = "pub fn answer( )->u32{42}\n";
const FORMATTED: &str = "pub fn answer() -> u32 {\n    42\n}\n";
const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

struct Sample {
    root: TempDir,
    package: PathBuf,
    library: PathBuf,
}

impl Sample {
    fn new() -> Self {
        let root = TempDir::new();
        let package = root.path().join("sample");
        fs::create_dir_all(package.join("src")).expect("the sample package can be created");
        fs::write(package.join("Cargo.toml"), MANIFEST).expect("the manifest can be written");
        let library = package.join("src").join("lib.rs");
        fs::write(&library, UNFORMATTED).expect("the library can be written");
        Self {
            root,
            package,
            library,
        }
    }

    fn library(&self) -> String {
        fs::read_to_string(&self.library).expect("the library is readable")
    }
}

fn edit(cwd: &Path, file: &Path) -> String {
    json!({ "cwd": cwd, "tool_input": { "file_path": file } }).to_string()
}

#[test]
fn formats_owning_package_even_when_project_dir_is_elsewhere() {
    let sample = Sample::new();
    let main_checkout = sample.root.path().join("main-checkout");

    let output = run_hook(
        "rustfmt",
        &edit(sample.root.path(), &sample.library),
        &[("CLAUDE_PROJECT_DIR", main_checkout.as_os_str())],
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(sample.library(), FORMATTED);
}

#[test]
fn resolves_relative_file_path_against_cwd() {
    let sample = Sample::new();

    let output = run_hook(
        "rustfmt",
        &edit(&sample.package, Path::new("src/lib.rs")),
        &[],
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(sample.library(), FORMATTED);
}

#[test]
fn ignores_non_rust_files() {
    let sample = Sample::new();
    let readme = sample.package.join("README.md");
    fs::write(&readme, "# sample\n").expect("the readme can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &readme), &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(sample.library(), UNFORMATTED);
}

#[test]
fn ignores_rust_file_outside_any_cargo_package() {
    let sample = Sample::new();
    let loose = sample.root.path().join("loose.rs");
    fs::write(&loose, UNFORMATTED).expect("the loose file can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &loose), &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&loose).expect("the loose file is readable"),
        UNFORMATTED
    );
}

#[test]
fn reports_syntax_error_to_claude_with_exit_two() {
    let sample = Sample::new();
    fs::write(&sample.library, "pub fn broken( {\n").expect("the library can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &sample.library), &[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("rustfmt failed for lib.rs"));
}

#[test]
fn stays_silent_while_a_declared_module_is_not_written_yet() {
    let sample = Sample::new();
    fs::write(&sample.library, "mod parser;\n").expect("the library can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &sample.library), &[]);

    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn ignores_malformed_input() {
    let output = run_hook("rustfmt", "not json", &[]);

    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn leaves_other_files_in_the_package_alone() {
    let sample = Sample::new();
    let binary = sample.package.join("src").join("main.rs");
    fs::write(&binary, "fn main( ){}\n").expect("the binary can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &sample.library), &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(sample.library(), FORMATTED);
    assert_eq!(
        fs::read_to_string(&binary).expect("the binary is readable"),
        "fn main( ){}\n"
    );
}

#[test]
fn ignores_syntax_errors_in_other_files() {
    let sample = Sample::new();
    fs::write(sample.package.join("src").join("main.rs"), "fn main( {\n")
        .expect("the binary can be written");

    let output = run_hook("rustfmt", &edit(sample.root.path(), &sample.library), &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(sample.library(), FORMATTED);
}

#[test]
fn uses_the_edition_a_workspace_member_inherits() {
    let root = TempDir::new();
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"member\"]\nresolver = \"3\"\n\n[workspace.package]\nedition = \"2024\"\n",
    )
    .expect("the workspace manifest can be written");
    let member = root.path().join("member");
    fs::create_dir_all(member.join("src")).expect("the member can be created");
    fs::write(
        member.join("Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition.workspace = true\n",
    )
    .expect("the member manifest can be written");
    let library = member.join("src").join("lib.rs");
    fs::write(&library, "pub async fn answer( )->u32{42}\n").expect("the library can be written");

    let output = run_hook("rustfmt", &edit(root.path(), &library), &[]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&library).expect("the library is readable"),
        "pub async fn answer() -> u32 {\n    42\n}\n"
    );
}
