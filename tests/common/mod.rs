use std::ffi::OsStr;
use std::process::{Command, Output};

pub(crate) fn run_binary<S: AsRef<OsStr>>(arguments: &[S]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_template_rs"))
        .args(arguments)
        .output()
        .expect("cargo builds the binary before running integration tests")
}
