use std::process::ExitCode;

fn main() -> ExitCode {
    match hod::run() {
        Ok(code) => code,
        Err(error) => hod::report(&error),
    }
}
