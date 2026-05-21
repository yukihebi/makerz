use std::env;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = run(args);
    exec_cargo_make(&args)
}

fn run(args: Vec<String>) -> Vec<String> {
    args
}

fn exec_cargo_make(args: &[String]) -> ExitCode {
    let status = Command::new("makers")
        .args(args)
        .status()
        .expect("failed to invoke `makers`; is cargo-make installed and on PATH?");
    match status.code() {
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::FAILURE,
    }
}
