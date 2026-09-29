//! Placeholder library. Replace it with your own code.

use std::error::Error;
use std::fmt;

/// Builds a greeting for `name`, ignoring surrounding whitespace.
///
/// # Errors
///
/// Returns [`GreetError::EmptyName`] if `name` is empty or only whitespace.
pub fn greet(name: &str) -> Result<String, GreetError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(GreetError::EmptyName);
    }
    Ok(format!("Hello, {name}!"))
}

/// Why a greeting could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GreetError {
    /// The name was empty or contained only whitespace.
    EmptyName,
}

impl fmt::Display for GreetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("name must not be empty"),
        }
    }
}

impl Error for GreetError {}
