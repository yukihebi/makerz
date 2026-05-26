use std::env;
use std::process::ExitCode;

mod cli;
mod error;
mod makers;

use cli::Parsed;
use error::Error;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("makerz: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}

fn run(args: Vec<String>) -> Result<(), Error> {
    match cli::parse(args)? {
        Parsed::Version => {
            print!("{}", cli::VERSION_TEXT);
            Ok(())
        }
        Parsed::Help => {
            print!("{}", cli::HELP_TEXT);
            Ok(())
        }
        Parsed::Init { extend: _ } => {
            unimplemented!("--init is implemented in a later PR")
        }
        Parsed::Passthrough { args } => {
            let argv = makers::build_args(&args);
            makers::spawn(makers::MAKERS_BINARY, &argv)
        }
    }
}
