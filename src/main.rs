use std::env;
use std::process::ExitCode;

mod caller;
mod cli;
mod directive_parser;
mod error;
mod location;
mod makers;

use caller::resolve_caller_env;
use cli::Parsed;
use directive_parser::parse as parse_makefile;
use error::Error;
use location::MakefileLocation;
use makers::Invocation;

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
    let location = MakefileLocation::find(&cwd)?;
    let parsed = parse_makefile(location.clone())?;

    let mut invocation = Invocation::new(location.dir().to_path_buf(), args);
    if let Some(entry) = resolve_caller_env(&parsed, &cwd) {
        invocation.push_env(entry);
    }
    invocation.run()
}
