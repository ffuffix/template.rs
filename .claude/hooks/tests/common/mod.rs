use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let name = format!(
            "claude-hooks-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = env::temp_dir().join(name);
        fs::create_dir_all(&path).expect("the system temporary directory is writable");
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn run_hook(hook: &str, input: &str, environment: &[(&str, &OsStr)]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-hooks"))
        .arg(hook)
        .envs(environment.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cargo builds the hook binary before running its tests");
    let written = child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes());
    // A hook that ignores its input can exit before this write finishes.
    if let Err(error) = written {
        assert_eq!(error.kind(), ErrorKind::BrokenPipe, "{error}");
    }
    child
        .wait_with_output()
        .expect("the hook runs to completion")
}
