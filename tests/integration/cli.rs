use crate::common::run_binary;

#[test]
fn prints_greeting_and_exits_successfully() {
    let output = run_binary(&["Ferris"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, Ferris!\n");
}

#[test]
fn reports_missing_name_on_stderr_with_exit_code_two() {
    let output = run_binary(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: name must not be empty\n"
    );
}
