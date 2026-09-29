use template_rs::{GreetError, greet};

#[test]
fn greets_by_name() {
    assert_eq!(greet("Ferris"), Ok("Hello, Ferris!".to_owned()));
}

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(greet("  Ferris\n"), Ok("Hello, Ferris!".to_owned()));
}

#[test]
fn rejects_blank_name() {
    assert_eq!(greet(" \t "), Err(GreetError::EmptyName));
}
