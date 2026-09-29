use std::env;
use std::process::ExitCode;

use template_rs::greet;

fn main() -> ExitCode {
    let name = env::args().nth(1).unwrap_or_default();
    match greet(&name) {
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
