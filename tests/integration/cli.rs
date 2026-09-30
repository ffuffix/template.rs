use crate::common::run_binary;

#[test]
fn prints_greeting_and_exits_successfully() {
    let output = run_binary(&["Ferris"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, Ferris!\n");
}

#[test]
fn reports_missing_name_on_stderr_with_exit_code_two() {
    let output = run_binary::<&str>(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: name must not be empty\n"
    );
}

#[cfg(unix)]
#[test]
fn replaces_invalid_unicode_in_name() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let output = run_binary(&[OsStr::from_bytes(b"Ferr\xFFis")]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Hello, Ferr\u{FFFD}is!\n"
    );
}
