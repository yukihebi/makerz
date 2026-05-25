use std::env;
use std::process::ExitCode;

use makerz::cli::{self, Parsed};
use makerz::error::Error;
use makerz::makers;

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
            println!("{}", cli::version_text());
            Ok(())
        }
        Parsed::Help => {
            print!("{}", cli::help_text());
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
