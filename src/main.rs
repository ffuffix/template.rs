use std::env;
use std::process::ExitCode;

use template_rs::greet;

fn main() -> ExitCode {
    let name = env::args_os().nth(1).unwrap_or_default();
    match greet(&name.to_string_lossy()) {
        Ok(greeting) => {
            println!("{greeting}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
