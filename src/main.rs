use std::env;
use std::process::ExitCode;

mod cli;
mod directive_parser;
mod error;
mod location;
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
        Parsed::Passthrough { args } => passthrough(args),
    }
}

fn passthrough(args: Vec<String>) -> Result<(), Error> {
    let cwd = env::current_dir().map_err(Error::Cwd)?;
    let location = location::MakefileLocation::find(&cwd)?;
    let _parsed = directive_parser::parse(location.clone())?;
    let argv = makers::build_args(location.dir(), &args);
    makers::spawn(makers::MAKERS_BINARY, &argv)
}
